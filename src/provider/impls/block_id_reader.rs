use crate::primitives::AlloyRethNodePrimitives;
use crate::{AlloyNetwork, AlloyRethProvider};
use alloy_consensus::BlockHeader;
use alloy_eips::{BlockNumHash, BlockNumberOrTag};
use alloy_network::primitives::HeaderResponse;
use alloy_network::BlockResponse;
use alloy_primitives::B256;
use alloy_provider::Provider;
use reth_errors::{ProviderError, ProviderResult};
use reth_provider::errors::any::AnyError;
use reth_provider::BlockIdReader;
use std::future::IntoFuture;
use tokio::runtime::Handle;

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
