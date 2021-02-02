pub mod google {
    pub mod rpc {
        tonic::include_proto!("google.rpc");
    }
    pub mod longrunning {
        tonic::include_proto!("google.longrunning");
    }
    pub mod cloud {
        pub mod speech {
            pub mod v1 {
                tonic::include_proto!("google.cloud.speech.v1");
            }
        }
    }
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

use google::cloud::speech::v1::{
    speech_client::SpeechClient, streaming_recognize_request::StreamingRequest,
    StreamingRecognitionConfig, StreamingRecognizeRequest,
};

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
        .tls_config(tls_config)?
        .connect()
        .await?;

    let mut client = SpeechClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut()
            .insert("authorization", header_value.clone());
        Ok(req)
    });

    let resp = client
        .streaming_recognize(Request::new(StreamingRecognizeRequest {
            streaming_request: Some(StreamingRequest::StreamingConfig(
                StreamingRecognitionConfig {
                    config: None,
                    single_utterance: false,
                    interim_results: false,
                },
            )),
        }))
        .await?;

    let mut reader = hound::WavReader::open("../tts_response.wav").unwrap();
    for sample in reader.samples::<i16>() {}

    Ok(())
}
