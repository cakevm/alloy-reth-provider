#[cfg(not(feature = "optimism"))]
use reth_ethereum_primitives::EthPrimitives;
#[cfg(feature = "optimism")]
use reth_optimism_primitives::OpPrimitives;
use reth_primitives_traits::NodePrimitives;

#[cfg(not(feature = "optimism"))]
pub trait AlloyRethNodePrimitives:
    NodePrimitives<
    Block = <EthPrimitives as NodePrimitives>::Block,
    SignedTx = <EthPrimitives as NodePrimitives>::SignedTx,
    BlockHeader = <EthPrimitives as NodePrimitives>::BlockHeader,
>
{
}

#[cfg(feature = "optimism")]
pub trait AlloyRethNodePrimitives:
    NodePrimitives<
    Block = <OpPrimitives as NodePrimitives>::Block,
    SignedTx = <OpPrimitives as NodePrimitives>::SignedTx,
    BlockHeader = <OpPrimitives as NodePrimitives>::BlockHeader,
>
{
}

#[cfg(not(feature = "optimism"))]
impl AlloyRethNodePrimitives for EthPrimitives {}

#[cfg(feature = "optimism")]
impl AlloyRethNodePrimitives for OpPrimitives {}
