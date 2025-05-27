use crate::primitives::AlloyRethNodePrimitives;
use crate::state_provider::alloy_reth_state_provider::AlloyRethStateProvider;
use crate::{AlloyNetwork, AlloyRethProvider};
#[cfg(not(feature = "optimism"))]
use alloy_consensus::BlockHeader;
#[cfg(not(feature = "optimism"))]
use alloy_eips::BlockId;
use alloy_eips::BlockNumberOrTag;
use alloy_primitives::{BlockHash, BlockNumber, B256};
use alloy_provider::Provider;
#[cfg(not(feature = "optimism"))]
use reth_chainspec::ChainSpecProvider;
use reth_errors::{ProviderError, ProviderResult};
#[cfg(not(feature = "optimism"))]
use reth_ethereum_primitives::EthPrimitives;
#[cfg(not(feature = "optimism"))]
use reth_evm::execute::Executor;
#[cfg(not(feature = "optimism"))]
use reth_evm::ConfigureEvm;
#[cfg(not(feature = "optimism"))]
use reth_evm_ethereum::EthEvmConfig;
#[cfg(not(feature = "optimism"))]
use reth_primitives_traits::SealedBlock;
use reth_provider::errors::any::AnyError;
use reth_provider::{BlockHashReader, BlockIdReader, StateProviderBox, StateProviderFactory};
#[cfg(not(feature = "optimism"))]
use reth_provider::{BlockReader, ExecutionOutcome, StateReader};
#[cfg(not(feature = "optimism"))]
use reth_revm::database::StateProviderDatabase;
use std::fmt::Debug;
use tokio::runtime::Handle;

#[cfg(not(feature = "optimism"))]
impl<P, NP> StateReader for AlloyRethProvider<P, NP>
where
    P: Provider<AlloyNetwork> + Send + Sync + Debug + Clone + 'static,
    NP: AlloyRethNodePrimitives,
{
    type Receipt = reth_ethereum_primitives::Receipt;

    fn get_state(&self, block_number: BlockNumber) -> ProviderResult<Option<ExecutionOutcome<Self::Receipt>>> {
        let result = self.block_by_number(block_number)?;
        match result {
            Some(block) => {
                let sealed = SealedBlock::from(block);
                let provider = AlloyRethProvider::new(self.provider.clone(), EthPrimitives::default());
                // get state for the previous block
                let state_provider = provider.state_by_block_id(BlockId::number(block_number - 1))?;

                let db = StateProviderDatabase::new(&state_provider);

                let evm_config = EthEvmConfig::ethereum(provider.chain_spec());
                let executor = evm_config.batch_executor(db);
                let block_execution_output =
                    executor.execute(&sealed.clone().try_recover().map_err(ProviderError::other)?).map_err(ProviderError::other)?;
                let execution_outcome = ExecutionOutcome::from((block_execution_output, sealed.number()));

                Ok(Some(execution_outcome))
            }
            None => Err(ProviderError::BlockBodyIndicesNotFound(block_number)),
        }
    }
}

impl<P, NP> StateProviderFactory for AlloyRethProvider<P, NP>
where
    P: Provider<AlloyNetwork> + Send + Sync + Debug + Clone + 'static,
    NP: AlloyRethNodePrimitives,
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
        Ok(Box::new(AlloyRethStateProvider::new_with_config(self.provider.clone(), block.into(), self.state_provider_config.clone())))
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

#[cfg(not(feature = "optimism"))]
#[cfg(test)]
mod tests {
    use crate::AlloyRethProvider;
    use alloy_eips::BlockId;
    use alloy_node_bindings::Anvil;
    use alloy_primitives::address;
    use alloy_provider::ProviderBuilder;
    use reth_ethereum_primitives::EthPrimitives;
    use reth_provider::{StateProviderFactory, StateReader};
    use ruint::uint;
    use std::env;

    #[test_with::no_env(SKIP_RPC_HEAVY_TESTS)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_state() {
        let node_url = env::var("MAINNET_HTTP").unwrap();
        let provider = ProviderBuilder::new().connect_http(node_url.parse().unwrap());

        let db_provider = AlloyRethProvider::new(provider, EthPrimitives::default());
        let state = db_provider.get_state(16148323).unwrap().unwrap();

        let bundle_account = state.bundle.state.get(&address!("0x677cfeb3aabf8f58dee20d798cd4c2c1caef7c56")).unwrap();
        assert_eq!(bundle_account.original_info.as_ref().unwrap().balance, uint!(463708014023642423_U256));
        assert_eq!(bundle_account.info.as_ref().unwrap().balance, uint!(500708014023642423_U256));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_state_provider_factory_state_by_block_id() {
        let node_url = env::var("MAINNET_HTTP").unwrap_or("https://eth.merkle.io".to_string());
        let provider = ProviderBuilder::new().connect_http(node_url.parse().unwrap());

        let db_provider = AlloyRethProvider::new(provider, EthPrimitives::default());
        let state = db_provider.state_by_block_id(BlockId::number(16148323)).unwrap();
        let acc_info = state.basic_account(&address!("220866b1a2219f40e72f5c628b65d54268ca3a9d")).unwrap().unwrap();

        assert_eq!(acc_info.nonce, 1);
        assert_eq!(acc_info.balance, uint!(250001010477701567100010_U256));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_state_provider_factory_latest() {
        let node_url = env::var("MAINNET_HTTP").unwrap_or("https://eth.merkle.io".to_string());
        let anvil = Anvil::new().fork(node_url).fork_block_number(16148323).spawn();
        let provider = ProviderBuilder::new().connect_http(anvil.endpoint_url());

        let db_provider = AlloyRethProvider::new(provider, EthPrimitives::default());
        let state = db_provider.latest().unwrap();
        let acc_info = state.basic_account(&address!("220866b1a2219f40e72f5c628b65d54268ca3a9d")).unwrap().unwrap();

        assert_eq!(acc_info.nonce, 1);
        assert_eq!(acc_info.balance, uint!(250001010477701567100010_U256));
    }
}
