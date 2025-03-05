use crate::alloy_reth_state_provider::AlloyRethStateProvider;
use alloy_consensus::BlockHeader;
use alloy_eips::eip4895::Withdrawals;
use alloy_eips::{BlockHashOrNumber, BlockNumberOrTag};
use alloy_network::primitives::{BlockTransactionsKind, HeaderResponse};
use alloy_network::{BlockResponse, Network};
use alloy_primitives::{Address, BlockHash, BlockNumber, TxHash, TxNumber, B256, U256};
use alloy_provider::Provider;
use alloy_rpc_types_eth::BlockTransactions;
use reth_chainspec::{ChainSpec, HOLESKY, MAINNET};
use reth_db_models::StoredBlockBodyIndices;
use reth_errors::{ProviderError, ProviderResult};
use reth_primitives::{Receipt, RecoveredBlock, SealedBlock, SealedHeader, TransactionMeta, TransactionSigned};
use reth_provider::errors::any::AnyError;
use reth_provider::{
    BlockBodyIndicesProvider, BlockHashReader, BlockIdReader, BlockNumReader, BlockReader, BlockSource, ChainSpecProvider, HeaderProvider,
    OmmersProvider, ReceiptProvider, StateProviderBox, StateProviderFactory, TransactionVariant, TransactionsProvider, WithdrawalsProvider,
};
use std::marker::PhantomData;
use std::ops::{RangeBounds, RangeInclusive};
use std::sync::Arc;
use tokio::runtime::Handle;

#[derive(Clone)]
pub struct AlloyRethProvider<N, P: Send + Sync + Clone + 'static> {
    provider: P,
    _n: PhantomData<N>,
}

impl<N, P> AlloyRethProvider<N, P>
where
    N: Network,
    P: Provider<N> + Send + Sync + Clone + 'static,
{
    pub fn new(provider: P) -> Self {
        Self { provider, _n: PhantomData }
    }
}

impl<N, P> BlockIdReader for AlloyRethProvider<N, P>
where
    N: Network,
    P: Provider<N> + Send + Sync + Clone + 'static,
{
    fn pending_block_num_hash(&self) -> ProviderResult<Option<alloy_eips::BlockNumHash>> {
        todo!()
    }

    fn safe_block_num_hash(&self) -> ProviderResult<Option<alloy_eips::BlockNumHash>> {
        todo!()
    }

    fn finalized_block_num_hash(&self) -> ProviderResult<Option<alloy_eips::BlockNumHash>> {
        todo!()
    }
}

impl<N, P> BlockNumReader for AlloyRethProvider<N, P>
where
    N: Network,
    P: Provider<N> + Send + Sync + Clone + 'static,
{
    fn chain_info(&self) -> ProviderResult<reth_chainspec::ChainInfo> {
        todo!()
    }

    fn best_block_number(&self) -> ProviderResult<BlockNumber> {
        let block_number = tokio::task::block_in_place(move || Handle::current().block_on(self.provider.get_block_number()));
        match block_number {
            Ok(block_number) => Ok(block_number),
            Err(e) => Err(ProviderError::Other(AnyError::new(e))),
        }
    }

    fn last_block_number(&self) -> ProviderResult<BlockNumber> {
        todo!()
    }

    fn block_number(&self, hash: B256) -> ProviderResult<Option<BlockNumber>> {
        let block = tokio::task::block_in_place(move || {
            Handle::current().block_on(self.provider.get_block_by_hash(hash, BlockTransactionsKind::Hashes))
        });
        match block {
            Ok(Some(block)) => Ok(Some(block.header().number())),
            Ok(None) => Err(ProviderError::BlockHashNotFound(hash)),
            Err(e) => Err(ProviderError::Other(AnyError::new(e))),
        }
    }
}

impl<N, P> BlockHashReader for AlloyRethProvider<N, P>
where
    N: Network,
    P: Provider<N> + Send + Sync + Clone + 'static,
{
    fn block_hash(&self, number: BlockNumber) -> ProviderResult<Option<B256>> {
        let block = tokio::task::block_in_place(move || {
            Handle::current().block_on(self.provider.get_block_by_number(number.into(), BlockTransactionsKind::Hashes))
        });
        match block {
            Ok(Some(block)) => Ok(Some(B256::from(*block.header().hash()))),
            Ok(None) => Err(ProviderError::BlockBodyIndicesNotFound(number)),
            Err(e) => Err(ProviderError::Other(AnyError::new(e))),
        }
    }

    fn canonical_hashes_range(&self, _start: BlockNumber, _end: BlockNumber) -> ProviderResult<Vec<B256>> {
        todo!()
    }
}

impl<N, P> ChainSpecProvider for AlloyRethProvider<N, P>
where
    N: Network,
    P: Provider<N> + Send + Sync + Clone + 'static,
{
    type ChainSpec = ChainSpec;

    fn chain_spec(&self) -> Arc<ChainSpec> {
        let chain_id = tokio::task::block_in_place(move || Handle::current().block_on(self.provider.get_chain_id())).unwrap_or(1);
        if chain_id == 17000 {
            HOLESKY.clone()
        } else {
            MAINNET.clone()
        }
    }
}

impl<N, P> StateProviderFactory for AlloyRethProvider<N, P>
where
    N: Network,
    P: Provider<N> + Send + Sync + Clone + 'static,
{
    fn latest(&self) -> ProviderResult<StateProviderBox> {
        let block_number = tokio::task::block_in_place(move || Handle::current().block_on(self.provider.get_block_number()));
        match block_number {
            Ok(block_number) => self.state_by_block_number_or_tag(BlockNumberOrTag::Number(block_number)),
            Err(e) => Err(ProviderError::Other(AnyError::new(e))),
        }
    }

    /// Returns a [`StateProviderBox`] indexed by the given block number or tag.
    fn state_by_block_number_or_tag(&self, number_or_tag: BlockNumberOrTag) -> ProviderResult<StateProviderBox> {
        match number_or_tag {
            BlockNumberOrTag::Latest => self.latest(),
            BlockNumberOrTag::Finalized => {
                // we can only get the finalized state by hash, not by num
                let hash = self.finalized_block_hash()?.ok_or(ProviderError::FinalizedBlockNotFound)?;
                self.state_by_block_hash(hash)
            }
            BlockNumberOrTag::Safe => {
                // we can only get the safe state by hash, not by num
                let hash = self.safe_block_hash()?.ok_or(ProviderError::SafeBlockNotFound)?;
                self.state_by_block_hash(hash)
            }
            BlockNumberOrTag::Earliest => self.history_by_block_number(0),
            BlockNumberOrTag::Pending => self.pending(),
            BlockNumberOrTag::Number(num) => {
                let hash = self.block_hash(num)?.ok_or_else(|| ProviderError::HeaderNotFound(num.into()))?;
                self.state_by_block_hash(hash)
            }
        }
    }

    fn history_by_block_number(&self, _block: BlockNumber) -> ProviderResult<StateProviderBox> {
        todo!()
    }

    fn history_by_block_hash(&self, block: BlockHash) -> ProviderResult<StateProviderBox> {
        Ok(Box::new(AlloyRethStateProvider::new(self.provider.clone(), block.into())))
    }

    fn state_by_block_hash(&self, hash: BlockHash) -> ProviderResult<StateProviderBox> {
        if let Ok(state) = self.history_by_block_hash(hash) {
            // This could be tracked by a historical block
            Ok(state)
        } else {
            // if we couldn't find it anywhere, then we should return an error
            Err(ProviderError::StateForHashNotFound(hash))
        }
    }

    fn pending(&self) -> ProviderResult<StateProviderBox> {
        todo!()
    }

    fn pending_state_by_hash(&self, _block_hash: B256) -> ProviderResult<Option<StateProviderBox>> {
        // not supported by rpc
        todo!()
    }
}

impl<N, P> HeaderProvider for AlloyRethProvider<N, P>
where
    N: Network<HeaderResponse = alloy_rpc_types_eth::Header>,
    P: 'static + Clone + Provider<N> + Send + Sync,
{
    type Header = alloy_consensus::Header;

    fn header(&self, block_hash: &BlockHash) -> ProviderResult<Option<Self::Header>> {
        let block = tokio::task::block_in_place(move || {
            Handle::current().block_on(self.provider.get_block_by_hash(*block_hash, BlockTransactionsKind::Hashes))
        });
        match block {
            Ok(Some(block)) => Ok(Some(block.header().clone().into())),
            Ok(None) => Err(ProviderError::BlockHashNotFound(*block_hash)),
            Err(e) => Err(ProviderError::Other(AnyError::new(e))),
        }
    }

    fn header_by_number(&self, _num: u64) -> ProviderResult<Option<Self::Header>> {
        todo!()
    }

    fn header_td(&self, _hash: &BlockHash) -> ProviderResult<Option<U256>> {
        todo!()
    }

    fn header_td_by_number(&self, _number: BlockNumber) -> ProviderResult<Option<U256>> {
        todo!()
    }

    fn headers_range(&self, _range: impl RangeBounds<BlockNumber>) -> ProviderResult<Vec<Self::Header>> {
        todo!()
    }

    fn sealed_header(&self, _number: BlockNumber) -> ProviderResult<Option<SealedHeader<Self::Header>>> {
        todo!()
    }

    fn sealed_headers_while(
        &self,
        _range: impl RangeBounds<BlockNumber>,
        _predicate: impl FnMut(&SealedHeader<Self::Header>) -> bool,
    ) -> ProviderResult<Vec<SealedHeader<Self::Header>>> {
        todo!()
    }
}

impl<N, P> TransactionsProvider for AlloyRethProvider<N, P>
where
    N: Network,
    P: 'static + Clone + Provider<N> + Send + Sync,
{
    type Transaction = reth_primitives::TransactionSigned;

    fn transaction_id(&self, _tx_hash: TxHash) -> ProviderResult<Option<TxNumber>> {
        todo!()
    }

    fn transaction_by_id(&self, _id: TxNumber) -> ProviderResult<Option<Self::Transaction>> {
        todo!()
    }

    fn transaction_by_id_unhashed(&self, _id: TxNumber) -> ProviderResult<Option<Self::Transaction>> {
        todo!()
    }

    fn transaction_by_hash(&self, _hash: TxHash) -> ProviderResult<Option<Self::Transaction>> {
        todo!()
    }

    fn transaction_by_hash_with_meta(&self, _hash: TxHash) -> ProviderResult<Option<(Self::Transaction, TransactionMeta)>> {
        todo!()
    }

    fn transaction_block(&self, _id: TxNumber) -> ProviderResult<Option<BlockNumber>> {
        todo!()
    }

    fn transactions_by_block(&self, _block: BlockHashOrNumber) -> ProviderResult<Option<Vec<Self::Transaction>>> {
        todo!()
    }

    fn transactions_by_block_range(&self, _range: impl RangeBounds<BlockNumber>) -> ProviderResult<Vec<Vec<Self::Transaction>>> {
        todo!()
    }

    fn transactions_by_tx_range(&self, _range: impl RangeBounds<TxNumber>) -> ProviderResult<Vec<Self::Transaction>> {
        todo!()
    }

    fn senders_by_tx_range(&self, _range: impl RangeBounds<TxNumber>) -> ProviderResult<Vec<Address>> {
        todo!()
    }

    fn transaction_sender(&self, _id: TxNumber) -> ProviderResult<Option<Address>> {
        todo!()
    }
}

impl<N, P> ReceiptProvider for AlloyRethProvider<N, P>
where
    N: Network,
    P: 'static + Clone + Provider<N> + Send + Sync,
{
    type Receipt = reth_primitives::Receipt;

    fn receipt(&self, _id: TxNumber) -> ProviderResult<Option<Receipt>> {
        todo!()
    }

    fn receipt_by_hash(&self, _hash: TxHash) -> ProviderResult<Option<Receipt>> {
        todo!()
    }

    fn receipts_by_block(&self, _block: BlockHashOrNumber) -> ProviderResult<Option<Vec<Receipt>>> {
        todo!()
    }

    fn receipts_by_tx_range(&self, _range: impl RangeBounds<TxNumber>) -> ProviderResult<Vec<Receipt>> {
        todo!()
    }
}

impl<N, P> WithdrawalsProvider for AlloyRethProvider<N, P>
where
    N: Network,
    P: 'static + Clone + Provider<N> + Send + Sync,
{
    fn withdrawals_by_block(&self, _id: BlockHashOrNumber, _timestamp: u64) -> ProviderResult<Option<Withdrawals>> {
        todo!()
    }
}

impl<N, P> BlockBodyIndicesProvider for AlloyRethProvider<N, P>
where
    N: Network,
    P: 'static + Clone + Provider<N> + Send + Sync,
{
    fn block_body_indices(&self, _num: u64) -> ProviderResult<Option<StoredBlockBodyIndices>> {
        todo!()
    }

    fn block_body_indices_range(&self, _range: RangeInclusive<BlockNumber>) -> ProviderResult<Vec<StoredBlockBodyIndices>> {
        todo!()
    }
}

impl<N, P> OmmersProvider for AlloyRethProvider<N, P>
where
    N: Network<HeaderResponse = alloy_rpc_types_eth::Header>,
    P: 'static + Clone + Provider<N> + Send + Sync,
{
    fn ommers(&self, _id: BlockHashOrNumber) -> ProviderResult<Option<Vec<Self::Header>>> {
        todo!()
    }
}

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
                    Handle::current()
                        .block_on(self.provider.get_block_by_number(BlockNumberOrTag::Number(block_number), BlockTransactionsKind::Full))
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
                    Handle::current().block_on(self.provider.get_block_by_hash(block_hash, BlockTransactionsKind::Full))
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

    fn block_with_senders(
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

    fn sealed_block_with_senders_range(&self, _range: RangeInclusive<BlockNumber>) -> ProviderResult<Vec<RecoveredBlock<Self::Block>>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_eips::BlockId;
    use alloy_primitives::address;
    use alloy_provider::ProviderBuilder;
    use reth_provider::AccountReader;
    use ruint::__private::ruint_macro::uint;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_alloy_reth_state_provider_factory() {
        let provider = ProviderBuilder::new().on_http("https://eth.merkle.io".parse().unwrap());
        let db_provider = AlloyRethProvider::new(provider);
        let state = db_provider.state_by_block_id(BlockId::number(16148323)).unwrap();
        let acc_info = state.basic_account(&address!("220866b1a2219f40e72f5c628b65d54268ca3a9d")).unwrap().unwrap();

        assert_eq!(acc_info.nonce, 1);
        assert_eq!(acc_info.balance, uint!(250001010477701567100010_U256));
    }
}
