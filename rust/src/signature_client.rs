use crate::error::{Error, ErrorCode};
use crate::models::{
    ClientPresentation, GeneratePresentationRequest, GeneratePresentationResponse,
    ServerPresentation, StartSignatureRequest, StartSignatureResponse, VerifySignatureRequest,
    VerifySignatureResponse,
};
use dg_xch_core::blockchain::sized_bytes::Bytes32;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

#[derive(Clone)]
pub struct SignatureClient {
    client: Client,
    url: String,
    api_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AuthNonceRequest {
    pub nonce: Bytes32,
}

impl SignatureClient {
    pub fn new(host: &str, port: u16, api_key: &str, secure: bool) -> SignatureClient {
        let client = build_http_client();
        let url = if secure {
            format!("https://{host}:{port}")
        } else {
            format!("http://{host}:{port}")
        };
        Self {
            client,
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
        Self {
            client: build_http_client(),
            url: url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }
    pub async fn start_signature(
        &self,
        request: StartSignatureRequest,
    ) -> Result<StartSignatureResponse, Error> {
        let builder = self
            .client
            .post(format!("{}/signature/start", &self.url))
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
            .post(format!("{}/signature/presentation", &self.url))
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
            .post(format!("{}/signature/verify", &self.url))
            .header("api-key", &self.api_key.clone());
        let response = builder.json(&request).send().await.map_err(|e| {
            Error::connection(format!("Error Connecting to Signature Server: {e:?}"))
        })?;
        let resp: VerifySignatureResponse = parse_response(response, "signature").await?;
        Ok(resp)
    }

    pub async fn get_auth_request_id(&self) -> Result<String, Error> {
        let response = self
            .client
            .get(format!("{}/auth/notbot", &self.url))
            .header("accept", "application/json")
            .send()
            .await
            .map_err(|e| Error::connection(format!("Error Connecting to Auth Server: {e:?}")))?;
        parse_response(response, "auth").await
    }

    pub async fn get_auth_status(&self) -> Result<bool, Error> {
        let response = self
            .client
            .get(format!("{}/auth/status", &self.url))
            .header("accept", "application/json")
            .send()
            .await
            .map_err(|e| Error::connection(format!("Error Connecting to Auth Server: {e:?}")))?;
        parse_response(response, "auth").await
    }

    pub async fn generate_auth_presentation(
        &self,
        request_id: &str,
        nonce: Bytes32,
    ) -> Result<ServerPresentation, Error> {
        let response = self
            .client
            .post(format!("{}/auth/notbot/{}", &self.url, request_id))
            .json(&AuthNonceRequest { nonce })
            .send()
            .await
            .map_err(|e| Error::connection(format!("Error Connecting to Auth Server: {e:?}")))?;
        parse_response(response, "auth").await
    }

    pub async fn verify_auth_presentation(
        &self,
        request_id: &str,
        presentation: Vec<u8>,
    ) -> Result<(), Error> {
        let response = self
            .client
            .post(format!("{}/auth/verify/{}", &self.url, request_id))
            .json(&ClientPresentation { presentation })
            .send()
            .await
            .map_err(|e| Error::connection(format!("Error Connecting to Auth Server: {e:?}")))?;
        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response.bytes().await.map_err(|e| {
                Error::new(
                    ErrorCode::BodyRead,
                    format!("Error Reading Auth Body: {e:?}"),
                )
            })?;
            Err(Error::new(
                ErrorCode::ServerError,
                format!(
                    "Got Auth Server Error: {} - {}",
                    status.as_u16(),
                    String::from_utf8_lossy(&body)
                ),
            ))
        }
    }
}

fn build_http_client() -> Client {
    Client::builder()
        .cookie_store(true)
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
