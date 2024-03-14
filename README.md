# Running the Application with Server and Client Crates

This application is divided into two binary crates: `server.rs` and `client.rs`. Below are the instructions to get both the backend service and the client service up and running.

## Starting the Backend Service

To launch the backend service, execute the following command:

```bash
cargo run --bin server 50051 database.db
```

- **`50051`** specifies the port on which the server will run.
- **`database.db`** is the name of the SQLite database file that the server will utilize.

## **Starting the Client Service**

To get the client service running, you need to perform two steps:

### **Step 1: Generate a TLS Certificate**

First, generate a TLS certificate to ensure the client service communicates over HTTPS. Use the following OpenSSL command to create a certificate:

```jsx
openssl req -x509 -newkey rsa:4096 -keyout private/key.pem -out private/cert.pem -days 365 -nodes
```

This command generates a new RSA key and a self-signed certificate valid for 365 days. The files **`key.pem`** and **`cert.pem`** are stored in the **`private`** directory.

### **Step 2: Launch the Client Service**

After generating the TLS certificate, start the client service with the following command:

```bash
cargo run --bin client
```
By default, the client service will start on address 127.0.0.1:8000 and expects the server to be accessible on port 50051.


## **Client API Endpoints**

The client's API exposes two endpoints:

### **1. POST /store_value**

This endpoint expects a key-value payload as the request body. Here is an example request body:

``` json
{
  "key": "test_key",
  "value": "test_value"
}
```

### **2. GET /retrieve_value/`<key>`**

This endpoint retrieves the value corresponding to the specified key. Replace **`<key>`** with the key you want to find. If the key is valid, the request will respond with the corresponding key-value pair.

## **Running Tests**

To run tests, execute the following commands:

```bash
cargo build
```

```bash
cargo test
```

