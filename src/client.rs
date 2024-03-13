pub mod kvp_store {
    tonic::include_proto!("kvp_store");
}

use std::sync::{Arc};
use rocket::{get, launch, post, routes, State};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tonic::transport::Channel;
use kvp_store::kvp_store_client::KvpStoreClient;
use kvp_store::{KvpKey, KvpPayload, KvpResponse};


#[derive(Deserialize, Serialize)]
struct KvpPayloadDto {
    key: String,
    value: String,
}

#[derive(Deserialize, Serialize)]
struct KvpResponseDto {
    message: String,
}

#[post("/store_value", data = "<kvp_payload_dto>")]
async fn store_value(
    kvp_payload_dto: Json<KvpPayloadDto>,
    client: &State<Mutex<KvpStoreClient<Channel>>>,
) -> Json<KvpResponseDto> {
    let request = tonic::Request::new(KvpPayload {
        key: kvp_payload_dto.0.key,
        value: kvp_payload_dto.0.value,
    });

    let lock = client.lock().await.store_kvp(request).await;


    Json(KvpResponseDto {
        message: lock.unwrap().into_inner().message
    })
}

#[get("/retrieve_value/<key>")]
async fn retrieve_value(
    key: String,
    client: &State<Mutex<KvpStoreClient<Channel>>>,
) -> Json<KvpPayloadDto> {
    let request = tonic::Request::new(KvpKey {
        key
    });

    let kvp = client.lock().await.get_kvp(request).await.unwrap().into_inner();

    Json(KvpPayloadDto {
        key: kvp.key,
        value: kvp.value,
    })
}


#[launch]
async fn rocket() -> _ {
    let channel = Channel::from_static("http://0.0.0.0:50051")
        .connect()
        .await.unwrap();


    let mut client = KvpStoreClient::new(channel);
    rocket::build().mount("/", routes![store_value, retrieve_value]).manage(Mutex::new(client))
}


