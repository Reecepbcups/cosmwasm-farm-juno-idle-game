use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
    #[error("Player account already exist: {address}")]
    PlayerAlreadyExists { address: String },

    #[error("Player does not exist: {address}")]
    PlayerDoesNotExist { address: String },

    #[error("That asset does not exist: {name}")]
    AssetDoesNotExist { name: String },

    #[error("You have not yet purchased this asset: {name}")]
    AssetNotPurchased { name: String },

    #[error("You have already bought this asset: {name}")]
    AssetAlreadyPurchased { name: String },

    #[error("You do not have enough points: balance={received:?},required={required:?}. MaxYouCanBuy={max_amount:?}")]
    NotEnoughPoints {
        received: u128,
        required: u128,
        max_amount: Option<u64>,
    },
}
