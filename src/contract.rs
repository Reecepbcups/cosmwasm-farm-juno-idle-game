use std::collections::BTreeMap;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    Asset, AssetTypes, ContractInformationResponse, ExecuteMsg, InstantiateMsg, Player, QueryMsg,
};

use crate::state::{INFORMATION, INITIAL_UPGRADES, PLAYERS};

const CONTRACT_NAME: &str = "crates.io:idlegame";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = msg.admin.unwrap_or_else(|| info.sender.into_string());
    deps.api.addr_validate(&admin)?;

    // Set initial upgrades
    INITIAL_UPGRADES.save(
        deps.storage,
        AssetTypes::Crops.as_str(),
        &Asset {
            amount: 1,

            growth_rate: 10_000,  // 10_000upoints per block
            growth_rate_inc: 100, // (growth_rate/growth_rate_inc)+growth_rate = 10_100 points for next upgrade

            cost: 1_000_000,
            cost_inc: 10, // (cost/cost_inc)+cost = 1_100_000upoints for next upgrade
        },
    )?;
    INITIAL_UPGRADES.save(
        deps.storage,
        AssetTypes::Animals.as_str(),
        &Asset {
            amount: 0,
            growth_rate: 30_000,
            growth_rate_inc: 90,
            cost: 10_000_000,
            cost_inc: 5,
        },
    )?;
    INITIAL_UPGRADES.save(
        deps.storage,
        AssetTypes::Workers.as_str(),
        &Asset {
            amount: 0,
            growth_rate: 65_000,
            growth_rate_inc: 80,
            cost: 15_000_000,
            cost_inc: 3,
        },
    )?;

    INFORMATION.save(deps.storage, &ContractInformationResponse { admin })?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

fn admin_error_check(deps: Deps, info: MessageInfo) -> Result<(), ContractError> {
    let contract_info = INFORMATION.load(deps.storage)?;
    if contract_info.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Start {} => {
            let sender = info.sender.to_string();

            if PLAYERS.may_load(deps.storage, sender.as_str())?.is_some() {
                return Err(ContractError::PlayerAlreadyExists { address: sender });
            }

            let mut player = Player {
                current_points: 0,
                last_claim_height: env.block.height,
                start_time: env.block.height,
                upgrades: BTreeMap::new(),
            };

            for asset_type in [AssetTypes::Crops, AssetTypes::Animals, AssetTypes::Workers].iter() {
                let upgrades = INITIAL_UPGRADES.load(deps.storage, asset_type.as_str())?;
                player
                    .upgrades
                    .insert(asset_type.as_str().to_string(), upgrades);
            }

            PLAYERS
                .save(deps.storage, sender.as_str(), &player)
                .unwrap();
            Ok(Response::new().add_attribute("action", "start"))
        }

        ExecuteMsg::Claim {} => todo!(),
        ExecuteMsg::Upgrade {} => todo!(),

        // ADMIN MESSAGES
        ExecuteMsg::RemovePlayer { address } => {
            admin_error_check(deps.as_ref(), info)?;
            deps.api.addr_validate(&address)?;

            PLAYERS.remove(deps.storage, address.as_str());

            Ok(Response::new().add_attribute("action", "remove_player"))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Info { address } => {
            let player = PLAYERS.may_load(deps.storage, address.as_str())?;
            let v = to_binary(&player)?;
            Ok(v)
        }

        QueryMsg::ContractInfo {} => {
            let info = INFORMATION.load(deps.storage)?;
            let v = to_binary(&info)?;
            Ok(v)
        }
    }
}

#[cfg(test)]
mod tests {}
