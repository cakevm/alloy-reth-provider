use crate::primitives::AlloyRethNodePrimitives;
use crate::{AlloyNetwork, AlloyRethProvider};
use alloy_consensus::BlockHeader;
use alloy_eips::{BlockId, BlockNumHash, BlockNumberOrTag};
use alloy_network::primitives::{BlockTransactionsKind, HeaderResponse};
use alloy_network::BlockResponse;
use alloy_primitives::{BlockNumber, B256};
use alloy_provider::Provider;
use reth_errors::{ProviderError, ProviderResult};
use reth_provider::errors::any::AnyError;
use reth_provider::{BlockIdReader, BlockNumReader};
use std::fmt::Debug;
use std::future::IntoFuture;
use tokio::runtime::Handle;

impl<P, NP> BlockNumReader for AlloyRethProvider<P, NP>
where
    P: Provider<AlloyNetwork> + Send + Sync + Debug + Clone + 'static,
    NP: AlloyRethNodePrimitives,
{
    fn chain_info(&self) -> ProviderResult<reth_chainspec::ChainInfo> {
        let block = tokio::task::block_in_place(move || {
            Handle::current().block_on(self.provider.get_block(BlockId::latest()).kind(BlockTransactionsKind::Hashes).into_future())
        });
        match block {
            Ok(Some(block)) => Ok(reth_chainspec::ChainInfo { best_hash: block.header().hash(), best_number: block.header().number() }),
            Ok(None) => Err(ProviderError::BestBlockNotFound),
            Err(e) => Err(ProviderError::Other(AnyError::new(e))),
        }
    }

    fn best_block_number(&self) -> ProviderResult<BlockNumber> {
        self.last_block_number()
    }

    fn last_block_number(&self) -> ProviderResult<BlockNumber> {
        let block_number = tokio::task::block_in_place(move || Handle::current().block_on(self.provider.get_block_number()));
        match block_number {
            Ok(block_number) => Ok(block_number),
            Err(e) => Err(ProviderError::Other(AnyError::new(e))),
        }
    }

    fn block_number(&self, hash: B256) -> ProviderResult<Option<BlockNumber>> {
        let block = tokio::task::block_in_place(move || {
            Handle::current().block_on(self.provider.get_block_by_hash(hash).kind(BlockTransactionsKind::Hashes).into_future())
        });
        match block {
            Ok(Some(block)) => Ok(Some(block.header().number())),
            Ok(None) => Err(ProviderError::BlockHashNotFound(hash)),
            Err(e) => Err(ProviderError::Other(AnyError::new(e))),
        }
    }
}

impl<P, NP> BlockIdReader for AlloyRethProvider<P, NP>
where
    P: Provider<AlloyNetwork> + Send + Sync + Debug + Clone + 'static,
    NP: AlloyRethNodePrimitives,
{
    fn pending_block_num_hash(&self) -> ProviderResult<Option<alloy_eips::BlockNumHash>> {
        todo!()
    }

    fn safe_block_num_hash(&self) -> ProviderResult<Option<alloy_eips::BlockNumHash>> {
        todo!()
    }

    fn finalized_block_num_hash(&self) -> ProviderResult<Option<alloy_eips::BlockNumHash>> {
        let block = tokio::task::block_in_place(move || {
            Handle::current().block_on(self.provider.get_block_by_number(BlockNumberOrTag::Finalized).into_future())
        });
        match block {
            Ok(Some(block)) => {
                let number = block.header().number();
                let hash = B256::from(*block.header().hash());
                Ok(Some(BlockNumHash { number, hash }))
            }
            Ok(None) => Err(ProviderError::FinalizedBlockNotFound),
            Err(e) => Err(ProviderError::Other(AnyError::new(e))),
        }
    }
}

#[cfg(not(feature = "optimism"))]
#[cfg(test)]
mod tests {
    use crate::AlloyRethProvider;
    use alloy_node_bindings::Anvil;
    use alloy_primitives::b256;
    use alloy_provider::ProviderBuilder;
    use reth_ethereum_primitives::EthPrimitives;
    use reth_provider::BlockNumReader;
    use std::env;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_chain_info() {
        let node_url = env::var("MAINNET_HTTP").unwrap_or("https://eth.merkle.io".to_string());
        let anvil = Anvil::new().fork(node_url).fork_block_number(16148323).spawn();
        let provider = ProviderBuilder::new().connect_http(anvil.endpoint_url());

        let db_provider = AlloyRethProvider::new(provider, EthPrimitives::default());
        let chain_info = db_provider.chain_info().unwrap();
        assert_eq!(chain_info.best_number, 16148323);
        assert_eq!(chain_info.best_hash, b256!("0xc133a5a4ceef2a6b5cd6fc682e49ca0f8fce3f18da85098c6a15f8e0f6f4c2cf"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_best_block_number() {
        let node_url = env::var("MAINNET_HTTP").unwrap_or("https://eth.merkle.io".to_string());
        let anvil = Anvil::new().fork(node_url).fork_block_number(16148323).spawn();
        let provider = ProviderBuilder::new().connect_http(anvil.endpoint_url());

        let db_provider = AlloyRethProvider::new(provider, EthPrimitives::default());
        let block_number = db_provider.best_block_number().unwrap();
        assert_eq!(block_number, 16148323);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_last_number() {
        let node_url = env::var("MAINNET_HTTP").unwrap_or("https://eth.merkle.io".to_string());
        let anvil = Anvil::new().fork(node_url).fork_block_number(16148323).spawn();
        let provider = ProviderBuilder::new().connect_http(anvil.endpoint_url());

        let db_provider = AlloyRethProvider::new(provider, EthPrimitives::default());
        let block_number = db_provider.last_block_number().unwrap();
        assert_eq!(block_number, 16148323);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_block_number() {
        let node_url = env::var("MAINNET_HTTP").unwrap_or("https://eth.merkle.io".to_string());
        let provider = ProviderBuilder::new().connect_http(node_url.parse().unwrap());

        let db_provider = AlloyRethProvider::new(provider, EthPrimitives::default());
        let block_number =
            db_provider.block_number(b256!("0xc133a5a4ceef2a6b5cd6fc682e49ca0f8fce3f18da85098c6a15f8e0f6f4c2cf")).unwrap().unwrap();
        assert_eq!(block_number, 16148323);
    }
}
