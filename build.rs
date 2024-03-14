fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
    .type_attribute("KvpKey", "#[derive(serde::Deserialize, serde::Serialize)]")
    .type_attribute("KvpPayload", "#[derive(serde::Deserialize, serde::Serialize)]")
    .type_attribute("KvpResponse", "#[derive(serde::Deserialize, serde::Serialize)]")
    .compile(
        &["proto/kvp_store.proto"],
        &["proto"],
    )?;
    
    Ok(())
}