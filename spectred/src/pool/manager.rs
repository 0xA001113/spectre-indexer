use deadpool::managed::{Manager, Metrics, RecycleError, RecycleResult};
use spectre_rpc_core::api::rpc::RpcApi;
use spectre_rpc_core::RpcNetworkType;
use spectre_wrpc_client::client::ConnectOptions;
use spectre_wrpc_client::error::Error;
use spectre_wrpc_client::prelude::*;
use spectre_wrpc_client::{SpectreRpcClient, WrpcEncoding};
use log::{debug, info, warn};
use std::sync::Arc;
use std::time::Duration;

pub struct SpectredManager {
    pub network_id: NetworkId,
    pub rpc_url: Option<String>,
}

impl Manager for SpectredManager {
    type Type = Arc<SpectreRpcClient>;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        debug!("Creating connection");
        Ok(Arc::new(connect_client(self.network_id, self.rpc_url.clone()).await?))
    }

    async fn recycle(&self, conn: &mut Self::Type, _: &Metrics) -> RecycleResult<Self::Error> {
        debug!("Recycling connection");
        if conn.is_connected() {
            Ok(())
        } else {
            let err_msg = "Spectred connection lost";
            warn!("{err_msg}");
            Err(RecycleError::Message(err_msg.into()))
        }
    }
}

pub async fn connect_client(network_id: NetworkId, rpc_url: Option<String>) -> Result<SpectreRpcClient, Error> {
    let url = if let Some(url) = &rpc_url { url } else { &Resolver::default().get_url(WrpcEncoding::Borsh, network_id).await? };

    debug!("Connecting to Spectred {}", url);
    let client = SpectreRpcClient::new_with_args(WrpcEncoding::Borsh, Some(url), None, Some(network_id), None)?;
    client.connect(Some(connect_options())).await.map_err(|e| {
        warn!("Spectred connection failed: {e}");
        e
    })?;

    let server_info = client.get_server_info().await?;
    let connected_network = format!(
        "{}{}",
        server_info.network_id.network_type,
        server_info.network_id.suffix.map(|s| format!("-{}", s)).unwrap_or_default()
    );
    info!("Connected to Spectred {}, version: {}, network: {}", url, server_info.server_version, connected_network);

    if network_id != server_info.network_id {
        panic!("Network mismatch, expected '{}', actual '{}'", network_id, connected_network);
    } else if !server_info.is_synced
        || server_info.network_id.network_type == RpcNetworkType::Mainnet && server_info.virtual_daa_score < 107107107
    {
        let err_msg = format!("Spectred {} is NOT synced", server_info.server_version);
        warn!("{err_msg}");
        Err(Error::Custom(err_msg))
    } else {
        Ok(client)
    }
}

fn connect_options() -> ConnectOptions {
    ConnectOptions {
        block_async_connect: true,
        strategy: ConnectStrategy::Fallback,
        url: None,
        connect_timeout: Some(Duration::from_secs(5)),
        retry_interval: None,
    }
}
