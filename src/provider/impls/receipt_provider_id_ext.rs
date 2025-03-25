use crate::primitives::AlloyRethNodePrimitives;
use crate::{AlloyNetwork, AlloyRethProvider};
use alloy_provider::Provider;
use reth_provider::ReceiptProviderIdExt;

impl<P, NP> ReceiptProviderIdExt for AlloyRethProvider<P, NP>
where
    NP: AlloyRethNodePrimitives,
    P: 'static + Clone + Provider<AlloyNetwork> + Send + Sync,
{
}
