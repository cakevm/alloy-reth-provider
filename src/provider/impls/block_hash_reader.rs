use crate::AlloyRethProvider;
use alloy_network::primitives::{BlockTransactionsKind, HeaderResponse};
use alloy_network::{BlockResponse, Network};
use alloy_primitives::{BlockNumber, B256};
use alloy_provider::Provider;
use reth_errors::{ProviderError, ProviderResult};
use reth_provider::errors::any::AnyError;
pub(crate) use reth_provider::BlockHashReader;
use tokio::runtime::Handle;

impl<N, P> BlockHashReader for AlloyRethProvider<N, P>
where
    N: Network,
    P: Provider<N> + Send + Sync + Clone + 'static,
{
    fn block_hash(&self, number: BlockNumber) -> ProviderResult<Option<B256>> {
        let block = tokio::task::block_in_place(move || {
            Handle::current().block_on(self.provider.get_block_by_number(number.into(), BlockTransactionsKind::Hashes))
        });
        match block {
            Ok(Some(block)) => Ok(Some(B256::from(*block.header().hash()))),
            Ok(None) => Err(ProviderError::BlockBodyIndicesNotFound(number)),
            Err(e) => Err(ProviderError::Other(AnyError::new(e))),
        }
    }

    fn canonical_hashes_range(&self, start: BlockNumber, end: BlockNumber) -> ProviderResult<Vec<B256>> {
        let mut hashes = Vec::with_capacity((end - start) as usize);
        for i in start..=end {
            let block = tokio::task::block_in_place(move || {
                Handle::current().block_on(self.provider.get_block_by_number(i.into(), BlockTransactionsKind::Hashes))
            });
            match block {
                Ok(Some(block)) => hashes.push(B256::from(*block.header().hash())),
                Ok(None) => return Err(ProviderError::BlockBodyIndicesNotFound(i)),
                Err(e) => return Err(ProviderError::Other(AnyError::new(e))),
            }
        }
        Ok(hashes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::b256;
    use alloy_provider::ProviderBuilder;
    use std::env;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_block_hash() {
        let node_url = env::var("MAINNET_HTTP").unwrap_or("https://eth.merkle.io".to_string());
        let provider = ProviderBuilder::new().on_http(node_url.parse().unwrap());

        let db_provider = AlloyRethProvider::new(provider);
        let block_hash = db_provider.block_hash(16148323).unwrap().unwrap();

        assert_eq!(block_hash, b256!("0xc133a5a4ceef2a6b5cd6fc682e49ca0f8fce3f18da85098c6a15f8e0f6f4c2cf"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_block_hash_not_found() {
        let node_url = env::var("MAINNET_HTTP").unwrap_or("https://eth.merkle.io".to_string());
        let provider = ProviderBuilder::new().on_http(node_url.parse().unwrap());

        let db_provider = AlloyRethProvider::new(provider);
        let provider_result = db_provider.block_hash(99999999);

        assert!(provider_result.is_err());
        assert!(matches!(provider_result.unwrap_err(), ProviderError::BlockBodyIndicesNotFound(99999999)));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_canonical_hashes_range() {
        let node_url = env::var("MAINNET_HTTP").unwrap_or("https://eth.merkle.io".to_string());
        let provider = ProviderBuilder::new().on_http(node_url.parse().unwrap());

        let db_provider = AlloyRethProvider::new(provider);
        let block_hashes = db_provider.canonical_hashes_range(16148323, 16148324).unwrap();

        assert_eq!(block_hashes.len(), 2);
        assert_eq!(block_hashes[0], b256!("0xc133a5a4ceef2a6b5cd6fc682e49ca0f8fce3f18da85098c6a15f8e0f6f4c2cf"));
        assert_eq!(block_hashes[1], b256!("0xe088a420cd40b420149f525ce3941be3ddae4d1781ec1bf61969ded90365a3ea"));
    }
}
