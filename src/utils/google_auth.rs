// The `google_auth` module provides a helper function for authenticating with the Gmail API.

use google_gmail1::{
    Gmail,
    api::Scope,
    yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod},
};
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

/// A type alias for the HTTPS connector.
pub type HttpsConnectorType = HttpsConnector<HttpConnector>;
/// A type alias for the Hyper client.
pub type HyperClient = Client<HttpsConnectorType, http_body_util::Full<bytes::Bytes>>;
/// A type alias for the authenticator.
pub type AuthType = google_gmail1::yup_oauth2::authenticator::Authenticator<HttpsConnectorType>;
/// A type alias for the Gmail hub.
pub type GmailHubType = Gmail<HttpsConnectorType>;

/// The `AuthError` enum defines the possible errors that can occur during authentication.
#[derive(Error, Debug)]
pub enum AuthError {
    /// An error occurred while authenticating.
    #[error("Error authenticating the element")]
    AuthError,
}

/// The `GConf` struct holds the configuration for Google authentication.
#[derive(Clone, Debug)]
pub struct GConf(Arc<InnerConf>);

/// The inner configuration for `GConf`.
#[derive(Clone, Debug)]
pub struct InnerConf {
    /// The path to the `credential.json` file.
    pub credentials_path: PathBuf,
    /// The path to the `token.json` file.
    pub token_path: PathBuf,
}

impl GConf {
    /// Creates a new `GConf`.
    pub fn new(credentials_path: PathBuf, token_path: PathBuf) -> GConf {
        GConf(Arc::new(InnerConf {
            credentials_path,
            token_path,
        }))
    }
}

/// Authenticates with the Gmail API and returns a `GmailHubType`.
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
    let _token = auth.token(scopes).await.unwrap();

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
