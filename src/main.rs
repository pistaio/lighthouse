#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let username = "polaruser".to_string();
    let password = "polarpass".to_string();
    let rpc_host= "127.0.0.1".to_string();
    let port: u16 = 18443;

    bitcoind_client::connect_to_bitcoin_core(username, password, rpc_host, port).await;
}

mod bitcoind_client {
    use bitcoin::{BlockHash, hashes::hex::FromHex};
    use lightning_block_sync::{http::{HttpEndpoint, JsonResponse}, rpc::RpcClient};

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

    pub async fn connect_to_bitcoin_core(username: String, password: String,
            rpc_host: String, port: u16) {

        let http_endpoint = HttpEndpoint::for_host(rpc_host).with_port(port);
        let credentials_string = format!("{}:{}", username, password);
        let credentials = base64::encode(credentials_string);
        let mut bitcoind_rpc_client = RpcClient::new(&credentials, http_endpoint).unwrap();

        let info = bitcoind_rpc_client.call_method::<BlockchainInfo>("getblockchaininfo", &[]).await.map_err(|_| {
				std::io::Error::new(std::io::ErrorKind::PermissionDenied,
				"Failed to make initial call to bitcoind - please check your RPC user/password and access settings")
			});
        println!("block info: {:#?}", info);
    }
}

