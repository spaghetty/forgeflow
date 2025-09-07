// The `google_auth` module provides a helper function for authenticating with the Gmail API.

use google_gmail1::{
    api::Scope,
    yup_oauth2::{
        self, authenticator_delegate::InstalledFlowDelegate, InstalledFlowAuthenticator,
        InstalledFlowReturnMethod,
    },
    Gmail,
};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client},
    rt::TokioExecutor,
};
use rustls::crypto::{ring::default_provider, CryptoProvider};
use serde::{Deserialize, Deserializer};
use std::{future::Future, path::PathBuf, pin::Pin, sync::Arc};
use thiserror::Error;
use tokio_util::bytes;
use tracing::info;

/// A type alias for the HTTPS connector.
pub type HttpsConnectorType = HttpsConnector<HttpConnector>;
/// A type alias for the Hyper client.
pub type HyperClient = Client<HttpsConnectorType, http_body_util::Full<bytes::Bytes>>;
/// A type alias for the authenticator.
pub type AuthType = yup_oauth2::authenticator::Authenticator<HttpsConnectorType>;
/// A type alias for the Gmail hub.
pub type GmailHubType = Gmail<HttpsConnectorType>;

/// The `AuthError` enum defines the possible errors that can occur during authentication.
#[derive(Error, Debug)]
pub enum AuthError {
    /// An error occurred while reading the credential file.
    #[error("Credential file not found or failed to read: {0}")]
    CredentialReadError(String),
    /// An error occurred during the OAuth2 flow.
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
}

/// The `GoogleAuthFlow` enum represents the different authentication flows.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GoogleAuthFlow {
    /// The redirect flow.
    Redirect {
        /// The port to use for the redirect server.
        port: Option<u16>,
        /// Whether to open the browser automatically.
        open_browser: bool,
    },
    /// The interactive flow.
    Interactive {
        /// Whether to open the browser automatically.
        open_browser: bool,
    },
}

impl Default for GoogleAuthFlow {
    fn default() -> Self {
        GoogleAuthFlow::Redirect {
            port: None,
            open_browser: false,
        }
    }
}

/// An installed flow delegate that opens the browser.
#[derive(Debug, Default)]
pub struct InstalledFlowBrowserDelegate;

impl InstalledFlowDelegate for InstalledFlowBrowserDelegate {
    fn present_user_url<'a>(
        &'a self,
        url: &'a str,
        need_code: bool,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + 'a>> {
        Box::pin(async move {
            if webbrowser::open(url).is_ok() {
                println!("Your browser has been opened to visit:\n\n\t{url}\n");
            } else {
                println!("Please visit this URL in your browser:\n\n\t{url}\n");
            }

            if need_code {
                println!("Please enter the code you see in your browser here: ");
                let mut code = String::new();
                std::io::stdin().read_line(&mut code).unwrap();
                Ok(code)
            } else {
                Ok(String::new())
            }
        })
    }
}

/// The `GConf` struct holds the configuration for Google authentication.
#[derive(Clone, Debug)]
pub struct GConf(pub Arc<InnerConf>);

/// The inner configuration for `GConf`.
#[derive(Clone, Debug, Deserialize)]
pub struct InnerConf {
    /// The path to the `credential.json` file.
    pub credentials_path: PathBuf,
    /// The path to the `token.json` file.
    pub token_path: PathBuf,
    /// The authentication flow to use.
    #[serde(default)]
    pub flow: GoogleAuthFlow,
}

impl<'de> Deserialize<'de> for GConf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = InnerConf::deserialize(deserializer)?;
        Ok(GConf(Arc::new(inner)))
    }
}

impl From<Arc<InnerConf>> for GConf {
    fn from(inner: Arc<InnerConf>) -> Self {
        GConf(inner)
    }
}

/// Authenticates with the Gmail API and returns a `GmailHubType`.
pub async fn gmail_auth(conf: GConf, scopes: &[Scope]) -> Result<GmailHubType, AuthError> {
    info!("Authenticating with Gmail API");

    // Read application secret
    let secret = yup_oauth2::read_application_secret(&conf.0.credentials_path)
        .await
        .map_err(|e| AuthError::CredentialReadError(e.to_string()))?;

    let (return_method, open_browser) = match conf.0.flow {
        GoogleAuthFlow::Redirect { port, open_browser } => (
            match port {
                Some(port) => InstalledFlowReturnMethod::HTTPPortRedirect(port),
                None => InstalledFlowReturnMethod::HTTPRedirect,
            },
            open_browser,
        ),
        GoogleAuthFlow::Interactive { open_browser } => {
            (InstalledFlowReturnMethod::Interactive, open_browser)
        }
    };

    // Set up OAuth2 authenticator with required Gmail scopes
    let mut builder = InstalledFlowAuthenticator::builder(secret, return_method)
        .persist_tokens_to_disk(&conf.0.token_path);

    if open_browser {
        builder = builder.flow_delegate(Box::new(InstalledFlowBrowserDelegate::default()));
    }

    let auth = builder
        .build()
        .await
        .map_err(|e| AuthError::AuthenticationFailed(e.to_string()))?;

    // Request initial token to ensure authentication works
    let _token = auth
        .token(scopes)
        .await
        .map_err(|e| AuthError::AuthenticationFailed(e.to_string()))?;

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
