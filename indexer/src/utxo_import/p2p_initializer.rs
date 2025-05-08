use spectre_p2p_lib::common::ProtocolError;
use spectre_p2p_lib::pb::spectred_message::Payload;
use spectre_p2p_lib::pb::{SpectredMessage, VersionMessage};
use spectre_p2p_lib::{ConnectionInitializer, IncomingRoute, SpectredHandshake, SpectredMessagePayloadType, Router};
use spectre_cli::cli_args::CliArgs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::Sender;
use tonic::async_trait;
use uuid::Uuid;

pub struct P2pInitializer {
    cli_args: CliArgs,
    sender: Sender<SpectredMessage>,
}

impl P2pInitializer {
    pub fn new(cli_args: CliArgs, sender: Sender<SpectredMessage>) -> Self {
        P2pInitializer { cli_args, sender }
    }
}

#[async_trait]
impl ConnectionInitializer for P2pInitializer {
    async fn initialize_connection(&self, router: Arc<Router>) -> Result<(), ProtocolError> {
        let mut handshake = SpectredHandshake::new(&router);
        router.start();
        let sender = self.sender.clone();
        let version_msg = handshake.handshake(self.version_message()).await?;
        let mut incoming_route = subscribe_all(&router);
        tokio::spawn(async move {
            while let Some(msg) = incoming_route.recv().await {
                let _ = sender.send(msg).await;
            }
        });
        handshake.exchange_ready_messages().await?;
        self.sender.send(SpectredMessage { request_id: 0, response_id: 0, payload: Some(Payload::Version(version_msg)) }).await.unwrap();
        Ok(())
    }
}

impl P2pInitializer {
    pub fn version_message(&self) -> VersionMessage {
        VersionMessage {
            protocol_version: 7,
            services: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64,
            address: None,
            id: Vec::from(Uuid::new_v4().as_bytes()),
            user_agent: format!("/{}:{}/", env!("CARGO_PKG_NAME"), self.cli_args.version()),
            disable_relay_tx: true,
            subnetwork_id: None,
            network: format!("spectre-{}", self.cli_args.network.to_lowercase()),
        }
    }
}

fn subscribe_all(router: &Arc<Router>) -> IncomingRoute {
    router.subscribe(vec![
        SpectredMessagePayloadType::Addresses,
        SpectredMessagePayloadType::Block,
        SpectredMessagePayloadType::Transaction,
        SpectredMessagePayloadType::BlockLocator,
        SpectredMessagePayloadType::RequestAddresses,
        SpectredMessagePayloadType::RequestRelayBlocks,
        SpectredMessagePayloadType::RequestTransactions,
        SpectredMessagePayloadType::IbdBlock,
        SpectredMessagePayloadType::InvRelayBlock,
        SpectredMessagePayloadType::InvTransactions,
        SpectredMessagePayloadType::Ping,
        SpectredMessagePayloadType::Pong,
        // SpectredMessagePayloadType::Verack,
        // SpectredMessagePayloadType::Version,
        SpectredMessagePayloadType::TransactionNotFound,
        SpectredMessagePayloadType::Reject,
        SpectredMessagePayloadType::PruningPointUtxoSetChunk,
        SpectredMessagePayloadType::RequestIbdBlocks,
        SpectredMessagePayloadType::UnexpectedPruningPoint,
        SpectredMessagePayloadType::IbdBlockLocator,
        SpectredMessagePayloadType::IbdBlockLocatorHighestHash,
        SpectredMessagePayloadType::RequestNextPruningPointUtxoSetChunk,
        SpectredMessagePayloadType::DonePruningPointUtxoSetChunks,
        SpectredMessagePayloadType::IbdBlockLocatorHighestHashNotFound,
        SpectredMessagePayloadType::BlockWithTrustedData,
        SpectredMessagePayloadType::DoneBlocksWithTrustedData,
        SpectredMessagePayloadType::RequestPruningPointAndItsAnticone,
        SpectredMessagePayloadType::BlockHeaders,
        SpectredMessagePayloadType::RequestNextHeaders,
        SpectredMessagePayloadType::DoneHeaders,
        SpectredMessagePayloadType::RequestPruningPointUtxoSet,
        SpectredMessagePayloadType::RequestHeaders,
        SpectredMessagePayloadType::RequestBlockLocator,
        SpectredMessagePayloadType::PruningPoints,
        SpectredMessagePayloadType::RequestPruningPointProof,
        SpectredMessagePayloadType::PruningPointProof,
        // SpectredMessagePayloadType::Ready,
        SpectredMessagePayloadType::BlockWithTrustedDataV4,
        SpectredMessagePayloadType::TrustedData,
        SpectredMessagePayloadType::RequestIbdChainBlockLocator,
        SpectredMessagePayloadType::IbdChainBlockLocator,
        SpectredMessagePayloadType::RequestAntipast,
        SpectredMessagePayloadType::RequestNextPruningPointAndItsAnticoneBlocks,
    ])
}
