use crate::AlloyRethProvider;
use alloy_network::primitives::BlockTransactionsKind;
use alloy_network::{BlockResponse, Network};
use alloy_primitives::{BlockHash, BlockNumber, U256};
use alloy_provider::Provider;
use reth_errors::{ProviderError, ProviderResult};
use reth_primitives::SealedHeader;
use reth_provider::errors::any::AnyError;
pub(crate) use reth_provider::HeaderProvider;
use std::future::IntoFuture;
use std::ops::RangeBounds;
use tokio::runtime::Handle;

impl<N, P> HeaderProvider for AlloyRethProvider<N, P>
where
    N: Network<HeaderResponse = alloy_rpc_types_eth::Header>,
    P: 'static + Clone + Provider<N> + Send + Sync,
{
    type Header = alloy_consensus::Header;

    fn header(&self, block_hash: &BlockHash) -> ProviderResult<Option<Self::Header>> {
        let block = tokio::task::block_in_place(move || {
            Handle::current().block_on(self.provider.get_block_by_hash(*block_hash).kind(BlockTransactionsKind::Hashes).into_future())
        });
        match block {
            Ok(Some(block)) => Ok(Some(block.header().clone().into())),
            Ok(None) => Err(ProviderError::BlockHashNotFound(*block_hash)),
            Err(e) => Err(ProviderError::Other(AnyError::new(e))),
        }
    }

    fn header_by_number(&self, _num: u64) -> ProviderResult<Option<Self::Header>> {
        todo!()
    }

    fn header_td(&self, _hash: &BlockHash) -> ProviderResult<Option<U256>> {
        todo!()
    }

    fn header_td_by_number(&self, _number: BlockNumber) -> ProviderResult<Option<U256>> {
        todo!()
    }

    fn headers_range(&self, _range: impl RangeBounds<BlockNumber>) -> ProviderResult<Vec<Self::Header>> {
        todo!()
    }

    fn sealed_header(&self, _number: BlockNumber) -> ProviderResult<Option<SealedHeader<Self::Header>>> {
        todo!()
    }

    fn sealed_headers_while(
        &self,
        _range: impl RangeBounds<BlockNumber>,
        _predicate: impl FnMut(&SealedHeader<Self::Header>) -> bool,
    ) -> ProviderResult<Vec<SealedHeader<Self::Header>>> {
        todo!()
    }
}
