use crate::primitives::AlloyRethNodePrimitives;
use crate::{AlloyNetwork, AlloyRethProvider};
use alloy_eips::{BlockHashOrNumber, BlockId, BlockNumberOrTag};
use alloy_network::primitives::{BlockTransactions, BlockTransactionsKind};
use alloy_network::BlockResponse;
use alloy_primitives::{BlockNumber, B256};
use alloy_provider::Provider;
use reth_errors::{ProviderError, ProviderResult};
use reth_primitives::{BlockBody, RecoveredBlock, SealedBlock};
use reth_primitives_traits::{Block, SealedHeader};
use reth_provider::errors::any::AnyError;
use reth_provider::{BlockReader, BlockReaderIdExt, BlockSource, TransactionVariant};
use std::fmt::Debug;
use std::future::IntoFuture;
use std::ops::RangeInclusive;
use tokio::runtime::Handle;

impl<P, NP> BlockReader for AlloyRethProvider<P, NP>
where
    P: Provider<AlloyNetwork> + Send + Sync + Debug + Clone + 'static,
    NP: AlloyRethNodePrimitives,
{
    type Block = NP::Block;

    fn find_block_by_hash(&self, _hash: B256, _source: BlockSource) -> ProviderResult<Option<Self::Block>> {
        todo!()
    }

    fn block(&self, id: BlockHashOrNumber) -> ProviderResult<Option<Self::Block>> {
        match id {
            BlockHashOrNumber::Number(block_number) => {
                let block = tokio::task::block_in_place(move || {
                    Handle::current().block_on(
                        self.provider
                            .get_block_by_number(BlockNumberOrTag::Number(block_number))
                            .kind(BlockTransactionsKind::Full)
                            .into_future(),
                    )
                });
                match block {
                    Ok(Some(block)) => {
                        let header = block.header().clone().into();
                        let withdrawals = block.withdrawals;
                        let BlockTransactions::Full(transactions) = block.transactions else { unimplemented!() };
                        let transactions = transactions
                            .into_iter()
                            .map(|tx| {
                                #[cfg(not(feature = "optimism"))]
                                {
                                    tx.into_inner().into()
                                }

                                #[cfg(feature = "optimism")]
                                {
                                    tx.inner.into_inner()
                                }
                            })
                            .collect::<Vec<NP::SignedTx>>();
                        let body = BlockBody::<NP::SignedTx> { transactions, ommers: vec![], withdrawals };

                        Ok(Some(Block::new(header, body)))
                    }
                    Ok(None) => Err(ProviderError::BlockBodyIndicesNotFound(block_number)),
                    Err(e) => Err(ProviderError::Other(AnyError::new(e))),
                }
            }
            BlockHashOrNumber::Hash(block_hash) => {
                let block = tokio::task::block_in_place(move || {
                    Handle::current().block_on(self.provider.get_block_by_hash(block_hash).kind(BlockTransactionsKind::Full).into_future())
                });
                match block {
                    Ok(Some(block)) => {
                        let header = block.header().clone().into();
                        let withdrawals = block.withdrawals;
                        let BlockTransactions::Full(transactions) = block.transactions else { unimplemented!() };
                        let transactions = transactions
                            .into_iter()
                            .map(|tx| {
                                #[cfg(not(feature = "optimism"))]
                                {
                                    tx.into_inner().into()
                                }

                                #[cfg(feature = "optimism")]
                                {
                                    tx.inner.into_inner()
                                }
                            })
                            .collect::<Vec<NP::SignedTx>>();
                        let body = BlockBody::<NP::SignedTx> { transactions, ommers: vec![], withdrawals };

                        Ok(Some(Block::new(header, body)))
                    }

                    Ok(None) => Err(ProviderError::BlockHashNotFound(block_hash)),
                    Err(e) => Err(ProviderError::Other(AnyError::new(e))),
                }
            }
        }
    }

    fn pending_block(&self) -> ProviderResult<Option<RecoveredBlock<Self::Block>>> {
        todo!()
    }

    fn pending_block_and_receipts(&self) -> ProviderResult<Option<(SealedBlock<Self::Block>, Vec<Self::Receipt>)>> {
        todo!()
    }

    fn recovered_block(
        &self,
        _id: BlockHashOrNumber,
        _transaction_kind: TransactionVariant,
    ) -> ProviderResult<Option<RecoveredBlock<Self::Block>>> {
        todo!()
    }

    fn sealed_block_with_senders(
        &self,
        _id: BlockHashOrNumber,
        _transaction_kind: TransactionVariant,
    ) -> ProviderResult<Option<RecoveredBlock<Self::Block>>> {
        todo!()
    }

    fn block_range(&self, _range: RangeInclusive<BlockNumber>) -> ProviderResult<Vec<Self::Block>> {
        todo!()
    }

    fn block_with_senders_range(&self, _range: RangeInclusive<BlockNumber>) -> ProviderResult<Vec<RecoveredBlock<Self::Block>>> {
        todo!()
    }

    fn recovered_block_range(&self, _range: RangeInclusive<BlockNumber>) -> ProviderResult<Vec<RecoveredBlock<Self::Block>>> {
        todo!()
    }
}

impl<P, NP> BlockReaderIdExt for AlloyRethProvider<P, NP>
where
    P: Provider<AlloyNetwork> + Send + Sync + Debug + Clone + 'static,
    NP: AlloyRethNodePrimitives,
{
    fn block_by_id(&self, id: BlockId) -> ProviderResult<Option<Self::Block>> {
        match id {
            BlockId::Number(num) => self.block_by_number_or_tag(num),
            BlockId::Hash(hash) => self.block_by_hash(hash.block_hash),
        }
    }

    fn sealed_header_by_id(&self, _id: BlockId) -> ProviderResult<Option<SealedHeader<Self::Header>>> {
        todo!()
    }

    fn header_by_id(&self, _id: BlockId) -> ProviderResult<Option<Self::Header>> {
        todo!()
    }
}
