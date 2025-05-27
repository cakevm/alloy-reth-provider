#[cfg(not(feature = "optimism"))]
mod eth_imports {
    pub use alloy_eips::BlockId;
    pub use alloy_primitives::keccak256;
    pub use alloy_primitives::map::B256Map;
    pub use alloy_primitives::{address, U256};
    pub use alloy_provider::ProviderBuilder;
    pub use alloy_reth_provider::AlloyRethProvider;
    pub use reth_ethereum_primitives::EthPrimitives;
    pub use reth_provider::BlockReaderIdExt;
    pub use reth_provider::StateProviderFactory;
    pub use reth_provider::{AccountReader, StateRootProvider};
    pub use reth_trie::HashedPostState;
    pub use std::env;
    pub use std::time::Instant;
}

#[cfg(not(feature = "optimism"))]
use eth_imports::*;

#[cfg(feature = "optimism")]
fn main() {
    println!("Optimism not implemented");
}

#[cfg(not(feature = "optimism"))]
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let node_url = env::var("MAINNET_HTTP")?;
    let provider = ProviderBuilder::default().connect_http(node_url.parse()?);
    let db_provider = AlloyRethProvider::new(provider, EthPrimitives::default());

    let latest_block_num = db_provider.block_by_id(BlockId::latest())?.ok_or_else(|| eyre::eyre!("Latest block not found"))?.number;
    let offset = 1;
    // Get state provider for latest block
    let latest_state = db_provider.state_by_block_id(BlockId::number(latest_block_num - offset))?;

    // Some random state change
    let address = address!("0x4838b106fce9647bdf1e7877bf73ce8b0bad5f97");
    let mut account = latest_state.basic_account(&address)?.unwrap();
    account.balance = U256::ONE;
    let hashed_address = keccak256(address);
    let accounts = B256Map::from_iter([(hashed_address, Some(account))]);

    let hashed_post_state = HashedPostState { accounts, storages: Default::default() };

    // Get the state root and trie for our hashed post state
    let now = Instant::now();
    let (root, trie) = latest_state.state_root_with_updates(hashed_post_state)?;
    let elapsed = now.elapsed();
    println!("State root computed in: {} ms", elapsed.as_millis());
    println!("state root: {}", root);
    println!("account_nodes length: {}", trie.account_nodes.len());
    println!("removed_nodes length: {}", trie.removed_nodes.len());
    println!("storage_tries length: {}", trie.storage_tries.len());
    Ok(())
}
