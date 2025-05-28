use crate::primitives::AlloyRethNodePrimitives;
use crate::utils::rpc_receipt_to_receipt;
use crate::{AlloyNetwork, AlloyRethProvider};
use alloy_eips::BlockHashOrNumber;
use alloy_primitives::{BlockNumber, TxHash, TxNumber};
use alloy_provider::Provider;
use reth_errors::{ProviderError, ProviderResult};
use reth_primitives::Receipt;
use reth_provider::{ReceiptProvider, ReceiptProviderIdExt};
use std::fmt::Debug;
use std::future::IntoFuture;
use std::ops::{RangeBounds, RangeInclusive};
use tokio::runtime::Handle;

impl<P, NP> ReceiptProvider for AlloyRethProvider<P, NP>
where
    P: 'static + Clone + Provider<AlloyNetwork> + Debug + Send + Sync,
    NP: AlloyRethNodePrimitives,
{
    type Receipt = reth_primitives::Receipt;

    fn receipt(&self, _id: TxNumber) -> ProviderResult<Option<Receipt>> {
        todo!()
    }

    fn receipt_by_hash(&self, hash: TxHash) -> ProviderResult<Option<Receipt>> {
        let receipt =
            tokio::task::block_in_place(move || Handle::current().block_on(self.provider.get_transaction_receipt(hash).into_future()));
        match receipt {
            Ok(Some(receipt)) => Ok(Some(rpc_receipt_to_receipt(receipt))),
            Ok(None) => Ok(None),
            Err(e) => Err(ProviderError::Other(reth_provider::errors::any::AnyError::new(e))),
        }
    }

    fn receipts_by_block(&self, block: BlockHashOrNumber) -> ProviderResult<Option<Vec<Receipt>>> {
        let receipts =
            tokio::task::block_in_place(move || Handle::current().block_on(self.provider.get_block_receipts(block.into()).into_future()));
        match receipts {
            Ok(Some(receipts)) => Ok(Some(receipts.into_iter().map(rpc_receipt_to_receipt).collect())),
            Ok(None) => Ok(None),
            Err(e) => Err(ProviderError::Other(reth_provider::errors::any::AnyError::new(e))),
        }
    }

    fn receipts_by_tx_range(&self, _range: impl RangeBounds<TxNumber>) -> ProviderResult<Vec<Receipt>> {
        todo!()
    }

    fn receipts_by_block_range(&self, _block_range: RangeInclusive<BlockNumber>) -> ProviderResult<Vec<Vec<Self::Receipt>>> {
        todo!()
    }
}

impl<P, NP> ReceiptProviderIdExt for AlloyRethProvider<P, NP>
where
    NP: AlloyRethNodePrimitives,
    P: 'static + Clone + Provider<AlloyNetwork> + Debug + Send + Sync,
{
}

#[cfg(not(feature = "optimism"))]
#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::b256;
    use alloy_provider::ProviderBuilder;
    use reth_ethereum_primitives::EthPrimitives;
    use std::env;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_receipt_by_hash() {
        let node_url = env::var("MAINNET_HTTP").unwrap_or("https://eth.merkle.io".to_string());
        let provider = ProviderBuilder::new().connect_http(node_url.parse().unwrap());

        let db_provider = AlloyRethProvider::new(provider, EthPrimitives::default());
        let receipt =
            db_provider.receipt_by_hash(b256!("0x411eaf37ec0c0c93582d74e509a51dcd3538ba8384cc4d5511d3e938784bf6a1")).unwrap().unwrap();

        assert!(receipt.success);
        assert_eq!(receipt.cumulative_gas_used, 16450487);
        assert_eq!(receipt.logs.len(), 1);
    }

    #[test_with::no_env(SKIP_RPC_HEAVY_TESTS)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_receipts_by_block() {
        let node_url = env::var("MAINNET_HTTP").unwrap_or("https://eth.merkle.io".to_string());
        let provider = ProviderBuilder::new().connect_http(node_url.parse().unwrap());

        let db_provider = AlloyRethProvider::new(provider, EthPrimitives::default());
        let receipt = db_provider.receipts_by_block(BlockHashOrNumber::Number(22523913)).unwrap().unwrap();

        assert_eq!(receipt.len(), 4);
        assert_eq!(receipt[0].success, true);
        assert_eq!(receipt[0].cumulative_gas_used, 125794);
        assert_eq!(receipt[0].logs.len(), 4);
    }
}
