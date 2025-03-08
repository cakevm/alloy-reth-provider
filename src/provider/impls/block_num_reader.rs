use crate::AlloyRethProvider;
use alloy_consensus::BlockHeader;
use alloy_eips::BlockId;
use alloy_network::primitives::{BlockTransactionsKind, HeaderResponse};
use alloy_network::{BlockResponse, Network};
use alloy_primitives::{BlockNumber, B256};
use alloy_provider::Provider;
use reth_errors::{ProviderError, ProviderResult};
use reth_provider::errors::any::AnyError;
pub(crate) use reth_provider::BlockNumReader;
use tokio::runtime::Handle;

impl<N, P> BlockNumReader for AlloyRethProvider<N, P>
where
    N: Network,
    P: Provider<N> + Send + Sync + Clone + 'static,
{
    fn chain_info(&self) -> ProviderResult<reth_chainspec::ChainInfo> {
        let block = tokio::task::block_in_place(move || {
            Handle::current().block_on(self.provider.get_block(BlockId::latest(), BlockTransactionsKind::Hashes))
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
            Handle::current().block_on(self.provider.get_block_by_hash(hash, BlockTransactionsKind::Hashes))
        });
        match block {
            Ok(Some(block)) => Ok(Some(block.header().number())),
            Ok(None) => Err(ProviderError::BlockHashNotFound(hash)),
            Err(e) => Err(ProviderError::Other(AnyError::new(e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::AlloyRethProvider;
    use alloy_node_bindings::Anvil;
    use alloy_primitives::b256;
    use alloy_provider::ProviderBuilder;
    use reth_provider::BlockNumReader;
    use std::env;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_chain_info() {
        let node_url = env::var("MAINNET_HTTP").unwrap_or("https://eth.merkle.io".to_string());
        let anvil = Anvil::new().fork(node_url).fork_block_number(16148323).spawn();
        let provider = ProviderBuilder::new().on_http(anvil.endpoint_url());

        let db_provider = AlloyRethProvider::new(provider);
        let chain_info = db_provider.chain_info().unwrap();
        assert_eq!(chain_info.best_number, 16148323);
        assert_eq!(chain_info.best_hash, b256!("0xc133a5a4ceef2a6b5cd6fc682e49ca0f8fce3f18da85098c6a15f8e0f6f4c2cf"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_best_block_number() {
        let node_url = env::var("MAINNET_HTTP").unwrap_or("https://eth.merkle.io".to_string());
        let anvil = Anvil::new().fork(node_url).fork_block_number(16148323).spawn();
        let provider = ProviderBuilder::new().on_http(anvil.endpoint_url());

        let db_provider = AlloyRethProvider::new(provider);
        let block_number = db_provider.best_block_number().unwrap();
        assert_eq!(block_number, 16148323);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_last_number() {
        let node_url = env::var("MAINNET_HTTP").unwrap_or("https://eth.merkle.io".to_string());
        let anvil = Anvil::new().fork(node_url).fork_block_number(16148323).spawn();
        let provider = ProviderBuilder::new().on_http(anvil.endpoint_url());

        let db_provider = AlloyRethProvider::new(provider);
        let block_number = db_provider.last_block_number().unwrap();
        assert_eq!(block_number, 16148323);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_block_number() {
        let node_url = env::var("MAINNET_HTTP").unwrap_or("https://eth.merkle.io".to_string());
        let provider = ProviderBuilder::new().on_http(node_url.parse().unwrap());

        let db_provider = AlloyRethProvider::new(provider);
        let block_number =
            db_provider.block_number(b256!("0xc133a5a4ceef2a6b5cd6fc682e49ca0f8fce3f18da85098c6a15f8e0f6f4c2cf")).unwrap().unwrap();
        assert_eq!(block_number, 16148323);
    }
}
