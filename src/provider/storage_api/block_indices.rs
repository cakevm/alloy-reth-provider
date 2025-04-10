use crate::primitives::AlloyRethNodePrimitives;
use crate::{AlloyNetwork, AlloyRethProvider};
use alloy_primitives::BlockNumber;
use alloy_provider::Provider;
use reth_db_models::StoredBlockBodyIndices;
use reth_errors::ProviderResult;
use reth_provider::BlockBodyIndicesProvider;
use std::ops::RangeInclusive;

impl<P, NP> BlockBodyIndicesProvider for AlloyRethProvider<P, NP>
where
    P: 'static + Clone + Provider<AlloyNetwork> + Send + Sync,
    NP: AlloyRethNodePrimitives,
{
    fn block_body_indices(&self, _num: u64) -> ProviderResult<Option<StoredBlockBodyIndices>> {
        todo!()
    }

    fn block_body_indices_range(&self, _range: RangeInclusive<BlockNumber>) -> ProviderResult<Vec<StoredBlockBodyIndices>> {
        todo!()
    }
}
