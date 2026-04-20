pub mod claims;
pub mod error;
pub mod models;
pub mod signature_client;

use crate::claims::ClaimProperties;
use crate::error::Error;
use crate::models::{
    ClientPresentation, GeneratePresentationRequest, ServerPresentation, StartSignatureRequest,
    VerifySignatureRequest, VerifySignatureResponse,
};
use crate::signature_client::{SignatureClient, create_signature_client};
use dg_xch_core::blockchain::sized_bytes::Bytes32;
use dg_xch_core::traits::SizedBytes;
use portfu::macros::*;
use portfu::pfcore::Json;
use portfu::pfcore::services::RequestHeaders;
use portfu::prelude::log::{debug, info};
use portfu::prelude::tokio_tungstenite::connect_async;
use portfu::prelude::tokio_tungstenite::tungstenite::Message;
use portfu::prelude::tokio_tungstenite::tungstenite::client::IntoClientRequest;
use portfu::prelude::uuid::Uuid;
use portfu::prelude::*;
use portfu::wrappers::sessions::{Session, SessionManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use time::OffsetDateTime;
use tokio::sync::RwLock;

pub type SessionSignatures = HashMap<String, String>; //(request_id, session_id)

pub type SuccessCallback = Box<
    dyn Fn(
            VerifySignatureResponse,
            Arc<RwLock<Session>>,
        ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;
pub type FailureCallback = Box<
    dyn Fn(&Error) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;
pub type MessageGenerator = Box<dyn Fn() -> String + Send + Sync + 'static>;
struct SignatureConfig {
    pub requested_claims: Vec<ClaimProperties>,
    pub required_site_pass: bool,
    pub message_generator: MessageGenerator,
    pub on_success: SuccessCallback,
    pub on_failure: FailureCallback,
    pub expire_time: i64,
}

pub struct ServiceBuilder {
    pub requested_claims: Vec<ClaimProperties>,
    pub required_site_pass: bool,
    pub message_generator: MessageGenerator,
    pub on_success: SuccessCallback,
    pub on_failure: FailureCallback,
    pub expire_time: i64,
}
impl ServiceBuilder {
    pub fn new() -> Self {
        ServiceBuilder {
            requested_claims: Vec::new(),
            required_site_pass: false,
            message_generator: Box::new(|| "".to_string()),
            on_success: Box::new(|_, _| Box::pin(async move { Ok(()) })),
            on_failure: Box::new(|_| Box::pin(async move { Ok(()) })),
            expire_time: 3600,
        }
    }
    pub fn request_claims(mut self, claims: Vec<ClaimProperties>) -> Self {
        self.requested_claims = claims;
        self
    }
    pub fn require_site_pass(mut self, required: bool) -> Self {
        self.required_site_pass = required;
        self
    }
    pub fn message_generator(mut self, generator: MessageGenerator) -> Self {
        self.message_generator = generator;
        self
    }
    pub fn on_success(mut self, generator: SuccessCallback) -> Self {
        self.on_success = generator;
        self
    }
    pub fn on_failure(mut self, generator: FailureCallback) -> Self {
        self.on_failure = generator;
        self
    }
    pub fn expire_time(mut self, expire_time: i64) -> Self {
        self.expire_time = expire_time;
        self
    }
    pub fn build(self) -> ServiceGroup {
        ServiceGroup::from(self)
    }
}
impl From<ServiceBuilder> for ServiceGroup {
    fn from(builder: ServiceBuilder) -> ServiceGroup {
        let client = create_signature_client();
        ServiceGroup::default()
            .shared_state(SignatureConfig {
                requested_claims: builder.requested_claims,
                required_site_pass: builder.required_site_pass,
                message_generator: builder.message_generator,
                on_success: builder.on_success,
                on_failure: builder.on_failure,
                expire_time: builder.expire_time,
            })
            .shared_state(client)
            .shared_state(RwLock::new(SessionSignatures::new()))
            .service(get_signature_url)
            .service(get_signature_status)
            .service(get_request_presentation)
            .service(verify_presentation)
            .service(verify_honestbot::default())
            .service(calculate_site_pass::default())
    }
}

#[get("/signature/notbot", output = "json", eoutput = "bytes")]
async fn get_signature_url(
    signature_client: State<SignatureClient>,
    session: State<RwLock<Session>>,
    config: State<SignatureConfig>,
    session_signatures: State<RwLock<SessionSignatures>>,
) -> Result<String, Error> {
    if let Some(current_claims) = session
        .0
        .write()
        .await
        .data
        .remove::<VerifySignatureResponse>()
    {
        info!(
            "Removing Old Session: {}",
            Bytes32::new(current_claims.alias_did.launcher_id)
        );
    }
    let resp = signature_client
        .start_signature(StartSignatureRequest {
            requested_credentials: config
                .requested_claims
                .iter()
                .map(|v| v.to_string())
                .collect(),
            require_site_pass: config.required_site_pass,
            required_alias_launcher: None,
            requested_message: (config.message_generator)().into_bytes(),
            expires: OffsetDateTime::now_utc().unix_timestamp() + config.expire_time,
        })
        .await?;
    let session_id = session.0.read().await.id.to_string();
    info!("Saving Request {} to session {session_id}", resp.request_id);
    session_signatures
        .write()
        .await
        .insert(resp.request_id.clone(), session_id);
    Ok(resp.request_id)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct SignaturePresentationRequest {
    pub nonce: Bytes32,
}

#[get("/signature/status", output = "json", eoutput = "bytes")]
async fn get_signature_status(session: State<RwLock<Session>>) -> Result<bool, Error> {
    Ok(session
        .0
        .read()
        .await
        .data
        .get::<VerifySignatureResponse>()
        .is_some())
}

#[post("/signature/notbot/{squid}", output = "json", eoutput = "bytes")]
async fn get_request_presentation(
    payload: Json<Option<SignaturePresentationRequest>>,
    squid: Path,
    signature_client: State<SignatureClient>,
) -> Result<ServerPresentation, Error> {
    let payload = payload.inner().ok_or(Error::input("Missing payload"))?;
    let response = signature_client
        .generate_presentation(GeneratePresentationRequest {
            request_id: squid.inner(),
            nonce: payload.nonce,
        })
        .await?;
    Ok(ServerPresentation {
        compressed_presentation: response.compressed_presentation,
    })
}
#[post("/signature/verify/{request_id}", output = "json", eoutput = "bytes")]
async fn verify_presentation(
    payload: Json<Option<ClientPresentation>>,
    request_id: Path,
    signature_client: State<SignatureClient>,
    session: State<RwLock<Session>>,
    session_signatures: State<RwLock<SessionSignatures>>,
    config: State<SignatureConfig>,
) -> Result<(), Error> {
    let payload = payload.inner().ok_or(Error::input("Missing payload"))?;
    let request_id = request_id.inner();
    let response = match signature_client
        .verify_presentation(VerifySignatureRequest {
            request_id: request_id.clone(),
            presentation: payload.presentation,
        })
        .await
    {
        Ok(response) => response,
        Err(e) => {
            (config.0.on_failure)(&e).await?;
            return Err(Error::validation(format!("{e:?}")));
        }
    };
    info!("Loading Request {request_id}");
    let signed_session = match session_signatures.0.write().await.remove(&request_id) {
        None => {
            info!("No Session Found to Add Signature");
            session.0.clone()
        }
        Some(session_id) => {
            info!("Found Existing Session: {session_id}");
            match SessionManager::get_session_from_id(&session_id) {
                Some(session) => {
                    info!("Found in SESSIONS Map");
                    session.clone()
                }
                None => {
                    info!("Missing From SESSIONS Map");
                    session.0.clone()
                }
            }
        }
    };
    info!("Running Success Callback");
    (config.0.on_success)(response, signed_session.clone()).await?;
    Ok(())
}

#[websocket("/signature/honestbot")]
async fn verify_honestbot(
    client_socket: WebSocket,
    headers: RequestHeaders,
    signature_client: State<SignatureClient>,
) -> Result<(), Error> {
    let ws_url = signature_client.url().replace("http", "ws");
    info!("Starting Upstream MPC Connection to: {}", ws_url);
    let mut request = (&format!("{}/signature/honestbot", ws_url))
        .into_client_request()
        .map_err(Error::connection)?;
    for (name, value) in headers.iter() {
        if name == "x-presentation-hash" {
            request
                .headers_mut()
                .insert(name.to_owned(), value.to_owned());
        }
    }
    let (ws_stream, response) = match connect_async(request).await {
        Ok(result) => result,
        Err(e) => return Err(Error::connection(e)),
    };
    info!("Connected to MPC with HTTP status: {}", response.status());
    let upstream_socket = WebSocket::new(
        WebsocketConnection::new(WebsocketMsgStream::Tls(Box::new(ws_stream))),
        Arc::new(Uuid::new_v4()),
    );
    let client_socket = Arc::new(client_socket);
    let upstream_socket = Arc::new(upstream_socket);
    let run = Arc::new(AtomicBool::new(true));
    info!("Starting Proxy");
    match proxy_websockets(client_socket.clone(), upstream_socket.clone(), run.clone()).await {
        Ok(()) => {
            run.store(false, Ordering::SeqCst);
            client_socket
                .send(Message::Close(None))
                .await
                .unwrap_or_default();
            upstream_socket
                .send(Message::Close(None))
                .await
                .unwrap_or_default();
            Ok(())
        }
        Err(e) => {
            run.store(false, Ordering::SeqCst);
            client_socket
                .send(Message::Close(None))
                .await
                .unwrap_or_default();
            upstream_socket
                .send(Message::Close(None))
                .await
                .unwrap_or_default();
            Err(Error::connection(e))
        }
    }
}

#[websocket("/calculate_site_pass")]
async fn calculate_site_pass(
    client_socket: WebSocket,
    signature_client: State<SignatureClient>,
) -> Result<(), Error> {
    let ws_url = signature_client.url().replace("http", "ws");
    info!("Starting Upstream MPC Connection to: {}", ws_url);
    let request = (&format!("{}/calculate_site_pass", ws_url))
        .into_client_request()
        .map_err(Error::connection)?;
    let (ws_stream, response) = match connect_async(request).await {
        Ok(result) => result,
        Err(e) => return Err(Error::connection(e)),
    };
    info!("Connected to MPC with HTTP status: {}", response.status());
    let upstream_socket = WebSocket::new(
        WebsocketConnection::new(WebsocketMsgStream::Tls(Box::new(ws_stream))),
        Arc::new(Uuid::new_v4()),
    );
    let client_socket = Arc::new(client_socket);
    let upstream_socket = Arc::new(upstream_socket);
    let run = Arc::new(AtomicBool::new(true));
    info!("Starting Proxy");
    match proxy_websockets(client_socket.clone(), upstream_socket.clone(), run.clone()).await {
        Ok(()) => {
            run.store(false, Ordering::SeqCst);
            client_socket
                .send(Message::Close(None))
                .await
                .unwrap_or_default();
            upstream_socket
                .send(Message::Close(None))
                .await
                .unwrap_or_default();
            Ok(())
        }
        Err(e) => {
            run.store(false, Ordering::SeqCst);
            client_socket
                .send(Message::Close(None))
                .await
                .unwrap_or_default();
            upstream_socket
                .send(Message::Close(None))
                .await
                .unwrap_or_default();
            Err(Error::connection(e))
        }
    }
}

async fn proxy_websockets(
    socket: Arc<WebSocket>,
    other_socket: Arc<WebSocket>,
    shutdown_signal: Arc<AtomicBool>,
) -> Result<(), Error> {
    let mut err = None;
    loop {
        match socket.next_message().await {
            Ok(Some(msg)) => match msg {
                Message::Binary(bin_msg) => {
                    if let Err(e) = other_socket.send(Message::Binary(bin_msg)).await {
                        err = Some(Error::other(format!("{e:?}")));
                        break;
                    }
                    continue;
                }
                Message::Text(msg) => {
                    if let Err(e) = other_socket.send(Message::Text(msg)).await {
                        err = Some(Error::other(format!("{e:?}")));
                        break;
                    }
                    continue;
                }
                Message::Ping(ping_data) => {
                    socket
                        .send(Message::Pong(ping_data))
                        .await
                        .map_err(Error::connection)?;
                }
                Message::Pong(_) | Message::Frame(_) => {
                    continue;
                }
                Message::Close(close_msg) => {
                    info!("End of MPC Proxy Web Stream. Closing Client Socket.");
                    other_socket
                        .send(Message::Close(close_msg))
                        .await
                        .unwrap_or_default();
                    break;
                }
            },
            Ok(None) => {
                if !shutdown_signal.load(Ordering::SeqCst) {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
            Err(e) => {
                info!("IO Error with MPC Proxy Web Stream. Closing Client Socket.");
                err = Some(Error::io(e));
                break;
            }
        }
        match other_socket.next_message().await {
            Ok(Some(msg)) => match msg {
                Message::Binary(bin_msg) => {
                    if let Err(e) = socket.send(Message::Binary(bin_msg)).await {
                        err = Some(Error::other(format!("{e:?}")));
                        break;
                    }
                    continue;
                }
                Message::Text(msg) => {
                    if let Err(e) = socket.send(Message::Text(msg)).await {
                        err = Some(Error::other(format!("{e:?}")));
                        break;
                    }
                    continue;
                }
                Message::Ping(ping_data) => {
                    other_socket
                        .send(Message::Pong(ping_data))
                        .await
                        .map_err(Error::connection)?;
                }
                Message::Pong(_) | Message::Frame(_) => {
                    continue;
                }
                Message::Close(close_msg) => {
                    info!("End of MPC Proxy Web Stream. Closing Server Socket.");
                    other_socket
                        .send(Message::Close(close_msg))
                        .await
                        .unwrap_or_default();
                    break;
                }
            },
            Ok(None) => {
                if !shutdown_signal.load(Ordering::SeqCst) {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
            Err(e) => {
                info!("IO Error with MPC Proxy Web Stream. Closing Server Socket.");
                err = Some(Error::io(e));
                break;
            }
        }
    }
    if let Some(e) = &err {
        return Err(Error::other(format!("Websocket connection closed: {e:?}")));
    }
    debug!("Shutting down MPC");
    shutdown_signal.store(false, Ordering::SeqCst);
    Ok::<(), Error>(())
}
