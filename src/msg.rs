use std::collections::BTreeMap;

use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Start {},
    Claim {},
    Upgrade {},

    // admin
    RemovePlayer { address: String }, // EditUpgrade
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Player)]
    Info { address: String },

    #[returns(ContractInformationResponse)]
    ContractInfo {},
}

pub enum AssetTypes {
    Crops,
    Animals,
    Workers,
}
impl AssetTypes {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetTypes::Crops => "crops",
            AssetTypes::Animals => "animals",
            AssetTypes::Workers => "workers",
        }
    }
}

// === Messages / Structures for State ====
#[cw_serde]
pub struct Asset {
    pub amount: u64, // 0 or 1 initially

    pub growth_rate: u64,
    pub growth_rate_inc: u64, // growth_rate / growth_rate_inc = increase. (10_000/100 = 100 + 10_000 = 10_100. so 1% faster production)

    pub cost: u64,
    pub cost_inc: u64, // (1_000_000/5 = 200_000 + 1_000_000 = 1_200_000. so 20% more expensive to upgrade)
}

#[cw_serde]
pub struct Player {
    pub start_time: u64,

    pub last_claim_height: u64,
    pub current_points: u64,

    // name: asset
    pub upgrades: BTreeMap<String, Asset>,
}

// === RESPONSES ===
#[cw_serde]
pub struct ContractInformationResponse {
    pub admin: String,
}
