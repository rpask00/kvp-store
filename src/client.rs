use std::process::{Child, Command};
use log::warn;
use rocket::{get, launch, post, routes, State};
use rocket::response::status::{BadRequest, NotFound};
use rocket::serde::json::{Json};
use tokio::sync::Mutex;
use tonic::transport::Channel;
use kvp_store::{KvpKey, KvpPayload, KvpResponse};
use kvp_store::kvp_store_client::KvpStoreClient;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::Client;
use tokio::time::sleep;


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


fn start_server(port: &str, db_file: &str) -> Child {
    Command::new("target/debug/server")
        .arg(port)
        .arg(db_file)
        .spawn()
        .expect("Failed to start server")
}

#[tokio::test]
async fn test_store_value_success() {
    let server_port = "3201";
    let db_file = "client_test1.db";
    let server_address = "http://0.0.0.0:3201";
    let mut server_process = start_server(server_port, db_file);

    sleep(std::time::Duration::from_secs(3)).await;

    let channel = Channel::from_static(server_address)
        .connect()
        .await.unwrap();

    let client = KvpStoreClient::new(channel);

    let rocket = rocket::build().mount("/", routes![store_value])
        .manage(Mutex::new(client));

    let client = Client::tracked(rocket).await.expect("valid rocket instance");

    let response = client
        .post("/store_value")
        .header(ContentType::JSON)
        .body(r#"{"key": "test_key", "value": "test_value"}"#)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    server_process.kill().unwrap();
    tokio::fs::remove_file(db_file).await.unwrap();
}

#[tokio::test]
async fn test_retrieve_value_success() {
    let server_port = "3202";
    let db_file = "client_test2.db";
    let server_address = "http://0.0.0.0:3202";
    let mut server_process = start_server(server_port, db_file);

    sleep(std::time::Duration::from_secs(3)).await;

    let channel = Channel::from_static(server_address)
        .connect()
        .await.unwrap();

    let client = KvpStoreClient::new(channel);

    let rocket = rocket::build().mount("/", routes![store_value, retrieve_value])
        .manage(Mutex::new(client));

    let client = Client::tracked(rocket).await.expect("valid rocket instance");

    client
        .post("/store_value")
        .header(ContentType::JSON)
        .body(r#"{"key": "test_key", "value": "test_value"}"#)
        .dispatch()
        .await;

    let response = client
        .get("/retrieve_value/test_key")
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    server_process.kill().unwrap();
    tokio::fs::remove_file(db_file).await.unwrap();
}


