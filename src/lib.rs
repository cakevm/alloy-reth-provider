pub use provider::alloy_reth_provider::{AlloyRethProvider, AlloyRethProviderConfig, GetStateExecutionOutcome};
pub use state_provider::alloy_reth_state_provider::{AlloyRethStateProvider, AlloyRethStateProviderConfig};

pub mod alloy_db;
pub mod primitives;
mod provider;
mod state_provider;
pub mod utils;

#[cfg(not(feature = "optimism"))]
pub use reth_api::RethApi;

#[cfg(not(feature = "optimism"))]
mod reth_api;

#[cfg(not(feature = "optimism"))]
pub type AlloyNetwork = alloy_network::Ethereum;

#[cfg(feature = "optimism")]
pub type AlloyNetwork = op_alloy_network::Optimism;
