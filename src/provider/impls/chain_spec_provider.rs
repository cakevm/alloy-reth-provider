use crate::primitives::AlloyRethNodePrimitives;
use crate::{AlloyNetwork, AlloyRethProvider};
use alloy_provider::Provider;
use reth_chainspec::{ChainSpec, ChainSpecProvider, HOLESKY, MAINNET, SEPOLIA};
#[cfg(feature = "optimism")]
use reth_optimism_chainspec::{BASE_MAINNET, BASE_SEPOLIA, OP_MAINNET, OP_SEPOLIA};
use std::sync::Arc;
use tokio::runtime::Handle;

impl<P, NP> ChainSpecProvider for AlloyRethProvider<P, NP>
where
    P: Provider<AlloyNetwork> + Send + Sync + Clone + 'static,
    NP: AlloyRethNodePrimitives,
{
    type ChainSpec = ChainSpec;

    fn chain_spec(&self) -> Arc<ChainSpec> {
        let chain_id = tokio::task::block_in_place(move || Handle::current().block_on(self.provider.get_chain_id())).unwrap_or(1);
        match chain_id {
            // Ethereum
            1 => MAINNET.clone(),
            17000 => HOLESKY.clone(),
            11155111 => SEPOLIA.clone(),
            // Optimism
            #[cfg(feature = "optimism")]
            10 => Arc::from(OP_MAINNET.inner.clone()),
            #[cfg(feature = "optimism")]
            11155420 => Arc::from(OP_SEPOLIA.inner.clone()),
            // Base
            #[cfg(feature = "optimism")]
            8453 => Arc::from(BASE_MAINNET.inner.clone()),
            #[cfg(feature = "optimism")]
            84532 => Arc::from(BASE_SEPOLIA.inner.clone()),

            _ => unimplemented!("Unknown chain id: {}", chain_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_provider::ProviderBuilder;
    use reth_chainspec::EthChainSpec;
    use reth_provider::ChainSpecProvider;
    use std::env;

    #[cfg(not(feature = "optimism"))]
    use reth_ethereum_primitives::EthPrimitives;

    #[cfg(feature = "optimism")]
    use op_alloy_network::Optimism;
    #[cfg(feature = "optimism")]
    use reth_optimism_primitives::OpPrimitives;

    #[cfg(not(feature = "optimism"))]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_chain_spec_eth() {
        let node_url = env::var("MAINNET_HTTP").unwrap_or("https://eth.merkle.io".to_string());
        let provider = ProviderBuilder::new().on_http(node_url.parse().unwrap());

        let db_provider = AlloyRethProvider::new(provider, EthPrimitives::default());
        let chain_spec = db_provider.chain_spec();

        assert_eq!(chain_spec.chain_id(), 1);
    }

    #[cfg(feature = "optimism")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_chain_spec_base() {
        let node_url = env::var("BASE_HTTP").unwrap_or("https://base.merkle.io".to_string());
        let provider = ProviderBuilder::<_, _, Optimism>::default().on_http(node_url.parse().unwrap());

        let db_provider = AlloyRethProvider::new(provider, OpPrimitives::default());
        let chain_spec = db_provider.chain_spec();

        assert_eq!(chain_spec.chain_id(), 8453);
    }
}
