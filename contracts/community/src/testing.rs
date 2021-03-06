use crate::contract::{execute, instantiate, query};
use crate::mock_querier::mock_dependencies;

use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    from_binary, to_binary, BankMsg, Coin, CosmosMsg, Decimal, ReplyOn, StdError, SubMsg, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use glow_protocol::community::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use glow_protocol::lotto::ExecuteMsg as LottoMsg;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::ExecuteMsg as TerraswapExecuteMsg;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        stable_denom: "uusd".to_string(),
        glow_token: "glow".to_string(),
        lotto_contract: "lotto".to_string(),
        gov_contract: "gov".to_string(),
        terraswap_factory: "terraswap".to_string(),
        spend_limit: Uint128::from(1000000u128),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // it worked, let's query the state
    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!("owner", config.owner.as_str());
    assert_eq!("glow", config.glow_token.as_str());
    assert_eq!(Uint128::from(1000000u128), config.spend_limit);
}

#[test]
fn update_spend_limit() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        stable_denom: "uusd".to_string(),
        glow_token: "glow".to_string(),
        lotto_contract: "lotto".to_string(),
        gov_contract: "gov".to_string(),
        terraswap_factory: "terraswap".to_string(),
        spend_limit: Uint128::from(1000000u128),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // it worked, let's query the state
    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!("owner", config.owner.as_str());
    assert_eq!("glow", config.glow_token.as_str());
    assert_eq!(Uint128::from(1000000u128), config.spend_limit);

    let msg = ExecuteMsg::UpdateConfig {
        spend_limit: Some(Uint128::from(500000u128)),
        owner: None,
    };
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());

    match res {
        Err(StdError::GenericErr { msg, .. }) => {
            assert_eq!(msg, "Unauthorized")
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            owner: "owner".to_string(),
            stable_denom: "uusd".to_string(),
            glow_token: "glow".to_string(),
            lotto_contract: "lotto".to_string(),
            gov_contract: "gov".to_string(),
            terraswap_factory: "terraswap".to_string(),
            spend_limit: Uint128::from(500000u128),
        }
    );
}

#[test]
fn transfer_ownership_gov() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        stable_denom: "uusd".to_string(),
        glow_token: "glow".to_string(),
        lotto_contract: "lotto".to_string(),
        gov_contract: "gov".to_string(),
        terraswap_factory: "terraswap".to_string(),
        spend_limit: Uint128::from(1000000u128),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // it worked, let's query the state
    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!("owner", config.owner.as_str());
    assert_eq!("glow", config.glow_token.as_str());
    assert_eq!(Uint128::from(1000000u128), config.spend_limit);

    let msg = ExecuteMsg::UpdateConfig {
        spend_limit: None,
        owner: Some("gov".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());

    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            owner: "gov".to_string(),
            stable_denom: "uusd".to_string(),
            glow_token: "glow".to_string(),
            lotto_contract: "lotto".to_string(),
            gov_contract: "gov".to_string(),
            terraswap_factory: "terraswap".to_string(),
            spend_limit: Uint128::from(1000000u128),
        }
    );

    let msg = ExecuteMsg::UpdateConfig {
        spend_limit: None,
        owner: Some("other".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);

    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn test_spend() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        stable_denom: "uusd".to_string(),
        glow_token: "glow".to_string(),
        lotto_contract: "lotto".to_string(),
        gov_contract: "gov".to_string(),
        terraswap_factory: "terraswap".to_string(),
        spend_limit: Uint128::from(1000000u128),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // permission failed
    let msg = ExecuteMsg::Spend {
        recipient: "addr0000".to_string(),
        amount: Uint128::from(1000000u128),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // failed due to spend limit
    let msg = ExecuteMsg::Spend {
        recipient: "addr0000".to_string(),
        amount: Uint128::from(2000000u128),
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(StdError::GenericErr { msg, .. }) => {
            assert_eq!(msg, "Cannot spend more than spend_limit")
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = ExecuteMsg::Spend {
        recipient: "addr0000".to_string(),
        amount: Uint128::from(1000000u128),
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "glow".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0000".to_string(),
                amount: Uint128::from(1000000u128),
            })
            .unwrap(),
        }))]
    );
}

#[test]
fn test_transfer_stable() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(10000000u128),
    }]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        stable_denom: "uusd".to_string(),
        glow_token: "glow".to_string(),
        lotto_contract: "lotto".to_string(),
        gov_contract: "gov".to_string(),
        terraswap_factory: "terraswap".to_string(),
        spend_limit: Uint128::from(1000000u128),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // permission failed
    let msg = ExecuteMsg::TransferStable {
        recipient: "addr0000".to_string(),
        amount: Uint128::from(1000000u128),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // failed due to spend limit
    let msg = ExecuteMsg::TransferStable {
        recipient: "addr0000".to_string(),
        amount: Uint128::from(2000000u128),
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(StdError::GenericErr { msg, .. }) => {
            assert_eq!(msg, "Cannot spend more than spend_limit")
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = ExecuteMsg::TransferStable {
        recipient: "addr0000".to_string(),
        amount: Uint128::from(1000000u128),
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "addr0000".to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(1000000u128),
            }],
        }))]
    );
}

#[test]
fn test_sponsor_lotto() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        stable_denom: "uusd".to_string(),
        glow_token: "glow".to_string(),
        lotto_contract: "lotto".to_string(),
        gov_contract: "gov".to_string(),
        terraswap_factory: "terraswap".to_string(),
        spend_limit: Uint128::from(1000000u128),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // permission failed
    let msg = ExecuteMsg::SponsorLotto {
        amount: Uint128::from(1000000u128),
        award: None,
        prize_distribution: None,
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "lotto".to_string(),
            funds: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(1000000u128)
            }],
            msg: to_binary(&LottoMsg::Sponsor {
                award: None,
                prize_distribution: None
            })
            .unwrap(),
        }))]
    );
}

#[test]
fn test_withdraw_lotto() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        stable_denom: "uusd".to_string(),
        glow_token: "glow".to_string(),
        lotto_contract: "lotto".to_string(),
        gov_contract: "gov".to_string(),
        terraswap_factory: "terraswap".to_string(),
        spend_limit: Uint128::from(1000000u128),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // permission failed
    let msg = ExecuteMsg::WithdrawSponsor {};

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "lotto".to_string(),
            funds: vec![],
            msg: to_binary(&LottoMsg::SponsorWithdraw {}).unwrap(),
        }))]
    );
}

#[test]
fn test_swap() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(100u128),
    }]);

    deps.querier.with_tax(
        Decimal::zero(),
        &[(&"uusd".to_string(), &Uint128::from(1000000u128))],
    );

    deps.querier
        .with_terraswap_pairs(&[(&"uusdglow".to_string(), &"pairglow".to_string())]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        stable_denom: "uusd".to_string(),
        glow_token: "glow".to_string(),
        lotto_contract: "lotto".to_string(),
        gov_contract: "gov".to_string(),
        terraswap_factory: "terraswap".to_string(),
        spend_limit: Uint128::from(1000000u128),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // permission failed
    let msg = ExecuteMsg::Swap {
        amount: Uint128::from(50u128),
    };
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg {
            id: 0,
            msg: WasmMsg::Execute {
                contract_addr: "pairglow".to_string(),
                msg: to_binary(&TerraswapExecuteMsg::Swap {
                    offer_asset: Asset {
                        info: AssetInfo::NativeToken {
                            denom: "uusd".to_string()
                        },
                        amount: Uint128::from(50u128),
                    },
                    max_spread: None,
                    belief_price: None,
                    to: None,
                })
                .unwrap(),
                funds: vec![Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(50u128),
                }],
            }
            .into(),
            gas_limit: None,
            reply_on: ReplyOn::Never
        }]
    );
}

#[test]
fn test_burn() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        stable_denom: "uusd".to_string(),
        glow_token: "glow".to_string(),
        lotto_contract: "lotto".to_string(),
        gov_contract: "gov".to_string(),
        terraswap_factory: "terraswap".to_string(),
        spend_limit: Uint128::from(1000000u128),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // permission failed
    let msg = ExecuteMsg::Spend {
        recipient: "addr0000".to_string(),
        amount: Uint128::from(1000000u128),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // failed due to spend limit
    let msg = ExecuteMsg::Spend {
        recipient: "addr0000".to_string(),
        amount: Uint128::from(2000000u128),
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(StdError::GenericErr { msg, .. }) => {
            assert_eq!(msg, "Cannot spend more than spend_limit")
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = ExecuteMsg::Burn {
        amount: Uint128::from(1000000u128),
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "glow".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Burn {
                amount: Uint128::from(1000000u128),
            })
            .unwrap(),
        }))]
    );
}
