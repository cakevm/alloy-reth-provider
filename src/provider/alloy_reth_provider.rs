use crate::primitives::AlloyRethNodePrimitives;
use crate::AlloyNetwork;
use alloy_provider::Provider;

#[derive(Clone)]
pub struct AlloyRethProvider<P: Send + Sync + Clone + 'static, NP: AlloyRethNodePrimitives> {
    pub(crate) provider: P,
    _np: NP,
}

impl<P, NP> AlloyRethProvider<P, NP>
where
    P: Provider<AlloyNetwork> + Send + Sync + Clone + 'static,
    NP: AlloyRethNodePrimitives,
{
    pub fn new(provider: P, _np: NP) -> Self {
        Self { provider, _np }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_provider::ProviderBuilder;
    use reth_chainspec::{ChainSpecProvider, EthChainSpec};
    use reth_provider::{BlockReader, StateProviderFactory};

    #[cfg(not(feature = "optimism"))]
    use reth_ethereum_primitives::EthPrimitives;

    #[cfg(feature = "optimism")]
    use op_alloy_network::Optimism;
    #[cfg(feature = "optimism")]
    use reth_optimism_primitives::OpPrimitives;

    /// Validate that all traits are implemented for the AlloyRethProvider
    fn test_trait<DBProvider>(db_provider: DBProvider)
    where
        DBProvider: StateProviderFactory + BlockReader + ChainSpecProvider + Clone + Unpin,
    {
        #[cfg(not(feature = "optimism"))]
        assert_eq!(db_provider.chain_spec().chain_id(), 1);
        #[cfg(feature = "optimism")]
        assert_eq!(db_provider.chain_spec().chain_id(), 8453);
    }

    #[cfg(not(feature = "optimism"))]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_alloy_reth_provider_ethereum() {
        let provider = ProviderBuilder::new().on_http("https://eth.merkle.io".parse().unwrap());
        test_trait(AlloyRethProvider::new(provider, EthPrimitives::default()));
    }

    #[cfg(feature = "optimism")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_alloy_reth_provider_any_network() {
        let provider = ProviderBuilder::<_, _, Optimism>::default().on_http("https://base.merkle.io".parse().unwrap());
        test_trait(AlloyRethProvider::new(provider, OpPrimitives::default()));
    }
}
