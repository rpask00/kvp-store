use std::process::{Child, Command};
use rusqlite::{Connection, Result};


fn start_server() -> Child {
    Command::new("target/debug/server")
        .arg("50051")
        .arg("client_test.db")
        .spawn()
        .expect("Failed to start server")
}

fn main() {
    let server = start_server();
    
    
    std::thread::sleep(std::time::Duration::from_secs(200));
    
}
