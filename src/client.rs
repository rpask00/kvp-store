use log::warn;
use rocket::{get, launch, post, routes, State};
use rocket::response::status::{BadRequest, NotFound};
use rocket::serde::json::{Json};
use tokio::sync::Mutex;
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
) -> Result<Json<KvpResponse>, BadRequest<&str>> {
    let key = kvp_payload_dto.0.key;
    let value = kvp_payload_dto.0.value;

    if key.len() == 0 {
        warn!("Key cannot be empty");
        return Err(BadRequest("Key cannot be empty"));
    }

    if value.len() == 0 {
        warn!("Value cannot be empty");
        return Err(BadRequest("Value cannot be empty"));
    }

    let request = tonic::Request::new(KvpPayload {
        key,
        value,
    });

    return match client.lock().await.store_kvp(request).await {
        Ok(response) => Ok(Json(response.into_inner())),
        Err(e) => Err(BadRequest(e.code().description()))
    };
}

#[get("/retrieve_value/<key>")]
async fn retrieve_value(
    key: String,
    client: &State<Mutex<KvpStoreClient<Channel>>>,
) -> Result<Json<KvpPayload>, BadRequest<&str>> {
    let request = tonic::Request::new(KvpKey {
        key
    });

    return match client.lock().await.get_kvp(request).await {
        Ok(payload) => Ok(Json(payload.into_inner())),
        Err(e) => Err(BadRequest(e.code().description()))
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


