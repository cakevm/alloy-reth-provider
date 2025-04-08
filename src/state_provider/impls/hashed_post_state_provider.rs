use crate::AlloyRethStateProvider;
use alloy_network::Network;
use alloy_provider::Provider;
use reth_provider::HashedPostStateProvider;
use reth_trie::{HashedPostState, KeccakKeyHasher};
use revm_database::BundleState;

impl<N, P> HashedPostStateProvider for AlloyRethStateProvider<N, P>
where
    N: Network,
    P: Clone + Provider<N>,
{
    fn hashed_post_state(&self, bundle_state: &BundleState) -> HashedPostState {
        HashedPostState::from_bundle_state::<KeccakKeyHasher>(bundle_state.state())
    }
}
