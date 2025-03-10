use crate::primitives::AlloyRethNodePrimitives;
use crate::{AlloyNetwork, AlloyRethProvider};
use alloy_provider::Provider;
use reth_errors::ProviderResult;
use reth_provider::BlockIdReader;

impl<P, NP> BlockIdReader for AlloyRethProvider<P, NP>
where
    P: Provider<AlloyNetwork> + Send + Sync + Clone + 'static,
    NP: AlloyRethNodePrimitives,
{
    fn pending_block_num_hash(&self) -> ProviderResult<Option<alloy_eips::BlockNumHash>> {
        todo!()
    }

    fn safe_block_num_hash(&self) -> ProviderResult<Option<alloy_eips::BlockNumHash>> {
        todo!()
    }

    fn finalized_block_num_hash(&self) -> ProviderResult<Option<alloy_eips::BlockNumHash>> {
        todo!()
    }
}
