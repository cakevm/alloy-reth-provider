use crate::AlloyRethStateProvider;
use alloy_network::Network;
use alloy_primitives::{Address, StorageKey, StorageValue, B256};
use alloy_provider::Provider;
use reth_errors::{ProviderError, ProviderResult};
use reth_primitives_traits::Bytecode;
use reth_provider::errors::any::AnyError;
use reth_provider::{HashedPostStateProvider, StateProvider, StateRootProvider};
use reth_trie::updates::TrieUpdates;
use reth_trie::{HashedPostState, KeccakKeyHasher, TrieInput};
use revm_database::BundleState;
use revm_database::DatabaseRef;
use tracing::warn;

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

impl<N, P> StateRootProvider for AlloyRethStateProvider<N, P>
where
    N: Network,
    P: Provider<N> + Clone,
{
    fn state_root(&self, hashed_state: HashedPostState) -> ProviderResult<B256> {
        self.state_root_from_nodes(TrieInput::from_state(hashed_state))
    }

    fn state_root_from_nodes(&self, _input: TrieInput) -> ProviderResult<B256> {
        warn!("state_root_from_nodes is not implemented and will return zero");
        Ok(B256::ZERO)
    }

    fn state_root_with_updates(&self, hashed_state: HashedPostState) -> ProviderResult<(B256, TrieUpdates)> {
        self.state_root_from_nodes_with_updates(TrieInput::from_state(hashed_state))
    }

    fn state_root_from_nodes_with_updates(&self, _input: TrieInput) -> ProviderResult<(B256, TrieUpdates)> {
        warn!("state_root_from_nodes_with_updates is not implemented and will return zero");
        Ok((B256::ZERO, TrieUpdates::default()))
    }
}

impl<N, P> HashedPostStateProvider for AlloyRethStateProvider<N, P>
where
    N: Network,
    P: Clone + Provider<N>,
{
    fn hashed_post_state(&self, bundle_state: &BundleState) -> HashedPostState {
        HashedPostState::from_bundle_state::<KeccakKeyHasher>(bundle_state.state())
    }
}
