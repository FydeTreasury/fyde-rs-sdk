use ethers::prelude::{ContractError, Http, MulticallError, Provider};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FydeError {
    #[error("Provider error: {0}")]
    ProviderError(#[from] ethers::providers::ProviderError),
    #[error("Contract error")]
    ContractError(#[from] ContractError<Provider<Http>>),
    #[error("Multicall error")]
    MulticallError(#[from] MulticallError<Provider<Http>>),
}
