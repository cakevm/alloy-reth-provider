use crate::AlloyRethStateProvider;
use alloy_network::Network;
use alloy_primitives::{Address, Bytes, B256};
use alloy_provider::Provider;
use reth_errors::ProviderResult;
use reth_provider::StateProofProvider;
use reth_trie::{AccountProof, HashedPostState, MultiProof, MultiProofTargets, TrieInput};

impl<N, P> StateProofProvider for AlloyRethStateProvider<N, P>
where
    N: Network,
    P: Provider<N> + Clone,
{
    fn proof(&self, _input: TrieInput, _address: Address, _slots: &[B256]) -> ProviderResult<AccountProof> {
        todo!()
    }

    fn multiproof(&self, _input: TrieInput, _targets: MultiProofTargets) -> ProviderResult<MultiProof> {
        todo!()
    }

    fn witness(&self, _input: TrieInput, _target: HashedPostState) -> ProviderResult<Vec<Bytes>> {
        todo!()
    }
}
