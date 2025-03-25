use crate::primitives::AlloyRethNodePrimitives;
use crate::{AlloyNetwork, AlloyRethProvider};
use alloy_eips::BlockId;
use alloy_provider::Provider;
use reth_errors::ProviderResult;
use reth_primitives_traits::SealedHeader;
use reth_provider::BlockReaderIdExt;

impl<P, NP> BlockReaderIdExt for AlloyRethProvider<P, NP>
where
    P: Provider<AlloyNetwork> + Send + Sync + Clone + 'static,
    NP: AlloyRethNodePrimitives,
{
    fn block_by_id(&self, _id: BlockId) -> ProviderResult<Option<Self::Block>> {
        todo!()
    }

    fn sealed_header_by_id(&self, _id: BlockId) -> ProviderResult<Option<SealedHeader<Self::Header>>> {
        todo!()
    }

    fn header_by_id(&self, _id: BlockId) -> ProviderResult<Option<Self::Header>> {
        todo!()
    }

    fn ommers_by_id(&self, _id: BlockId) -> ProviderResult<Option<Vec<Self::Header>>> {
        todo!()
    }
}
