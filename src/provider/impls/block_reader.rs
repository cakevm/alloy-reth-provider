use crate::AlloyRethProvider;
use alloy_eips::{BlockHashOrNumber, BlockNumberOrTag};
use alloy_network::primitives::{BlockTransactions, BlockTransactionsKind};
use alloy_network::{BlockResponse, Network};
use alloy_primitives::{BlockNumber, B256};
use alloy_provider::Provider;
use reth_errors::{ProviderError, ProviderResult};
use reth_primitives::{RecoveredBlock, SealedBlock, TransactionSigned};
use reth_provider::errors::any::AnyError;
pub(crate) use reth_provider::{BlockReader, BlockSource, TransactionVariant};
use std::future::IntoFuture;
use std::ops::RangeInclusive;
use tokio::runtime::Handle;

impl<N, P> BlockReader for AlloyRethProvider<N, P>
where
    N: Network<
        TransactionResponse = alloy_rpc_types_eth::Transaction,
        HeaderResponse = alloy_rpc_types_eth::Header,
        BlockResponse = alloy_rpc_types_eth::Block,
    >,
    P: Provider<N> + Send + Sync + Clone + 'static,
{
    type Block = alloy_consensus::Block<TransactionSigned>;

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
                        let transactions = transactions.into_iter().map(|tx| tx.into()).collect::<Vec<TransactionSigned>>();
                        let body = alloy_consensus::BlockBody { transactions, ommers: vec![], withdrawals };

                        Ok(Some(alloy_consensus::Block::new(header, body)))
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
                        let transactions = transactions.into_iter().map(|tx| tx.into()).collect::<Vec<TransactionSigned>>();
                        let body = alloy_consensus::BlockBody { transactions, ommers: vec![], withdrawals };

                        Ok(Some(alloy_consensus::Block::new(header, body)))
                    }

                    Ok(None) => Err(ProviderError::BlockHashNotFound(block_hash)),
                    Err(e) => Err(ProviderError::Other(AnyError::new(e))),
                }
            }
        }
    }

    fn pending_block(&self) -> ProviderResult<Option<SealedBlock<Self::Block>>> {
        todo!()
    }

    fn pending_block_with_senders(&self) -> ProviderResult<Option<RecoveredBlock<Self::Block>>> {
        todo!()
    }

    fn pending_block_and_receipts(&self) -> ProviderResult<Option<(SealedBlock<Self::Block>, Vec<Self::Receipt>)>> {
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

    fn recovered_block(
        &self,
        _id: BlockHashOrNumber,
        _transaction_kind: TransactionVariant,
    ) -> ProviderResult<Option<RecoveredBlock<Self::Block>>> {
        todo!()
    }

    fn recovered_block_range(&self, _range: RangeInclusive<BlockNumber>) -> ProviderResult<Vec<RecoveredBlock<Self::Block>>> {
        todo!()
    }
}
