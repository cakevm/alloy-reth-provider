use crate::AlloyRethStateProvider;
use alloy_network::Network;
use alloy_primitives::{BlockNumber, B256};
use alloy_provider::Provider;
use reth_errors::{ProviderError, ProviderResult};
use reth_provider::errors::any::AnyError;
use reth_provider::BlockHashReader;
use revm_database::DatabaseRef;

impl<N, P> BlockHashReader for AlloyRethStateProvider<N, P>
where
    N: Network,
    P: Provider<N> + Clone,
{
    fn block_hash(&self, number: BlockNumber) -> ProviderResult<Option<B256>> {
        match self.alloy_db.block_hash_ref(number) {
            Ok(value) => Ok(Some(value)),
            Err(e) => Err(ProviderError::Other(AnyError::new(e))),
        }
    }

    fn canonical_hashes_range(&self, _start: BlockNumber, _end: BlockNumber) -> ProviderResult<Vec<B256>> {
        todo!()
    }
}
