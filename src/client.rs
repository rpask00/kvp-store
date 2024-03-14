use log::warn;
use rocket::{get, launch, post, routes, State};
use rocket::response::status::BadRequest;
use rocket::serde::json::Json;
use tokio::sync::Mutex;
use tonic::{Response, Status};
use tonic::transport::Channel;

use kvp_store::{KvpKey, KvpPayload, KvpResponse};
use kvp_store::kvp_store_client::KvpStoreClient;

pub mod kvp_store {
    tonic::include_proto!("kvp_store");
}

#[post("/store_value", data = "<kvp_payload_dto>")]
async fn store_value(
    kvp_payload_dto: Json<KvpPayload>,
    client: &State<Mutex<KvpStoreClient<Channel>>>,
) -> Result<Json<KvpResponse>, BadRequest<String>> {
    let key = kvp_payload_dto.0.key;
    let value = kvp_payload_dto.0.value;

    if key.len() == 0 {
        warn!("Key cannot be empty");
        return Err(BadRequest("Key cannot be empty".to_string()));
    }

    if value.len() == 0 {
        warn!("Value cannot be empty");
        return Err(BadRequest("Value cannot be empty".to_string()));
    }

    let request = tonic::Request::new(KvpPayload {
        key,
        value,
    });

    return match client.lock().await.store_kvp(request).await {
        Ok(response) => Ok(Json(response.into_inner())),
        Err(e) => Err(BadRequest(e.message().to_string()))
    };
}

#[get("/retrieve_value/<key>")]
async fn retrieve_value(
    key: String,
    client: &State<Mutex<KvpStoreClient<Channel>>>,
) -> Result<Json<KvpPayload>, BadRequest<String>> {
    if key.len() == 0 {
        warn!("Cannot retrieve value for empty key");
        return Err(BadRequest("Key cannot be empty".to_string()));
    }

    let request = tonic::Request::new(KvpKey {
        key
    });

    return match client.lock().await.get_kvp(request).await {
        Ok(payload) => Ok(Json(payload.into_inner())),
        Err(e) => Err(BadRequest(e.message().to_string()))
    };
}


#[launch]
async fn rocket() -> _ {
    let channel = Channel::from_static("http://0.0.0.0:50051")
        .connect()
        .await.unwrap();


    let mut client = KvpStoreClient::new(channel);
    rocket::build().mount("/", routes![store_value, retrieve_value]).manage(Mutex::new(client))
}


