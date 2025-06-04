#[cfg(not(feature = "optimism"))]
mod eth_imports {
    pub use alloy_provider::ProviderBuilder;
    pub use alloy_provider::WsConnect;
    pub use alloy_reth_provider::AlloyRethProvider;
    pub use alloy_reth_provider::RethApi;
    pub use futures_util::{FutureExt, StreamExt};
    pub use reth_chain_state::CanonStateSubscriptions;
    pub use reth_ethereum_primitives::EthPrimitives;
    pub use std::env;
    pub use std::future::pending;
    pub use tracing_subscriber::layer::SubscriberExt;
    pub use tracing_subscriber::util::SubscriberInitExt;
    pub use tracing_subscriber::{fmt, EnvFilter};
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
    tracing_subscriber::registry().with(fmt::layer()).with(EnvFilter::from("info")).init();

    // Connect to a node that supports `reth_subscribeChainNotifications`
    let node_url = env::var("MAINNET_WS")?;
    let ws = WsConnect::new(node_url);
    let ws_provider = ProviderBuilder::new().connect_ws(ws).await?;

    // Create new alloy reth provider
    let reth_provider = AlloyRethProvider::new(ws_provider.clone(), EthPrimitives::default());

    // Spawn reth task manager
    let manager = reth_tasks::TaskManager::new(tokio::runtime::Handle::current());
    let executor = manager.executor();

    // Subscribe to canon state notifications
    let chain_notifications = match ws_provider.subscribe_subscribe_chain_notifications().await {
        Ok(subscription) => subscription,
        Err(e) => {
            eprintln!("Failed to subscribe to chain notifications: {}", e);
            return Err(eyre::eyre!("Subscription failed"));
        }
    };

    // Spawn task to forward canon state notifications from node to the reth provider
    let reth_provider_clone = reth_provider.clone();
    executor.spawn_critical("chain-notifications-forwarder", async move {
        // Create a channel to send canon state notifications
        let canon_notification_sender = reth_provider_clone.canon_state_notification_sender.clone();

        let mut stream = chain_notifications.into_stream();
        while let Some(canon_state_notification) = stream.next().await {
            if let Err(err) = canon_notification_sender.send(canon_state_notification) {
                eprintln!("Failed to send canon state notification: {}", err);
            }
        }
    });

    // Spawn task to handle chain state notifications
    executor.spawn_critical("chain-notifications-receiver", async move {
        println!("Waiting for chain state notifications...");
        // Subscribe to the canon state notifications using the `CanonStateSubscriptions` implementation
        let mut canon_notification_receiver = reth_provider.subscribe_to_canonical_state();

        loop {
            // Process notifications as they come in
            let chain_state_notification = match canon_notification_receiver.recv().await {
                Ok(chain_state_notification) => chain_state_notification,
                Err(err) => {
                    eprintln!("Failed to receive chain state notification: {}", err);
                    continue;
                }
            };
            println!("New chain state notification: block_number={}", chain_state_notification.tip().number);
        }
    });

    pending().await
}
