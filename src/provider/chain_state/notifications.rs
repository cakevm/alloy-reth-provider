use crate::primitives::AlloyRethNodePrimitives;
use crate::{AlloyNetwork, AlloyRethProvider};
use alloy_provider::Provider;
use reth_ethereum_primitives::EthPrimitives;
use reth_provider::{CanonStateNotifications, CanonStateSubscriptions, NodePrimitivesProvider};
use std::fmt::Debug;

impl<P, NP> NodePrimitivesProvider for AlloyRethProvider<P, NP>
where
    NP: AlloyRethNodePrimitives,
    P: 'static + Clone + Debug + Provider<AlloyNetwork> + Send + Sync,
{
    type Primitives = EthPrimitives;
}

impl<P, NP> CanonStateSubscriptions for AlloyRethProvider<P, NP>
where
    P: Provider<AlloyNetwork> + Send + Sync + Clone + Debug + 'static,
    NP: AlloyRethNodePrimitives,
{
    fn subscribe_to_canonical_state(&self) -> CanonStateNotifications<Self::Primitives> {
        self.canon_state_notification_sender.subscribe()
    }
}
