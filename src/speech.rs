pub mod stt {
    tonic::include_proto!("google.cloud.speech.v1");
}

use std::{
    fs::File,
    io::{BufWriter, Write},
};
use tonic::{
    metadata::MetadataValue,
    transport::{Certificate, Channel, ClientTlsConfig},
    Request,
};
use yup_oauth2;

pub const CERTIFICATES: &[u8] = include_bytes!("../data/gcp/roots.pem");
static API_DOMAIN: &'static str = "speech.googleapis.com";
static API_ENDPOINT: &'static str = "https://speech.googleapis.com";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sa_key = yup_oauth2::read_service_account_key("gcp_sa_key.json").await?;
    let auth = yup_oauth2::ServiceAccountAuthenticator::builder(sa_key)
        .build()
        .await?;

    let token = auth
        .token(&["https://www.googleapis.com/auth/cloud-platform"])
        .await?;
    let bearer_token = format!("Bearer {}", token.as_str());
    let header_value = MetadataValue::from_str(&bearer_token)?;

    let tls_config = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(CERTIFICATES))
        .domain_name(API_DOMAIN);

    let channel = Channel::from_static(API_ENDPOINT)
        .tls_config(tls_config)
        .connect()
        .await?;

    Ok(())
}
