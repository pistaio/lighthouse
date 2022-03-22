use std::{time::SystemTime, io::Error, str::FromStr, net::{SocketAddr, Ipv4Addr}, sync::Arc, ops::Deref};

use bitcoin::BlockHash;
use bitcoin::blockdata::constants::genesis_block;
use bitcoin::{Block, secp256k1::PublicKey, Transaction};
use chrono::Utc;
use lightning::chain::BestBlock;
use lightning::chain::Filter;
use lightning::chain::chainmonitor;
use lightning::chain::keysinterface::InMemorySigner;
use lightning::ln::channelmanager::ChainParameters;
use lightning::ln::channelmanager::ChannelManager;
use lightning::ln::peer_handler;
use lightning::ln::peer_handler::IgnoringMessageHandler;
use lightning::ln::peer_handler::MessageHandler;
use lightning::ln::peer_handler::SimpleArcPeerManager;
use lightning::routing::network_graph::NetworkGraph;
use lightning::util::config::UserConfig;
use lightning::{chain::{keysinterface::{KeysManager, KeysInterface, Recipient, Sign}, self, chaininterface::{BroadcasterInterface, FeeEstimator, ConfirmationTarget}}, routing::network_graph::NetGraphMsgHandler, util::logger::{Logger, Record}};
use lightning_block_sync::{http::{HttpEndpoint, JsonResponse}, rpc::RpcClient, BlockSource, AsyncBlockSourceResult, BlockHeaderData};
use lightning_net_tokio::SocketDescriptor;
use lightning_persister::FilesystemPersister;
use rand::RngCore;
use tokio::sync::Mutex;

type CustomChainMonitor = chainmonitor::ChainMonitor<
    InMemorySigner,
	Arc<dyn Filter + Send + Sync>,
	Arc<BitcoinClient>,
	Arc<BitcoinClient>,
	Arc<CustomLogger>,
	Arc<FilesystemPersister>,
>;

type PeerManager = SimpleArcPeerManager<
	SocketDescriptor,
	CustomChainMonitor,
	BitcoinClient,
	BitcoinClient,
	dyn chain::Access + Send + Sync,
	CustomLogger,
>;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let username = "polaruser".to_string();
    let password = "polarpass".to_string();
    let rpc_host= "127.0.0.1".to_string();
    let port: u16 = 18443;
    let network_chain = "regtest".to_string();

    // Step 1
    let mut bitcoind_client = BitcoinClient::connect_to_bitcoin_node(username, password, rpc_host, port);
    let blockchain_info = bitcoind_client.get_blockchain_info().await;
    assert_eq!(blockchain_info.chain, network_chain);
    println!("number of blocks: {}", blockchain_info.blocks);
    println!("i guess it worked? i didn't specify the chain anywhere tho");
    setup_ldk(bitcoind_client).await;
}

async fn setup_ldk(mut bitcoind_client: BitcoinClient) {
    // Step 2
    let keys_manager = Arc::new(create_keys_manager());
    // println!("secret key: {}", sk);
    // Step 6
    let fee_estimator = Arc::new(bitcoind_client.clone());
    // Step 8
    let broadcaster = Arc::new(bitcoind_client.clone());
    // Step 9
    let logger = Arc::new(CustomLogger{});
    // Step 10
    let persister = Arc::new(FilesystemPersister::new("".to_string()));
    // Step 11
    let config = UserConfig::default();
    // Step 12 - create chain_params
    let blockchain_info = bitcoind_client.get_blockchain_info().await;
    let best_block_hash = blockchain_info.best_block_hash;
    let height = blockchain_info.blocks;
    let chain_params = ChainParameters {
        network: bitcoin::Network::Regtest,
        best_block: BestBlock::new(best_block_hash, height),
    };

    // Step 7
    let chain_monitor: Arc<CustomChainMonitor> = Arc::new(chainmonitor::ChainMonitor::new(
            None, // FIXME: why is this none?
            broadcaster.clone(), 
            logger.clone(), 
            fee_estimator.clone(), 
            persister));

    // Step 5
    let channel_manager = Arc::new(ChannelManager::new(
            fee_estimator.clone(), 
            chain_monitor.clone(), 
            broadcaster.clone(), 
            logger.clone(), 
            keys_manager.clone(), 
            config, 
            chain_params));

    // Step 13 - create route_handler
    let genesis_hash = genesis_block(bitcoin::Network::Regtest).block_hash();
    let network_graph = Arc::new(NetworkGraph::new(genesis_hash));
    let network_gossip: Arc<NetGraphMsgHandler<
        Arc<NetworkGraph>,
        Arc<dyn chain::Access + Send + Sync>,
        Arc<CustomLogger>
    >> = Arc::new(NetGraphMsgHandler::new(network_graph, None, logger.clone()));

    // Step 4
    let message_handler = MessageHandler {
        chan_handler: channel_manager,
        route_handler: network_gossip,
    };


    println!("trying to connect peer now");

    // Step 3 - connect to peer
    let peer_node_id = PublicKey::from_str("036d8910820847acc4da58cf595f9f1d5ce5dd7f7efc0b63ccce14fc8e85ff0403").unwrap();
    // Going full crazy building this socket address
    let peer_address = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9836);
    let sk = keys_manager.get_node_secret(Recipient::Node).unwrap();
    let mut ephemeral_bytes = [0; 32];
	rand::thread_rng().fill_bytes(&mut ephemeral_bytes);
    let peer_manager: Arc<PeerManager>  = Arc::new(peer_handler::PeerManager::new(
            message_handler, 
            sk, 
            &ephemeral_bytes, 
            logger.clone(), 
            Arc::new(IgnoringMessageHandler {})));
    tokio::spawn(async move {
            lightning_net_tokio::connect_outbound(peer_manager.clone(), peer_node_id, peer_address)
            .await;
            println!("Something happened");
            // lightning_net_tokio::setup_inbound(peer_mgr.clone(), tcp_stream.into_std().unwrap())
            // .await;
            // });
    });
}

// https://lightningdevkit.org/key_management/
fn create_keys_manager() -> KeysManager {
    // TODO: save key_seed to file
    let key_seed_path = ".lighthouse/keys_seed";
    let mut key_seed: [u8; 32] = [0; 32];
    rand::thread_rng().try_fill_bytes(&mut key_seed).unwrap(); 
    let cur = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let keys_manager = KeysManager::new(&key_seed, cur.as_secs(), cur.subsec_micros());
    return keys_manager 
}

fn connect_to_peer(keys_manager: KeysManager) {
}


// ------- BitcoinClient ------- 

#[derive(Clone)]
// Need to use Arc for clone, among other reasons I'm not entirely sure of yet.
// Can't implement clone for RpcClient since it isn't defined in this crate.
// Need to add Mutex to give client mut properties.
// This just seems like a clusterfuck. I wanted a very simple single level struct like this.
// Mostly I feel it's because I'm new to Rust.
// struct BitcoinClient {
//     client: RpcClient 
// }

struct BitcoinClient {
    client: Arc<Mutex<RpcClient>>
}


impl BitcoinClient {
    pub fn connect_to_bitcoin_node(username: String, password: String,
            rpc_host: String, port: u16) -> BitcoinClient {
        let http_endpoint = HttpEndpoint::for_host(rpc_host.clone()).with_port(port);
        let credentials_string = format!("{}:{}", username, password);
        let credentials = base64::encode(credentials_string);
        let bitcoind_rpc_client = Arc::new(Mutex::new(RpcClient::new(&credentials, http_endpoint)
            .map_err(|_| {
                Error::new(std::io::ErrorKind::ConnectionRefused, "Bitcoind refused the connection")
            })
            .unwrap()));
        let client = BitcoinClient {
            client: bitcoind_rpc_client
        };
        return client 
}

    pub async fn get_blockchain_info<'a>(&'a mut self) -> BlockchainInfo {
        let mut rpc = self.client.lock().await;
        match rpc.call_method::<BlockchainInfo>("getblockchaininfo", &[]).await {
            Ok(result) => result,
            _ => panic!("Something whent wrong")
        }
    }
}

impl FeeEstimator for BitcoinClient {
	fn get_est_sat_per_1000_weight(&self, confirmation_target: ConfirmationTarget) -> u32 {
        match confirmation_target {
            ConfirmationTarget::Background => 100,
            ConfirmationTarget::Normal => 100,
            ConfirmationTarget::HighPriority => 100,
        }
    }
}

impl BroadcasterInterface for BitcoinClient {
	fn broadcast_transaction(&self, tx: &Transaction) {
        todo!()
    }
}

// impl Deref for BitcoinClient {
//     type Target = Arc<RpcClient>;

//     fn deref(&self) -> &Self::Target {
//         &self.client
//     }
// }

struct BlockchainInfo {
    chain: String,
    blocks: u32,
    best_block_hash: BlockHash,
}

impl TryInto<BlockchainInfo> for JsonResponse {
    type Error = std::io::Error; 
    fn try_into(self) -> Result<BlockchainInfo, Self::Error> {
        Ok(BlockchainInfo {
            chain: self.0["chain"].as_str().unwrap().to_string(),
            blocks: self.0["blocks"].as_u64().unwrap() as u32,
            best_block_hash: BlockHash::from_str(self.0["bestblockhash"].as_str().unwrap()).unwrap()
        })
    }
}
// ----------- Standard output logger (temp) -------------

#[derive(Clone)]
struct CustomLogger {}

impl Logger for CustomLogger {
	fn log(&self, record: &Record) {
        println!("{} {:<5} [{}:{}]\n",
			Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
			record.level.to_string(),
			record.module_path,
			record.line);
    }
}

// impl Deref for CustomLogger {
//     type Target = None;

//     fn deref(&self) -> &Self::Target {
//         None
//     }
// }
