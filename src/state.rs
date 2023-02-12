use cosmwasm_std::Deps;
use cw_storage_plus::{Item, Map};

use crate::msg::{Asset, ContractInformationResponse, Player};

pub const INFORMATION: Item<ContractInformationResponse> = Item::new("info");

// Addr, Information about player state
pub const PLAYERS: Map<&str, Player> = Map::new("players");

pub const INITIAL_UPGRADES: Map<&str, Asset> = Map::new("upgrades");

pub fn get_last_claim_height(deps: Deps, address: &str) -> u64 {
    let player = PLAYERS.may_load(deps.storage, address);
    match player {
        Ok(Some(player)) => player.last_claim_height,
        _ => 0,
    }
}
