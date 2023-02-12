use std::collections::BTreeMap;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    Asset, AssetTypes, ContractInformationResponse, ExecuteMsg, InstantiateMsg, Player, QueryMsg, PointsPerBlock,
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
            unlock_cost: 0, // already unlocked
            level: 1,

            growth_rate: 10_000,  // 10_000upoints per block
            growth_rate_inc: 75, // (growth_rate/growth_rate_inc)+growth_rate = 10_100 points for next upgrade

            cost: 1_000_000,
            cost_inc: 10, // (cost/cost_inc)+cost = 1_100_000upoints for next upgrade
        },
    )?;
    INITIAL_UPGRADES.save(
        deps.storage,
        AssetTypes::Animals.as_str(),
        &Asset {
            amount: 0,
            unlock_cost: 25_000_000,
            level: 1,

            growth_rate: 100_000,
            growth_rate_inc: 40,
            cost: 10_000_000,
            cost_inc: 5,
        },
    )?;
    INITIAL_UPGRADES.save(
        deps.storage,
        AssetTypes::Workers.as_str(),
        &Asset {
            amount: 0,
            unlock_cost: 100_000_000,
            level: 1,

            growth_rate: 1_000_000,
            growth_rate_inc: 20,
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

fn calculate_per_block_rewards(player: &Player) -> PointsPerBlock {
    let mut ppb = PointsPerBlock {
        total: 0,
        per_asset: BTreeMap::new(),
    };

    for (asset_type, asset) in player.upgrades.iter() {
        let per_asset = asset.amount * asset.growth_rate;
        ppb.total += per_asset;
        ppb.per_asset.insert(asset_type.to_string(), per_asset);
    }

    ppb
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

        ExecuteMsg::Claim {} => {
            let sender = info.sender.to_string();

            let player = PLAYERS.may_load(deps.storage, sender.as_str())?;
            match player {
                Some(player) => {
                    let mut player = player;

                    let total_points = calculate_per_block_rewards(&player).total;

                    let blocks_since_last_claim = env.block.height - player.last_claim_height;
                    let points_since_last_claim = total_points * blocks_since_last_claim as u128;

                    player.current_points += points_since_last_claim;
                    player.last_claim_height = env.block.height;

                    PLAYERS
                        .save(deps.storage, sender.as_str(), &player)
                        .unwrap();
                }
                None => return Err(ContractError::PlayerDoesNotExist { address: sender }),
            }

            // return points claimed as well?
            Ok(Response::new().add_attribute("action", "claim"))
        }

        ExecuteMsg::Upgrade { name, num_of_times } => {
            let player = PLAYERS.may_load(deps.storage, info.sender.as_str())?;

            match player {
                Some(player) => {
                    let mut player = player;

                    let mut asset = match player.upgrades.get(&name) {
                        Some(asset) => asset.to_owned(),
                        None => return Err(ContractError::AssetDoesNotExist { name }),
                    };
                    
                    if asset.amount == 0 {
                        return Err(ContractError::AssetNotPurchased { name });
                    }

                    // iterate through as multiple tiems needs to apply the %s correctly
                    let mut total_cost = 0;
                    let mut max_purchase_amount_possible = 0;
                    let num_of_times = num_of_times.unwrap_or_else(|| 1);
                    for _ in 0..num_of_times {
                        total_cost += asset.cost;
                        // asset.amount += 1; // they do this with a different type of upgrade?
                        asset.growth_rate =
                            (asset.growth_rate / asset.growth_rate_inc) + asset.growth_rate;
                        asset.cost = (asset.cost / asset.cost_inc) + asset.cost;

                        // keep running track of the max number of times we could buy it if we do not have enough
                        max_purchase_amount_possible += 1;
                        if player.current_points < total_cost {
                            break;
                        }
                    }

                    if player.current_points < total_cost {
                        return Err(ContractError::NotEnoughPoints {
                            received: player.current_points,
                            required: total_cost,
                            max_amount: Some(max_purchase_amount_possible - 1),
                        });
                    }

                    player.current_points -= total_cost;
                    asset.level += num_of_times as u128;
                    player.upgrades.insert(name, asset);

                    PLAYERS
                        .save(deps.storage, info.sender.as_str(), &player)
                        .unwrap();
                }
                None => {
                    return Err(ContractError::PlayerDoesNotExist {
                        address: info.sender.to_string(),
                    })
                }
            }

            Ok(Response::new().add_attribute("action", "upgrade"))
        }

        ExecuteMsg::Unlock { name } => {
            let player = PLAYERS.may_load(deps.storage, info.sender.as_str())?;

            match player {
                Some(player) => {
                    let mut player = player;

                    let mut asset = match player.upgrades.get(&name) {
                        Some(asset) => asset.to_owned(),
                        None => return Err(ContractError::AssetDoesNotExist { name }),
                    };
                    
                    // for now you can only buy 1 of said asset. maybe allow more in the future? (likely at a higher cost)
                    if asset.amount == 1 {
                        return Err(ContractError::AssetAlreadyPurchased { name });
                    }
                    
                    let total_cost = asset.unlock_cost;
                    if player.current_points < total_cost {
                        return Err(ContractError::NotEnoughPoints {
                            received: player.current_points,
                            required: total_cost,
                            max_amount: None,
                        });
                    }

                    player.current_points -= total_cost;
                    asset.amount = 1;
                    player.upgrades.insert(name, asset);

                    PLAYERS
                        .save(deps.storage, info.sender.as_str(), &player)
                        .unwrap();
                }
                None => {
                    return Err(ContractError::PlayerDoesNotExist {
                        address: info.sender.to_string(),
                    })
                }
            }

            Ok(Response::new().add_attribute("action", "upgrade"))
        }

        // ADMIN MESSAGES
        ExecuteMsg::RemovePlayer { address } => {
            admin_error_check(deps.as_ref(), info)?;
            deps.api.addr_validate(&address)?;

            PLAYERS.remove(deps.storage, address.as_str());

            Ok(Response::new().add_attribute("action", "remove_player"))
        }
        
        ExecuteMsg::AddFunds { address, amount } => {
            admin_error_check(deps.as_ref(), info)?;
            deps.api.addr_validate(&address)?;

            let player = PLAYERS.may_load(deps.storage, address.as_str())?;
            match player {
                Some(player) => {
                    let mut player = player;
                    player.current_points += amount;

                    PLAYERS
                        .save(deps.storage, address.as_str(), &player)
                        .unwrap();
                }
                None => return Err(ContractError::PlayerDoesNotExist { address }),
            }

            Ok(Response::new().add_attribute("action", "add_funds"))
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

        QueryMsg::PointsPerBlock { address } => {
            let player = PLAYERS.may_load(deps.storage, address.as_str())?;
            let player = match player {
                Some(player) => player,
                None => return Ok(to_binary(&0)?),
            };

            let points_per_block = calculate_per_block_rewards(&player);
            Ok(to_binary(&points_per_block)?)
        }

        QueryMsg::Upgrades {} => {
            let upgrades =
                INITIAL_UPGRADES.keys(deps.storage, None, None, cosmwasm_std::Order::Ascending);

            let upgrades: Vec<String> = upgrades.map(|upgrade| upgrade.unwrap()).collect();

            let v = to_binary(&upgrades)?;
            Ok(v)
        }
    }
}

#[cfg(test)]
mod tests {}
