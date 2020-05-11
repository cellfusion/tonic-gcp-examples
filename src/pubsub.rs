pub mod pubsub {
    tonic::include_proto!("google.pubsub.v1");
}

use pubsub::{publisher_client::PublisherClient, ListTopicsRequest};
use std::env;
use tonic::{
    metadata::MetadataValue,
    transport::{Certificate, Channel, ClientTlsConfig},
    Request,
};
use yup_oauth2;

pub const CERTIFICATES: &[u8] = include_bytes!("../data/gcp/roots.pem");
static API_ENDPOINT: &'static str = "https://pubsub.googleapis.com";
static API_DOMAIN: &'static str = "pubsub.googleapis.com";

fn usage() {
    println!("pubsub PROJECT_NAME");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project_name = match env::args().nth(1) {
        Some(name) => name,
        None => {
            usage();
            return Ok(());
        }
    };

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

    let mut client = PublisherClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut()
            .insert("authorization", header_value.clone());
        Ok(req)
    });

    let project = format!("projects/{}", project_name);
    let response = client
        .list_topics(Request::new(ListTopicsRequest {
            project: project.into(),
            page_size: 10,
            ..Default::default()
        }))
        .await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
