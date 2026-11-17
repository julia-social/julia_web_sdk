use crate::error::{Error, ErrorCode, SignatureFailure};
use crate::models::{
    ClientPresentation, GeneratePresentationRequest, GeneratePresentationResponse,
    ServerPresentation, StartSignatureRequest, StartSignatureResponse, VerifySignatureRequest,
    VerifySignatureResponse,
};
use dg_xch_core::blockchain::sized_bytes::Bytes32;
use reqwest::cookie::{CookieStore, Jar};
use reqwest::{Client, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct SignatureClient {
    client: Client,
    cookies: Arc<Jar>,
    url: String,
    api_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SignatureNonceRequest {
    pub nonce: Bytes32,
}

impl SignatureClient {
    pub fn new(host: &str, port: u16, api_key: &str, secure: bool) -> SignatureClient {
        let cookies = Arc::new(Jar::default());
        let client = build_http_client(cookies.clone());
        let url = if secure {
            format!("https://{host}:{port}")
        } else {
            format!("http://{host}:{port}")
        };
        Self {
            client,
            cookies,
            url,
            api_key: api_key.to_string(),
        }
    }

    pub fn with_url(url: &str) -> SignatureClient {
        SignatureClient::with_url_and_api_key(
            url,
            &env::var("SIGNATURE_API_KEY").unwrap_or("CHANGE_ME".to_string()),
        )
    }

    pub fn with_url_and_api_key(url: &str, api_key: &str) -> SignatureClient {
        let cookies = Arc::new(Jar::default());
        Self {
            client: build_http_client(cookies.clone()),
            cookies,
            url: url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    /// Returns cookies issued by the signature server for use in its WSS upgrade.
    ///
    /// Reqwest owns HTTP cookie handling, but tokio-tungstenite does not share
    /// Reqwest's cookie jar, so callers must add this header explicitly.
    pub fn cookie_header(&self) -> Result<Option<reqwest::header::HeaderValue>, Error> {
        let url = Url::parse(&self.url)
            .map_err(|error| Error::connection(format!("Invalid signature server URL: {error}")))?;
        Ok(self.cookies.cookies(&url))
    }
    pub async fn start_signature(
        &self,
        request: StartSignatureRequest,
    ) -> Result<StartSignatureResponse, Error> {
        let builder = self
            .client
            .post(format!("{}/signature/start", self.url))
            .header("api-key", &self.api_key.clone());
        let response = builder.json(&request).send().await.map_err(|e| {
            Error::connection(format!("Error Connecting to Signature Server: {e:?}"))
        })?;
        let resp: StartSignatureResponse = parse_response(response, "signature").await?;
        Ok(resp)
    }
    pub async fn generate_presentation(
        &self,
        request: GeneratePresentationRequest,
    ) -> Result<GeneratePresentationResponse, Error> {
        let builder = self
            .client
            .post(format!("{}/signature/presentation", self.url))
            .header("api-key", &self.api_key.clone());
        let response = builder.json(&request).send().await.map_err(|e| {
            Error::connection(format!("Error Connecting to Signature Server: {e:?}"))
        })?;
        let resp: GeneratePresentationResponse = parse_response(response, "signature").await?;
        Ok(resp)
    }
    pub async fn verify_presentation(
        &self,
        request: VerifySignatureRequest,
    ) -> Result<VerifySignatureResponse, Error> {
        let builder = self
            .client
            .post(format!("{}/signature/verify", self.url))
            .header("api-key", &self.api_key.clone());
        let response = builder.json(&request).send().await.map_err(|e| {
            Error::connection(format!("Error Connecting to Signature Server: {e:?}"))
        })?;
        let resp: VerifySignatureResponse = parse_response(response, "signature").await?;
        Ok(resp)
    }

    /// Generate a presentation with protocol v1 typed, safe failures.
    /// Requires a server supporting `x-notbot-protocol: 1`.
    pub async fn generate_presentation_v1(
        &self,
        request: GeneratePresentationRequest,
    ) -> Result<GeneratePresentationResponse, SignatureFailure> {
        self.post_v1("/signature/presentation", &request).await
    }

    /// Verify a presentation with protocol v1 typed, safe failures.
    /// Requires a server supporting `x-notbot-protocol: 1`.
    pub async fn verify_presentation_v1(
        &self,
        request: VerifySignatureRequest,
    ) -> Result<VerifySignatureResponse, SignatureFailure> {
        self.post_v1("/signature/verify", &request).await
    }

    async fn post_v1<T: DeserializeOwned>(
        &self,
        path: &str,
        request: &impl Serialize,
    ) -> Result<T, SignatureFailure> {
        let response = self
            .client
            .post(format!("{}{path}", self.url))
            .header("api-key", &self.api_key)
            .header("x-notbot-protocol", "1")
            .json(request)
            .send()
            .await
            .map_err(|_| SignatureFailure::internal(None))?;
        parse_response_v1(response).await
    }

    pub async fn get_signature_request_id(&self) -> Result<String, Error> {
        let response = self
            .client
            .get(format!("{}/signature/notbot", self.url))
            .header("accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                Error::connection(format!("Error Connecting to Signature Server: {e:?}"))
            })?;
        parse_response(response, "signature").await
    }

    pub async fn get_signature_status(&self) -> Result<bool, Error> {
        let response = self
            .client
            .get(format!("{}/signature/status", self.url))
            .header("accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                Error::connection(format!("Error Connecting to Signature Server: {e:?}"))
            })?;
        parse_response(response, "signature").await
    }

    pub async fn generate_signature_presentation(
        &self,
        request_id: &str,
        nonce: Bytes32,
    ) -> Result<ServerPresentation, Error> {
        let response = self
            .client
            .post(format!("{}/signature/notbot/{}", self.url, request_id))
            .json(&SignatureNonceRequest { nonce })
            .send()
            .await
            .map_err(|e| {
                Error::connection(format!("Error Connecting to Signature Server: {e:?}"))
            })?;
        parse_response(response, "signature").await
    }

    pub async fn verify_signature_presentation(
        &self,
        request_id: &str,
        presentation: Vec<u8>,
    ) -> Result<(), Error> {
        let response = self
            .client
            .post(format!("{}/signature/verify/{}", self.url, request_id))
            .json(&ClientPresentation { presentation })
            .send()
            .await
            .map_err(|e| {
                Error::connection(format!("Error Connecting to Signature Server: {e:?}"))
            })?;
        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response.bytes().await.map_err(|e| {
                Error::new(
                    ErrorCode::BodyRead,
                    format!("Error Reading Signature Body: {e:?}"),
                )
            })?;
            Err(Error::new(
                ErrorCode::ServerError,
                format!(
                    "Got Signature Server Error: {} - {}",
                    status.as_u16(),
                    String::from_utf8_lossy(&body)
                ),
            ))
        }
    }
}

fn build_http_client(cookies: Arc<Jar>) -> Client {
    Client::builder()
        .cookie_provider(cookies)
        .timeout(Duration::from_secs(180))
        .connect_timeout(Duration::from_secs(180))
        .read_timeout(Duration::from_secs(180))
        .pool_idle_timeout(Duration::from_secs(180))
        .build()
        .expect("Failed to build signature client")
}

pub fn create_signature_client() -> SignatureClient {
    let server_hostname = env::var("SIGNATURE_HOSTNAME").unwrap_or("localhost".to_string());
    let server_port = env::var("SIGNATURE_PORT")
        .map(|s| s.parse().unwrap())
        .unwrap_or(8080u16);
    let api_key = env::var("SIGNATURE_API_KEY").unwrap_or("CHANGE_ME".to_string());
    SignatureClient::new(
        &server_hostname,
        server_port,
        &api_key,
        server_hostname != "localhost",
    )
}

async fn parse_response_v1<T: DeserializeOwned>(
    mut response: reqwest::Response,
) -> Result<T, SignatureFailure> {
    let status = response.status();
    let internal = || SignatureFailure::internal(Some(status.as_u16()));
    let mut protocol_headers = response.headers().get_all("x-notbot-protocol").iter();
    let protocol_valid = match (protocol_headers.next(), protocol_headers.next()) {
        (None, None) => true,
        (Some(value), None) => value == "1",
        _ => false,
    };
    if !protocol_valid {
        return Err(internal());
    }
    let mut content_types = response
        .headers()
        .get_all(reqwest::header::CONTENT_TYPE)
        .iter();
    let typed_error = content_types
        .next()
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value
                .split(';')
                .next()
                .unwrap_or_default()
                .trim()
                .eq_ignore_ascii_case("application/vnd.notbot.error+json")
        });
    if content_types.next().is_some() {
        return Err(internal());
    }
    if status.is_success() {
        if typed_error {
            return Err(internal());
        }
        let body = response.bytes().await.map_err(|_| internal())?;
        return serde_json::from_slice(&body).map_err(|_| internal());
    }
    if !typed_error || !(status.is_client_error() || status.is_server_error()) {
        return Err(internal());
    }
    // Error envelopes are small. Bound allocation even if an upstream proxy
    // supplies an unbounded or chunked diagnostic response.
    const MAX_ERROR_BYTES: usize = 64 * 1024;
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| internal())? {
        if chunk.len() > MAX_ERROR_BYTES - body.len() {
            return Err(internal());
        }
        body.extend_from_slice(&chunk);
    }
    Err(SignatureFailure::from_envelope(&body, status.as_u16()).unwrap_or_else(internal))
}

pub(crate) async fn parse_response<T: DeserializeOwned>(
    response: reqwest::Response,
    service: &str,
) -> Result<T, Error> {
    let status = response.status();
    let body = response.bytes().await.map_err(|e| {
        Error::new(
            ErrorCode::BodyRead,
            format!("Error Reading Response Body {service}: {e:?}"),
        )
    })?;
    if status.is_success() {
        serde_json::from_slice(&body).map_err(|e| {
            Error::new(
                ErrorCode::InvalidJson,
                format!(
                    "Error Parsing Response from {service}: {e:?} - {}",
                    String::from_utf8_lossy(&body)
                ),
            )
        })
    } else {
        Err(Error {
            code: ErrorCode::ServerError,
            message: format!(
                "Got {service} Server Error: {} - {}",
                status.as_u16(),
                String::from_utf8_lossy(&body)
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_header_includes_secure_signer_cookie() {
        let client = SignatureClient::with_url_and_api_key("https://signer.example.com", "key");
        let signer_url = Url::parse(&client.url).unwrap();
        client
            .cookies
            .add_cookie_str("route=signer-1; Secure; Path=/", &signer_url);

        assert_eq!(
            client.cookie_header().unwrap().unwrap().to_str().unwrap(),
            "route=signer-1"
        );
    }
}
