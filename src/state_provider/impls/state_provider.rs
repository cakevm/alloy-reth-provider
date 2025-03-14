use crate::AlloyRethStateProvider;
use alloy_network::Network;
use alloy_primitives::{Address, StorageKey, StorageValue, B256};
use alloy_provider::Provider;
use reth_errors::{ProviderError, ProviderResult};
use reth_primitives::Bytecode;
use reth_provider::errors::any::AnyError;
use reth_provider::StateProvider;
use revm_database::DatabaseRef;

impl<N, P> StateProvider for AlloyRethStateProvider<N, P>
where
    N: Network,
    P: Provider<N> + Clone,
{
    fn storage(&self, account: Address, storage_key: StorageKey) -> ProviderResult<Option<StorageValue>> {
        match self.alloy_db.storage_ref(account, storage_key.into()) {
            Ok(value) => Ok(Some(value)),
            Err(e) => Err(ProviderError::Other(AnyError::new(e))),
        }
    }

    // Will be easier with https://github.com/paradigmxyz/reth/issues/14479
    fn bytecode_by_hash(&self, code_hash: &B256) -> ProviderResult<Option<Bytecode>> {
        // revm will first call account info, which will insert the bytecode into the hashmap
        Ok(self.bytecode.read().get(code_hash).cloned())
    }
}
