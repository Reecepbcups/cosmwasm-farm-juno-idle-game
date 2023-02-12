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
    
    // Buys a new asset to produce income
    Unlock {
        name: String,        
    },

    // Buy more of a given asset
    Upgrade {
        name: String,  
        num_of_times: Option<u64>,      
    },

    // admin
    RemovePlayer {
        address: String,
    }, // EditUpgrade
    AddFunds {
        address: String,
        amount: u128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Player)]
    Info { address: String },

    #[returns(ContractInformationResponse)]
    ContractInfo {},

    #[returns(Upgrades)]
    Upgrades {},

    #[returns(PointsPerBlock)]
    PointsPerBlock { address: String },
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
    pub amount: u128, // 0 or 1 initially
    
    pub level: u128, // every buy we increase this +X amount of buys

    pub unlock_cost: u128,

    pub growth_rate: u128,
    pub growth_rate_inc: u128, // growth_rate / growth_rate_inc = increase. (10_000/100 = 100 + 10_000 = 10_100. so 1% faster production)

    pub cost: u128,
    pub cost_inc: u128, // (1_000_000/5 = 200_000 + 1_000_000 = 1_200_000. so 20% more expensive to upgrade)
}

#[cw_serde]
pub struct PointsPerBlock {
    pub total: u128,
    pub per_asset: BTreeMap<String, u128>,
}

#[cw_serde]
pub struct Upgrades {
    pub values: Vec<String>,
}

#[cw_serde]
pub struct Player {
    pub start_time: u64,
    
    pub current_points: u128,
    pub last_claim_height: u64,

    // name: asset
    pub upgrades: BTreeMap<String, Asset>,
}

// === RESPONSES ===
#[cw_serde]
pub struct ContractInformationResponse {
    pub admin: String,
}
