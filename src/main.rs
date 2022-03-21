use std::{time::SystemTime, io::Error};

use bitcoin::Block;
use lightning::chain::keysinterface::{KeysManager, KeysInterface, Recipient};
use lightning_block_sync::{http::{HttpEndpoint, JsonResponse}, rpc::RpcClient, BlockSource, AsyncBlockSourceResult, BlockHeaderData};

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let username = "polaruser".to_string();
    let password = "polarpass".to_string();
    let rpc_host= "127.0.0.1".to_string();
    let port: u16 = 18443;
    let network_chain = "regtest".to_string();

    let mut bitcoind_client = BitcoinClient::connect_to_bitcoin_node(username, password, rpc_host, port);
    let blockchain_info = bitcoind_client.get_blockchain_info().await;
    assert_eq!(blockchain_info.chain, network_chain);
    println!("number of blocks: {}", blockchain_info.blocks);
    println!("i guess it worked? i didn't specify the chain anywhere tho");
    let sk = generate_secret_key();
    println!("secret key: {}", sk);
}

fn generate_secret_key() -> String {
    // TODO: save key_seed to file
    let key_seed_path = ".lighthouse/keys_seed";
    let key_seed: [u8; 32] = [0; 32];
    let cur = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let keys_manager = KeysManager::new(&key_seed, cur.as_secs(), cur.subsec_micros());
    let secret_key = keys_manager.get_node_secret(Recipient::Node).unwrap();
    return secret_key.to_string()
}



// ------- BitcoinClient ------- 

struct BitcoinClient {
    client: RpcClient
}

impl BitcoinClient {
    pub fn connect_to_bitcoin_node(username: String, password: String,
            rpc_host: String, port: u16) -> BitcoinClient {
        let http_endpoint = HttpEndpoint::for_host(rpc_host.clone()).with_port(port);
        let credentials_string = format!("{}:{}", username, password);
        let credentials = base64::encode(credentials_string);
        let bitcoind_rpc_client = RpcClient::new(&credentials, http_endpoint)
            .map_err(|_| {
                Error::new(std::io::ErrorKind::ConnectionRefused, "Bitcoind refused the connection")
            })
            .unwrap();
        let client = BitcoinClient {
            client: bitcoind_rpc_client
        };
        return client 
}

    pub async fn get_blockchain_info<'a>(&'a mut self) -> BlockchainInfo {
        match self.client.call_method::<BlockchainInfo>("getblockchaininfo", &[]).await {
            Ok(result) => result,
            _ => panic!("Something whent wrong")
        }
    }
}

struct BlockchainInfo {
    chain: String,
    blocks: u64,
    _best_block_hash: String,
}

impl TryInto<BlockchainInfo> for JsonResponse {
    type Error = std::io::Error; 
    fn try_into(self) -> Result<BlockchainInfo, Self::Error> {
        Ok(BlockchainInfo {
            chain: self.0["chain"].as_str().unwrap().to_string(),
            blocks: self.0["blocks"].as_u64().unwrap(),
            _best_block_hash: self.0["bestblockhash"].as_str().unwrap().to_string()
        })
    }
}
