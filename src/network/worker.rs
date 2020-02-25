use super::{behaviour, transport};
use fnv::FnvBuildHasher;
use hashbrown::HashSet;
use libp2p::{Multiaddr, PeerId, Swarm};

/// State machine representing the network currently running.
pub struct Network {
    swarm: Swarm<behaviour::Behaviour>,

    /// List of block requests in progress.
    blocks_requests: HashSet<BlocksRequestId, FnvBuildHasher>,

    /// Identifier to assign to the next blocks request to start.
    next_blocks_request: BlocksRequestId,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BlocksRequestId(u64);

/// Event that can happen on the network.
#[derive(Debug)]
pub enum Event {
    /// Received a block announcement for specific blocks.
    BlocksAnnouncementReceived {
        /// List of encoded headers.
        headers: Vec<Vec<u8>>,
    },

    /// A blocks request started with [`Network::start_block_request`] has gotten a response.
    BlocksRequestFinished {
        result: Result<(), ()>,
    },
}

/// Configuration for starting the network.
///
/// Internal to the `network` module.
pub(super) struct Config {
    pub(super) known_addresses: Vec<(PeerId, Multiaddr)>,
}

impl Network {
    pub(super) async fn start(config: Config) -> Self {
        let local_key_pair = libp2p::identity::Keypair::generate_ed25519();
        let local_public_key = local_key_pair.public();
        let local_peer_id = local_public_key.clone().into_peer_id();
        let (transport, _) = transport::build_transport(local_key_pair, false, None, true);
        let behaviour = behaviour::Behaviour::new(
            "substrate-lite".to_string(),
            local_public_key,
            config.known_addresses,
            true,
            true,
            50,
        )
        .await;
        let swarm = Swarm::new(transport, behaviour, local_peer_id);

        Network {
            swarm,
            blocks_requests: Default::default(),
            next_blocks_request: BlocksRequestId(0),
        }
    }

    /// Sends out an announcement about the given block.
    pub async fn announce_block(&mut self) {
        unimplemented!()
    }

    pub async fn start_block_request(&mut self, block_num: u32) {
        let id = self.next_blocks_request;
        self.next_blocks_request.0 += 1;    // TODO: overflows

        self.blocks_requests.insert(id);

        self.swarm.send_block_request(block_num);
    }

    /// Returns the number of ongoing block requests.
    pub fn num_blocks_request(&self) -> usize {
        self.blocks_requests.len()
    }

    /// Returns the next event that happened on the network.
    pub async fn next_event(&mut self) -> Event {
        loop {
            match self.swarm.next_event().await {
                // TODO: don't println
                ev => println!("{:?}", ev),
            }
        }
    }
}
