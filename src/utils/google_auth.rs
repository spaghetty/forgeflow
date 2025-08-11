use google_gmail1::yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use google_gmail1::{Gmail, api::Scope};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::{
    client::legacy::Client, client::legacy::connect::HttpConnector, rt::TokioExecutor,
};
use rustls::crypto::{CryptoProvider, ring::default_provider};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio_util::bytes;
use tracing::info;

pub type HttpsConnectorType = HttpsConnector<HttpConnector>;
pub type HyperClient = Client<HttpsConnectorType, http_body_util::Full<bytes::Bytes>>;
pub type AuthType = google_gmail1::yup_oauth2::authenticator::Authenticator<HttpsConnectorType>;
pub type GmailHubType = Gmail<HttpsConnectorType>;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Error authenticating the element")]
    AuthError,
}

#[derive(Clone, Debug)]
pub struct GConf(Arc<InnerConf>);

#[derive(Clone, Debug)]
pub struct InnerConf {
    pub credentials_path: PathBuf,
    pub token_path: PathBuf,
}

impl GConf {
    pub fn new(credentials_path: PathBuf, token_path: PathBuf) -> GConf {
        GConf(Arc::new(InnerConf {
            credentials_path,
            token_path,
        }))
    }
}

pub async fn gmail_auth(conf: GConf, scopes: &[Scope]) -> Result<GmailHubType, AuthError> {
    info!("Authenticating with Gmail API");

    // Read application secret
    let secret = google_gmail1::yup_oauth2::read_application_secret(&conf.0.credentials_path)
        .await
        .expect("credential file missing");

    // Set up OAuth2 authenticator with required Gmail scopes
    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk(&conf.0.token_path)
        .build()
        .await
        .unwrap();

    // Request initial token to ensure authentication works
    let _token = auth.token(&scopes).await.unwrap();

    // Initialize the crypto provider
    _ = CryptoProvider::install_default(default_provider());

    // Create HTTP client with native roots
    let https = HttpsConnectorBuilder::new()
        .with_native_roots()
        .unwrap()
        .https_or_http()
        .enable_http1()
        .build();

    let client = Client::builder(TokioExecutor::new()).build(https);

    // Create Gmail hub
    let hub = Gmail::new(client, auth);
    info!("Successfully authenticated with Gmail API");
    Ok(hub)
}
