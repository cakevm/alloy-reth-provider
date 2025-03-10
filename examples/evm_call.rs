#[cfg(not(feature = "optimism"))]
use alloy_eips::BlockId;
#[cfg(not(feature = "optimism"))]
use alloy_primitives::utils::parse_units;
#[cfg(not(feature = "optimism"))]
use alloy_primitives::{address, Bytes, B256, U256};
#[cfg(not(feature = "optimism"))]
use alloy_provider::ProviderBuilder;
#[cfg(not(feature = "optimism"))]
use alloy_reth_provider::AlloyRethProvider;
#[cfg(not(feature = "optimism"))]
use reth_ethereum_primitives::EthPrimitives;
#[cfg(not(feature = "optimism"))]
use reth_provider::StateProviderFactory;
#[cfg(not(feature = "optimism"))]
use reth_revm::database::StateProviderDatabase;
#[cfg(not(feature = "optimism"))]
use revm::{ExecuteEvm, MainBuilder, MainContext};
#[cfg(not(feature = "optimism"))]
use revm_context::result::ResultAndState;
#[cfg(not(feature = "optimism"))]
use revm_context::{BlockEnv, Context, Evm, TransactTo, TxEnv};
#[cfg(not(feature = "optimism"))]
use std::str::FromStr;

#[cfg(feature = "optimism")]
fn main() {
    println!("Optimism not implemented");
}

#[cfg(not(feature = "optimism"))]
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let provider = ProviderBuilder::default().on_http("https://eth.merkle.io".parse()?);
    let db_provider = AlloyRethProvider::new(provider, EthPrimitives::default());
    // Top of block state previous block
    let state_provider = db_provider.state_by_block_id(BlockId::number(16148322))?;
    let state_db = StateProviderDatabase::new(state_provider);

    let ctx = Context::mainnet()
        .with_db(state_db)
        .with_block(BlockEnv {
            // next block
            number: 16148323,
            timestamp: 1670565947,
            basefee: 176_658583385,
            ..BlockEnv::default()
        })
        .build_mainnet();

    // Simulate 0x2bb4f9fd58b3cd99591737eca784d0cf281034661bbdc509b58cf6a499d3d998
    let tx = TxEnv {
        caller: address!("0x80d3aB7f834f786CAb110Bea9E8A96d45B2dc9C2"),
        gas_limit: 55_984,
        gas_price: 241_400106299,
        gas_priority_fee: Some(parse_units("1.5", "gwei")?.get_absolute().to::<u128>()),
        kind: TransactTo::Call(address!("0x8686525d6627A25C68De82c228448f43c97999F2")),
        data: Bytes::from_str(
            "0x095ea7b3\
        00000000000000000000000068b3465833fb72a70ecdf485e0e4c7bd8665fc45\
        ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        )?,
        nonce: 401,
        chain_id: Some(1),
        ..TxEnv::default()
    };

    let mut evm = Evm::new(ctx);
    let ResultAndState { result, state } = evm.transact(tx)?;

    println!("Success: {}", result.is_success());
    let slot = state
        .get(&address!("0x8686525d6627A25C68De82c228448f43c97999F2"))
        .unwrap()
        .storage
        .get(&U256::from_str("0xd3e71b600a79d9f8f28b05f66aae715ed55392acd344d20f75999f38afc352d5")?)
        .unwrap();
    println!("Storage (before): {:?}", B256::from(slot.original_value));
    println!("Storage (after) : {:?}", B256::from(slot.present_value));

    Ok(())
}
