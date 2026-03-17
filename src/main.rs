use anyhow::{anyhow, Result};
use blsttc::SecretKey;
use chrono::{Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub mod archive {
    tonic::include_proto!("archive");
}
pub mod tarchive {
    tonic::include_proto!("tarchive");
}
pub mod pnr {
    tonic::include_proto!("pnr");
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    name: String,

    #[arg(short, long)]
    url: Option<String>,

    #[arg(short, long)]
    email: Option<String>,

    #[arg(short, long)]
    private_key: Option<String>,

    #[arg(long, default_value = "disk")]
    store_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Profile {
    #[serde(rename = "@context")]
    context: String,
    #[serde(rename = "@type")]
    type_: String,
    #[serde(rename = "@id")]
    id: String,
    name: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    identifier: Vec<PropertyValue>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PropertyValue {
    #[serde(rename = "@type")]
    type_: String,
    #[serde(rename = "propertyID")]
    property_id: String,
    value: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct PublicKeyDoc {
    #[serde(rename = "type")]
    type_: String,
    format: String,
    curve: String,
    encoding: String,
    #[serde(rename = "publicKey")]
    public_key: String,
    created: String,
    id: String,
    fingerprints: HashMap<String, String>,
}

fn validate_email(email: &str) -> Result<()> {
    if !email.contains('@') {
        return Err(anyhow!(
            "Error: Invalid field 'email'\nCause: The email '{}' is not a valid mailto URI.\nSuggestion: Ensure the email follows the format 'user@example.com'.",
            email
        ));
    }
    Ok(())
}

fn validate_url(url: &str) -> Result<()> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(anyhow!(
            "Error: Invalid field 'url'\nCause: '{}' is not a valid absolute URL.\nSuggestion: Use a full URL starting with http:// or https://.",
            url
        ));
    }
    Ok(())
}

fn get_pnr_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect();
    let cleaned = sanitized.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");
    let timestamp = Utc::now().timestamp();
    format!("{}-profile-{}", cleaned, timestamp)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(ref url) = args.url {
        validate_url(url)?;
    }
    if let Some(ref email) = args.email {
        validate_email(email)?;
    }

    let secret_key = if let Some(pk_hex) = args.private_key {
        let bytes = hex::decode(&pk_hex).map_err(|_| {
            anyhow!(
                "Error: Invalid Private Key\nCause: Provided key is not a valid hex-encoded string.\nSuggestion: Provide a 64-character hex string or omit the parameter to auto-generate a key."
            )
        })?;
        if bytes.len() != 32 {
            return Err(anyhow!("Error: Invalid Private Key\nCause: Provided key is not 32 bytes (64 hex characters).\nSuggestion: Provide a 64-character hex string."));
        }
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes);
        SecretKey::from_bytes(key_bytes).map_err(|_| anyhow!("Error: Invalid Private Key\nCause: Failed to parse SecretKey from bytes."))?
    } else {
        SecretKey::random()
    };

    let public_key = secret_key.public_key();
    let pk_bytes = public_key.to_bytes();
    let pk_base64 = base64_encode(&pk_bytes);
    let mut hasher = Sha256::new();
    hasher.update(&pk_bytes);
    let pk_fingerprint = hex::encode(hasher.finalize());

    let now = Utc::now();
    let pnr_name = get_pnr_name(&args.name);
    let base_url = format!("http://{}", pnr_name);
    let profile_url = format!("{}/profile.jsonld", base_url);
    let key_doc_url = format!("{}/keys/blsttc/{}/public-key.json", base_url, now.format("%Y-%m-%d"));

    let pk_doc = PublicKeyDoc {
        type_: "blsttc-public-key".to_string(),
        format: "blsttc".to_string(),
        curve: "BLS12-381".to_string(),
        encoding: "base64".to_string(),
        public_key: pk_base64,
        created: now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        id: key_doc_url.clone(),
        fingerprints: {
            let mut m = HashMap::new();
            m.insert("sha256".to_string(), pk_fingerprint.clone());
            m
        },
    };

    let profile = Profile {
        context: "https://schema.org".to_string(),
        type_: "Person".to_string(),
        id: format!("{}#me", profile_url),
        name: args.name,
        url: args.url.unwrap_or_else(|| base_url.clone()),
        email: args.email.map(|e| format!("mailto:{}", e)),
        identifier: vec![
            PropertyValue {
                type_: "PropertyValue".to_string(),
                property_id: "blsttc:public-key".to_string(),
                value: key_doc_url,
            },
            PropertyValue {
                type_: "PropertyValue".to_string(),
                property_id: "blsttc:public-key-fingerprint".to_string(),
                value: format!("sha256:{}", pk_fingerprint),
            },
        ],
    };

    let profile_json = serde_json::to_string_pretty(&profile)?;
    let pk_doc_json = serde_json::to_string_pretty(&pk_doc)?;

    let mut tarchive_client = tarchive::tarchive_service_client::TarchiveServiceClient::connect("http://localhost:18887").await
        .map_err(|_| anyhow!("Error: Failed to persist AntID\nCause: Could not connect to AntTP gRPC API at localhost:18887.\nSuggestion: Ensure the AntTP service is running and accessible."))?;

    let tarchive_request_root = tonic::Request::new(tarchive::CreateTarchiveRequest {
        files: vec![
            tarchive::File {
                name: "profile.jsonld".to_string(),
                content: profile_json.as_bytes().to_vec(),
            },
        ],
        path: None,
        store_type: Some(args.store_type.clone()),
    });

    let tarchive_response_root = tarchive_client.create_tarchive(tarchive_request_root).await
        .map_err(|e| anyhow!("Error: Failed to create tarchive (root)\nCause: {}\nSuggestion: Check AntTP logs.", e))?;
    
    let initial_address = tarchive_response_root.into_inner().address.ok_or_else(|| anyhow!("Error: Failed to create tarchive\nCause: Server returned empty address."))?;

    let key_dir = format!("keys/blsttc/{}", now.format("%Y-%m-%d"));
    let tarchive_request_keys = tonic::Request::new(tarchive::UpdateTarchiveRequest {
        address: initial_address,
        files: vec![
            tarchive::File {
                name: "public-key.json".to_string(),
                content: pk_doc_json.as_bytes().to_vec(),
            },
        ],
        path: Some(key_dir),
        store_type: Some(args.store_type.clone()),
    });

    let tarchive_response_final = tarchive_client.update_tarchive(tarchive_request_keys).await
        .map_err(|e| anyhow!("Error: Failed to update tarchive with keys\nCause: {}\nSuggestion: Check AntTP logs.", e))?;

    let tarchive_address = tarchive_response_final.into_inner().address.ok_or_else(|| anyhow!("Error: Failed to update tarchive\nCause: Server returned empty address."))?;

    let mut pnr_client = pnr::pnr_service_client::PnrServiceClient::connect("http://localhost:18887").await
        .map_err(|_| anyhow!("Error: Failed to register PNR\nCause: Could not connect to AntTP gRPC API at localhost:18887.\nSuggestion: Ensure the AntTP service is running and accessible."))?;

    let mut records = HashMap::new();
    records.insert("".to_string(), pnr::PnrRecord {
        address: tarchive_address.clone(),
        record_type: pnr::PnrRecordType::A.into(),
        ttl: 3600,
    });

    let pnr_request = tonic::Request::new(pnr::CreatePnrRequest {
        pnr_zone: Some(pnr::PnrZone {
            name: pnr_name,
            records,
            resolver_address: None,
            personal_address: None,
        }),
        store_type: Some(args.store_type.clone()),
        is_immutable: true,
    });

    pnr_client.create_pnr(pnr_request).await
        .map_err(|e| anyhow!("Error: Failed to register PNR\nCause: {}\nSuggestion: Check AntTP logs.", e))?;

    println!("AntID generated successfully!");
    println!("\n--- profile.jsonld ---\n{}", profile_json);
    println!("\n--- public-key.json ---\n{}", pk_doc_json);

    println!("\nImmutable Address: {} ({})", tarchive_address, args.store_type);

    Ok(())
}

fn base64_encode(bytes: &[u8]) -> String {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pnr_name_generation() {
        let name = "Joe Blogs";
        let pnr = get_pnr_name(name);
        assert!(pnr.starts_with("joe-blogs-profile-"));
    }

    #[test]
    fn test_email_validation() {
        assert!(validate_email("ada@example.com").is_ok());
        assert!(validate_email("invalid-email").is_err());
    }

    #[test]
    fn test_url_validation() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("ftp://example.com").is_err());
    }

    #[test]
    fn test_get_pnr_name_with_spaces() {
        let name = "  Joe   Blogs  ";
        let pnr = get_pnr_name(name);
        assert!(pnr.starts_with("joe-blogs-profile-"));
    }
}
