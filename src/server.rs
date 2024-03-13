use tonic::{transport::Server, Request, Response, Status};
pub mod kvp_store {
    tonic::include_proto!("kvp_store");
}

use kvp_store::kvp_store_server::{KvpStore, KvpStoreServer};
use kvp_store::{KvpResponse, KvpPayload, KvpKey};


#[derive(Debug, Default)]
pub struct MyKvpStore {}

#[tonic::async_trait]
impl KvpStore for MyKvpStore {
    async fn store_kvp(
        &self,
        request: Request<KvpPayload>,
    ) -> Result<Response<KvpResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = KvpResponse {
            message: format!("Hello {}!", request.into_inner().key),
        };

        Ok(Response::new(reply))
    }
    
    async fn get_kvp(
        &self,
        request: Request<KvpKey>,
    ) -> Result<Response<KvpPayload>, Status> {
        println!("Got a request: {:?}", request);

        let reply = KvpPayload {
            key: format!("{}!", request.into_inner().key),
            value: "value".into(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let kvp_store = MyKvpStore::default();

    println!("KVP Store server listening on {}", addr);
    
    Server::builder()
        .add_service(KvpStoreServer::new(kvp_store))
        .serve(addr)
        .await?;

    Ok(())
}