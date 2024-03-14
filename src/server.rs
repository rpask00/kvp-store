use std::sync::{Arc};
use rusqlite::{Connection, Result};
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

pub mod kvp_store {
    tonic::include_proto!("kvp_store");
}

use kvp_store::kvp_store_server::{KvpStore, KvpStoreServer};
use kvp_store::{KvpResponse, KvpPayload, KvpKey};

pub struct MyKvpStore {
    db_conn: Arc<Mutex<Connection>>,
}

impl MyKvpStore {
    pub fn init(db_file_name: &str) -> Result<Self> {
        let db_conn = Connection::open(db_file_name)?;

        let table_exists = db_conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?")?.exists(["key_value_pairs"])?;

        if !table_exists {
            db_conn.execute(
                "CREATE TABLE key_value_pairs (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                )",
                (),
            )?;
        }

        Ok(MyKvpStore {
            db_conn: Arc::new(Mutex::new(db_conn)),
        })
    }
}


#[tonic::async_trait]
impl KvpStore for MyKvpStore {
    async fn store_kvp(&self, request: Request<KvpPayload>) -> Result<Response<KvpResponse>, Status> {
        let kvp_payload = request.into_inner();

        let response_message = match self.db_conn.lock().await.execute(
            "INSERT OR REPLACE INTO key_value_pairs (key, value) VALUES (?1, ?2)",
            (&kvp_payload.key, &kvp_payload.value),
        ) {
            Ok(_) => format!("Value {} stored successfully for key {}", kvp_payload.value, kvp_payload.key),
            Err(e) => e.to_string(),
        };


        Ok(Response::new(KvpResponse {
            message: response_message
        }))
    }

    async fn get_kvp(&self, request: Request<KvpKey>) -> Result<Response<KvpPayload>, Status> {
        let query = self.db_conn.lock().await.query_row(
            "SELECT key, value FROM key_value_pairs WHERE key = ?1",
            &[&request.into_inner().key],
            |row| {
                Ok(KvpPayload {
                    key: row.get(0)?,
                    value: row.get(1)?,
                })
            },
        );

        match query {
            Ok(payload) => Ok(Response::new(payload)),
            Err(e) => Err(Status::not_found(e.to_string())),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: <port> <database.db>");
        std::process::exit(1);
    }

    let port = args[1].parse::<u16>()?;
    let db_file = args[2].as_str();

    let addr = format!("0.0.0.0:{}", port).parse()?;
    let kvp_store = MyKvpStore::init(db_file)?;


    println!("KVP Store server listening on {}", addr);

    Server::builder()
        .add_service(KvpStoreServer::new(kvp_store))
        .serve(addr)
        .await?;

    Ok(())
}


#[tokio::test]
async fn test_store_kvp() {
    let kvp_store = MyKvpStore::init("test_store_kvp.db");

    if let Ok(kvp_store) = kvp_store {
        let request = Request::new(KvpPayload {
            key: "test_key".to_string(),
            value: "test_value".to_string(),
        });

        let response = kvp_store.store_kvp(request).await;
        assert_eq!(response.is_ok(), true, "Failed to store key-value pair");

        let response = response.unwrap();

        assert_eq!(response.into_inner().message, "Value test_value stored successfully for key test_key");
    } else {
        assert!(false, "Database connection failed");
    }

    std::fs::remove_file("test_store_kvp.db").unwrap();
}

#[tokio::test]
async fn test_get_kvp() {
    let kvp_store = MyKvpStore::init("test_get_kvp.db");

    if let Ok(kvp_store) = kvp_store {
        let request = Request::new(KvpPayload {
            key: "test_key".to_string(),
            value: "test_value".to_string(),
        });

        let response = kvp_store.store_kvp(request).await;
        assert_eq!(response.is_ok(), true, "Failed to store key-value pair");

        let request = Request::new(KvpKey {
            key: "test_key".to_string(),
        });

        if let Ok(response) = kvp_store.get_kvp(request).await {
            let response = response.into_inner();
            assert_eq!(response.key, "test_key", "test_key not matched");
            assert_eq!(response.value, "test_value", "test_value not matched");
        } else {
            assert!(false, "Failed to retrieve key-value pair");
        }
    } else {
        assert!(false, "Database connection failed");
    }
    std::fs::remove_file("test_get_kvp.db").unwrap();
}

