pub mod tts {
    tonic::include_proto!("google.cloud.texttospeech.v1");
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
use tts::{
    synthesis_input::InputSource, text_to_speech_client::TextToSpeechClient, AudioConfig,
    AudioEncoding, SsmlVoiceGender, SynthesisInput, SynthesizeSpeechRequest, VoiceSelectionParams,
};
use yup_oauth2;

pub const CERTIFICATES: &[u8] = include_bytes!("../data/gcp/roots.pem");
static API_DOMAIN: &'static str = "texttospeech.googleapis.com";
static API_ENDPOINT: &'static str = "https://texttospeech.googleapis.com";

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

    let mut client = TextToSpeechClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut()
            .insert("authorization", header_value.clone());
        Ok(req)
    });

    let response = client
        .synthesize_speech(Request::new(SynthesizeSpeechRequest {
            audio_config: Some(AudioConfig {
                audio_encoding: AudioEncoding::Linear16 as i32,
                speaking_rate: 0f64,
                pitch: 0f64,
                volume_gain_db: 0f64,
                sample_rate_hertz: 16000,
                effects_profile_id: vec![],
            }),
            voice: Some(VoiceSelectionParams {
                language_code: "ja-JP".into(),
                name: "ja-JP-Wavenet-A".into(),
                ssml_gender: SsmlVoiceGender::Female as i32,
            }),
            input: Some(SynthesisInput {
                input_source: Some(InputSource::Text("おはよう".into())),
            }),
        }))
        .await?;

    let response = response.into_inner();

    let mut writer = BufWriter::new(File::create("tts_response.wav").unwrap());
    writer.write(&response.audio_content).unwrap();

    Ok(())
}
