use crate::primitives::AlloyRethNodePrimitives;
use crate::{AlloyNetwork, AlloyRethProvider};
use alloy_eips::BlockHashOrNumber;
use alloy_primitives::{TxHash, TxNumber};
use alloy_provider::Provider;
use reth_errors::ProviderResult;
use reth_primitives::Receipt;
use reth_provider::{ReceiptProvider, ReceiptProviderIdExt};
use std::fmt::Debug;
use std::ops::RangeBounds;

impl<P, NP> ReceiptProvider for AlloyRethProvider<P, NP>
where
    P: 'static + Clone + Provider<AlloyNetwork> + Debug + Send + Sync,
    NP: AlloyRethNodePrimitives,
{
    type Receipt = reth_primitives::Receipt;

    fn receipt(&self, _id: TxNumber) -> ProviderResult<Option<Receipt>> {
        todo!()
    }

    fn receipt_by_hash(&self, _hash: TxHash) -> ProviderResult<Option<Receipt>> {
        todo!()
    }

    fn receipts_by_block(&self, _block: BlockHashOrNumber) -> ProviderResult<Option<Vec<Receipt>>> {
        todo!()
    }

    fn receipts_by_tx_range(&self, _range: impl RangeBounds<TxNumber>) -> ProviderResult<Vec<Receipt>> {
        todo!()
    }
}

impl<P, NP> ReceiptProviderIdExt for AlloyRethProvider<P, NP>
where
    NP: AlloyRethNodePrimitives,
    P: 'static + Clone + Provider<AlloyNetwork> + Debug + Send + Sync,
{
}
