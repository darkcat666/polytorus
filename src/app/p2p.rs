use std::collections::HashSet;
use libp2p::{
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity,
    mdns::{Mdns, MdnsEvent},
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour,
    PeerId,
    Swarm,
};
use once_cell::sync::Lazy;
use tokio::sync::mpsc;

use crate::blockchain::chain::Chain;

pub static KEYS: Lazy<identity::Keypair> = Lazy::new(identity::Keypair::generate_ed25519);
pub static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));
pub static CHAIN_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("chains"));
pub static BLOCK_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("blocks"));

#[derive(Serialize, Deserialize, Debug)]
pub struct ChainResponse {
    pub chain: Chain,
    pub receiver: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalChainRequest {
    pub from_peer_id: String,
}

pub enum Event {
    LocalChainRequest(ChainResponse),
    Input(String),
    Init,
}

#[derive(NetworkBehaviour)]
pub struct ChainBehaviour {
    pub floodsub: Floodsub,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub response_sender: mpsc::UnboundedSender<ChainResponse>,
    #[behaviour(ignore)]
    pub init_sender: mpsc::UnboundedSender<bool>,
    #[behaviour(ignore)]
    pub chain: Chain,
}

impl ChainBehaviour {
    pub async fn new(
        chain: Chain,
        response_sender: mpsc::UnboundedSender<ChainResponse>,
        init_sender: mpsc::UnboundedSender<bool>,
    ) -> Self {
        let mut behaviour = Self {
            chain,
            floodsub: Floodsub::new(*PEER_ID),
            mdns: Mdns::new(Default::default()).await.expect("can create mdns"),
            response_sender,
            init_sender,
        };

        behaviour.floodsub.subscribe(CHAIN_TOPIC.clone());
        behaviour.floodsub.subscribe(BLOCK_TOPIC.clone());

        behaviour
    }
}
