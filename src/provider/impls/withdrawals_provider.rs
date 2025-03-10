use crate::primitives::AlloyRethNodePrimitives;
use crate::{AlloyNetwork, AlloyRethProvider};
use alloy_eips::eip4895::Withdrawals;
use alloy_eips::BlockHashOrNumber;
use alloy_provider::Provider;
use reth_errors::ProviderResult;
use reth_provider::WithdrawalsProvider;

impl<P, NP> WithdrawalsProvider for AlloyRethProvider<P, NP>
where
    P: 'static + Clone + Provider<AlloyNetwork> + Send + Sync,
    NP: AlloyRethNodePrimitives,
{
    fn withdrawals_by_block(&self, _id: BlockHashOrNumber, _timestamp: u64) -> ProviderResult<Option<Withdrawals>> {
        todo!()
    }
}
