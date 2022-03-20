use bitcoin::{BlockHash, hashes::hex::FromHex};
use lightning_block_sync::{http::{HttpEndpoint, JsonResponse}, rpc::RpcClient};

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let username = "polaruser".to_string();
    let password = "polarpass".to_string();
    let rpc_host= "127.0.0.1".to_string();
    let port: u16 = 18443;
    let network_chain = "regtest".to_string();

    let bitcoind_client = BitcoinClient::connect_to_bitcoin_node(username, password, rpc_host, port);
    assert_eq!(bitcoind_client.getblockchaininfo().await.chain, network_chain);
    println!("i guess it worked? i didn't specify the chain anywhere tho");
}

struct BitcoinClient {
    client: RpcClient
}

impl BitcoinClient {
    pub fn connect_to_bitcoin_node(username: String, password: String,
            rpc_host: String, port: u16) -> BitcoinClient {
        let http_endpoint = HttpEndpoint::for_host(rpc_host.clone()).with_port(port);
        let credentials_string = format!("{}:{}", username, password);
        let credentials = base64::encode(credentials_string);
        let bitcoind_rpc_client = RpcClient::new(&credentials, http_endpoint).unwrap();
        let client = BitcoinClient {
            client: bitcoind_rpc_client
        };
        return client 
}

    pub async fn getblockchaininfo(mut self) -> BlockchainInfo {
        let info = self.client.call_method::<BlockchainInfo>("getblockchaininfo", &[]).await.map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::PermissionDenied,
                    "Failed to make initial call to bitcoind - please check your RPC user/password and access settings")
            }).expect("Some error");
        // println!("block info: {:#?}", info);
        return info
    }
}

#[derive(Debug)]
pub struct BlockchainInfo {
    pub latest_height: usize,
    pub latest_blockhash: BlockHash,
    pub chain: String,
}

impl TryInto<BlockchainInfo> for JsonResponse {
    type Error = std::io::Error;
    fn try_into(self) -> std::io::Result<BlockchainInfo> {
        Ok(BlockchainInfo {
            latest_height: self.0["blocks"].as_u64().unwrap() as usize,
            latest_blockhash: BlockHash::from_hex(self.0["bestblockhash"].as_str().unwrap())
                .unwrap(),
            chain: self.0["chain"].as_str().unwrap().to_string(),
        })
    }
}

