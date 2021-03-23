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

use google::cloud::speech::v1::{
    recognition_audio::AudioSource, recognition_config::AudioEncoding, speech_client::SpeechClient,
    RecognitionAudio, RecognitionConfig, RecognizeRequest, RecognizeResponse, SpeechContext,
};
use std::{
    fs::File,
    io::{BufWriter, Read, Write},
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
        .tls_config(tls_config)?
        .connect()
        .await?;

    let mut client = SpeechClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut()
            .insert("authorization", header_value.clone());
        Ok(req)
    });

    let mut file = File::open("./tts_response.wav")?;
    let mut buf = Vec::<u8>::new();
    file.read_to_end(&mut buf)?;

    let request = Request::new(RecognizeRequest {
        config: Some(RecognitionConfig {
            audio_channel_count: 1,
            diarization_config: None,
            enable_automatic_punctuation: false,
            enable_separate_recognition_per_channel: false,
            enable_word_time_offsets: false,
            use_enhanced: false,
            language_code: "ja-JP".to_string(),
            max_alternatives: 1,
            encoding: AudioEncoding::Linear16.into(),
            model: "default".to_string(),
            metadata: None,
            profanity_filter: false,
            sample_rate_hertz: 16000,
            speech_contexts: Vec::<SpeechContext>::new(),
        }),
        audio: Some(RecognitionAudio {
            audio_source: Some(AudioSource::Content(buf)),
        }),
    });

    let resp = client.recognize(request).await?;

    println!("response:{:?}", resp);

    Ok(())
}
