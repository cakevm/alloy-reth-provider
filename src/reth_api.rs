use alloy_network::Network;
use alloy_provider::Provider;
use alloy_transport::TransportResult;
use async_trait::async_trait;
use reth_chain_state::CanonStateNotification;

#[async_trait]
pub trait RethApi<N>: Send + Sync {
    async fn subscribe_subscribe_chain_notifications(&self) -> TransportResult<alloy_pubsub::Subscription<CanonStateNotification>>;
}

#[async_trait]
impl<N, P> RethApi<N> for P
where
    N: Network,
    P: Provider<N>,
{
    async fn subscribe_subscribe_chain_notifications(&self) -> TransportResult<alloy_pubsub::Subscription<CanonStateNotification>> {
        self.root().client().pubsub_frontend().ok_or_else(alloy_transport::TransportErrorKind::pubsub_unavailable)?;

        let mut call = self.client().request("reth_subscribeChainNotifications", ());
        call.set_is_subscription();
        let id = call.await?;
        self.root().get_subscription(id).await
    }
}
