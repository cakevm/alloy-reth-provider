#[cfg(not(feature = "optimism"))]
use alloy_consensus::transaction::PooledTransaction;
#[cfg(not(feature = "optimism"))]
use alloy_eips::eip4844::env_settings::EnvKzgSettings;
#[cfg(not(feature = "optimism"))]
use alloy_eips::{Decodable2718, Encodable2718};
#[cfg(not(feature = "optimism"))]
use alloy_provider::{Provider, ProviderBuilder, WsConnect};
#[cfg(not(feature = "optimism"))]
use alloy_reth_provider::AlloyRethProvider;
#[cfg(not(feature = "optimism"))]
use eyre::eyre;
#[cfg(not(feature = "optimism"))]
use futures_util::stream::StreamExt;
#[cfg(not(feature = "optimism"))]
use reth_ethereum_primitives::{EthPrimitives, TransactionSigned};
#[cfg(not(feature = "optimism"))]
use reth_primitives::{BlockBody, SealedBlock};
#[cfg(not(feature = "optimism"))]
use reth_primitives_traits::{RecoveredBlock, SealedHeader};
#[cfg(not(feature = "optimism"))]
use reth_provider::{BlockNumReader, BlockReader, CanonStateNotification, CanonStateSubscriptions, Chain, ExecutionOutcome};
#[cfg(not(feature = "optimism"))]
use reth_transaction_pool::blobstore::InMemoryBlobStore;
#[cfg(not(feature = "optimism"))]
use reth_transaction_pool::{
    CoinbaseTipOrdering, EthPooledTransaction, PoolTransaction, TransactionPool, TransactionValidationTaskExecutor,
};
#[cfg(not(feature = "optimism"))]
use std::future::pending;
#[cfg(not(feature = "optimism"))]
use std::sync::Arc;
#[cfg(not(feature = "optimism"))]
use tracing_subscriber::layer::SubscriberExt;
#[cfg(not(feature = "optimism"))]
use tracing_subscriber::util::SubscriberInitExt;
#[cfg(not(feature = "optimism"))]
use tracing_subscriber::{fmt, EnvFilter};

#[cfg(feature = "optimism")]
fn main() {
    println!("Optimism not implemented");
}

/// This example shows how to use the `reth` transaction pool with a WebSocket provider and AlloyRethProvider.
///
/// In this example we spawn 3 tasks:
/// 1. A task to listen for new blocks and send them to the canon state
/// 2. A task to listen for new transactions from the WS provider and add them to the pool
/// 3. A task to listen for new transactions in the pool and print them
#[cfg(not(feature = "optimism"))]
#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::registry().with(fmt::layer()).with(EnvFilter::from("info")).init();

    let ws = WsConnect::new("wss://eth.llamarpc.com");
    let ws_provider = ProviderBuilder::new().connect_ws(ws).await?;
    let reth_provider = AlloyRethProvider::new(ws_provider.clone(), EthPrimitives::default());

    let canon_state_notification_sender = reth_provider.canon_state_notification_sender.clone();
    let block_subscription = ws_provider.subscribe_full_blocks().full();

    let manager = reth_tasks::TaskManager::new(tokio::runtime::Handle::current());
    let executor = manager.executor();

    // Spawn a 1. task to listen for new blocks and send them to the canon state
    executor.spawn_critical("new-block-stream", async move {
        let mut stream = block_subscription.into_stream().await.unwrap();
        while let Some(block_res) = stream.next().await {
            let block = block_res.unwrap();
            let header = block.header.clone();

            // We need to convert an RPC block to a reth recovered block
            let block = block.map_transactions(|tx| tx.into_inner().into());
            let block_body = BlockBody::<TransactionSigned> {
                transactions: block.transactions.into_transactions().collect(),
                ommers: vec![],
                withdrawals: block.withdrawals,
            };
            let sealed_header = SealedHeader::new(header.inner, block.header.hash);
            let sealed_block = SealedBlock::from_sealed_parts(sealed_header, block_body);
            let recovered_block = RecoveredBlock::try_recover_sealed(sealed_block).unwrap();

            // ExecutionOutcome should be not important for the pool
            let chain = Chain::new(vec![recovered_block], ExecutionOutcome::default(), None);
            let commit = CanonStateNotification::Commit { new: Arc::new(chain.clone()) };
            canon_state_notification_sender.send(commit).unwrap();
        }
    });

    // Fetch the best block from the database
    let best_block_number = reth_provider.best_block_number()?;
    let best_block = reth_provider.block_by_number(best_block_number)?.ok_or(eyre!("could not get best block"))?;

    let blob_store = InMemoryBlobStore::default();

    let validator = TransactionValidationTaskExecutor::eth_builder(reth_provider.clone())
        .with_head_timestamp(best_block.timestamp)
        .kzg_settings(EnvKzgSettings::Default)
        .with_additional_tasks(1)
        .build_with_tasks::<EthPooledTransaction, _, _>(executor.clone(), blob_store.clone());
    let pool =
        reth_transaction_pool::Pool::new(validator, CoinbaseTipOrdering::default(), InMemoryBlobStore::default(), Default::default());
    let chain_events = reth_provider.canonical_state_stream();
    executor.spawn_critical(
        "txpool-maintenance-task",
        reth_transaction_pool::maintain::maintain_transaction_pool_future(
            reth_provider.clone(),
            pool.clone(),
            chain_events,
            executor.clone(),
            Default::default(),
        ),
    );

    // Spawn 2. task to put pending transactions from WS provider to the pool
    let tx_subscription = ws_provider.subscribe_pending_transactions().await?;
    let pool_clone = pool.clone();
    let ws_provider_clone = ws_provider.clone();
    executor.spawn_critical("pending-tx-stream", async move {
        let mut stream = tx_subscription.into_stream();
        while let Some(tx_hash) = stream.next().await {
            match ws_provider_clone.get_transaction_by_hash(tx_hash).await {
                Ok(Some(tx)) => {
                    // The easiest way to get `alloy_rpc_types_eth` to the reth type is by encoding and decoding
                    // But 4844 transactions are not working yet
                    let tx_raw = tx.inner.encoded_2718();
                    let tx = match PooledTransaction::decode_2718(&mut tx_raw.as_ref()) {
                        Ok(tx) => tx,
                        Err(e) => {
                            tracing::error!("Error decoding tx: {:?}", e);
                            continue;
                        }
                    };
                    let recovered = match tx.try_into_recovered() {
                        Ok(recovered) => recovered,
                        Err(e) => {
                            tracing::error!("Error recovering tx: {:?}", e);
                            continue;
                        }
                    };

                    if let Err(e) = pool_clone
                        .add_transaction(reth_transaction_pool::TransactionOrigin::External, EthPooledTransaction::from_pooled(recovered))
                        .await
                    {
                        tracing::error!("Error adding tx with hash {}: {:?}", tx_hash, e);
                    }
                }
                Ok(None) => {
                    tracing::warn!("No tx for hash: {}", tx_hash);
                }
                Err(e) => {
                    tracing::error!("Error get tx by hash {}: {:?}", tx_hash, e);
                }
            }
        }
    });

    // Spawn 3. task to listen for new transactions in the pool
    executor.spawn_critical("txpool-receiver-task", async move {
        let mut pending_transactions = pool.new_pending_pool_transactions_listener();
        while let Some(event) = pending_transactions.next().await {
            let tx = event.transaction;
            println!("Transaction received: {tx:?}");
        }
    });

    pending().await
}
