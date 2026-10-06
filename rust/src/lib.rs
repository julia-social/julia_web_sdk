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
use portfu::prelude::log::{debug, info};
use portfu::prelude::tokio_tungstenite::connect_async;
use portfu::prelude::tokio_tungstenite::tungstenite::Message;
use portfu::prelude::tokio_tungstenite::tungstenite::client::IntoClientRequest;
use portfu::prelude::wrappers::sessions::SessionManager;
use portfu::prelude::*;
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
impl Default for ServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
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
            .service(get_service(
                "/signature/notbot",
                "signature_notbot",
                GetSignatureUrl,
            ))
            .service(get_service(
                "/signature/status",
                "signature_status",
                GetSignatureStatus,
            ))
            .service(post_service(
                "/signature/notbot/{squid}",
                "signature_presentation",
                GetRequestPresentation,
            ))
            .service(post_service(
                "/signature/verify/{request_id}",
                "signature_verify",
                VerifyPresentation,
            ))
            .service(websocket_service(
                "/signature/honestbot",
                "signature_honestbot",
                VerifyHonestbot,
            ))
            .service(websocket_service(
                "/calculate_site_pass",
                "calculate_site_pass",
                CalculateSitePass,
            ))
    }
}

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

async fn get_signature_status(session: State<RwLock<Session>>) -> Result<bool, Error> {
    Ok(session
        .0
        .read()
        .await
        .data
        .get::<VerifySignatureResponse>()
        .is_some())
}

async fn get_request_presentation(
    payload: Json<Option<SignaturePresentationRequest>>,
    squid: Path,
    signature_client: State<SignatureClient>,
) -> Result<ServerPresentation, Error> {
    let payload = payload
        .into_inner()
        .ok_or(Error::input("Missing payload"))?;
    let response = signature_client
        .generate_presentation(GeneratePresentationRequest {
            request_id: squid.inner(),
            nonce: payload.nonce,
            presentation_format: None,
        })
        .await?;
    Ok(ServerPresentation {
        compressed_presentation: response.compressed_presentation,
    })
}
async fn verify_presentation(
    payload: Json<Option<ClientPresentation>>,
    request_id: Path,
    signature_client: State<SignatureClient>,
    session: State<RwLock<Session>>,
    session_signatures: State<RwLock<SessionSignatures>>,
    config: State<SignatureConfig>,
) -> Result<(), Error> {
    let payload = payload
        .into_inner()
        .ok_or(Error::input("Missing payload"))?;
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
    if let Some(cookie) = signature_client.cookie_header()? {
        request.headers_mut().insert(http::header::COOKIE, cookie);
    }

    let (ws_stream, response) = match connect_async(request).await {
        Ok(result) => result,
        Err(e) => return Err(Error::connection(e)),
    };
    info!("Connected to MPC with HTTP status: {}", response.status());
    let upstream_socket = ClientWebSocket::new(ws_stream);
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

async fn calculate_site_pass(
    client_socket: WebSocket,
    signature_client: State<SignatureClient>,
) -> Result<(), Error> {
    let ws_url = signature_client.url().replace("http", "ws");
    info!("Starting Upstream MPC Connection to: {}", ws_url);
    let mut request = (&format!("{}/calculate_site_pass", ws_url))
        .into_client_request()
        .map_err(Error::connection)?;
    if let Some(cookie) = signature_client.cookie_header()? {
        request.headers_mut().insert(http::header::COOKIE, cookie);
    }

    let (ws_stream, response) = match connect_async(request).await {
        Ok(result) => result,
        Err(e) => return Err(Error::connection(e)),
    };
    info!("Connected to MPC with HTTP status: {}", response.status());
    let upstream_socket = ClientWebSocket::new(ws_stream);
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

fn get_service<H>(path: &str, name: &str, handler: H) -> Service
where
    H: ServiceTrait + Send + Sync + 'static,
{
    portfu::prelude::ServiceBuilder::new(path)
        .name(name)
        .filter(filters::method::GET.clone())
        .handler(Arc::new(handler))
        .build()
}

fn post_service<H>(path: &str, name: &str, handler: H) -> Service
where
    H: ServiceTrait + Send + Sync + 'static,
{
    portfu::prelude::ServiceBuilder::new(path)
        .name(name)
        .filter(filters::method::POST.clone())
        .handler(Arc::new(handler))
        .build()
}

fn websocket_service<H>(path: &str, name: &str, handler: H) -> Service
where
    H: ServiceTrait + Send + Sync + 'static,
{
    portfu::prelude::ServiceBuilder::new(path)
        .name(name)
        .filter(Arc::new(filters::any(
            String::new(),
            &[
                filters::method::GET.clone(),
                filters::method::OPTIONS.clone(),
            ],
        )))
        .handler(Arc::new(handler))
        .build()
}

macro_rules! extract_or_error {
    ($request:expr, $ty:ty) => {
        match <$ty as FromRequest<Request>>::try_from($request).await {
            Ok(value) => value,
            Err(error) => return Ok(Response::internal_error(format!("{error:?}"))),
        }
    };
}

struct GetSignatureUrl;

impl ServiceTrait for GetSignatureUrl {
    fn name(&self) -> &str {
        "signature_notbot"
    }

    fn serve<'a>(
        &'a self,
        request: &'a mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, PortfuError>> + Send + 'a>> {
        Box::pin(async move {
            let signature_client = extract_or_error!(request, State<SignatureClient>);
            let session = extract_or_error!(request, State<RwLock<Session>>);
            let config = extract_or_error!(request, State<SignatureConfig>);
            let session_signatures = extract_or_error!(request, State<RwLock<SessionSignatures>>);
            match get_signature_url(signature_client, session, config, session_signatures).await {
                Ok(response) => Ok(Response::json(response)),
                Err(error) => Ok(Response::internal_error(format!("{error:?}"))),
            }
        })
    }
}

struct GetSignatureStatus;

impl ServiceTrait for GetSignatureStatus {
    fn name(&self) -> &str {
        "signature_status"
    }

    fn serve<'a>(
        &'a self,
        request: &'a mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, PortfuError>> + Send + 'a>> {
        Box::pin(async move {
            let session = extract_or_error!(request, State<RwLock<Session>>);
            match get_signature_status(session).await {
                Ok(response) => Ok(Response::json(response)),
                Err(error) => Ok(Response::internal_error(format!("{error:?}"))),
            }
        })
    }
}

struct Squid;
impl PathName for Squid {
    const NAME: &'static str = "squid";
}

struct GetRequestPresentation;

impl ServiceTrait for GetRequestPresentation {
    fn name(&self) -> &str {
        "signature_presentation"
    }

    fn serve<'a>(
        &'a self,
        request: &'a mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, PortfuError>> + Send + 'a>> {
        Box::pin(async move {
            let payload = extract_or_error!(request, Json<Option<SignaturePresentationRequest>>);
            let squid: Path = extract_or_error!(request, PathImpl<Squid>).into();
            let signature_client = extract_or_error!(request, State<SignatureClient>);
            match get_request_presentation(payload, squid, signature_client).await {
                Ok(response) => Ok(Response::json(response)),
                Err(error) => Ok(Response::internal_error(format!("{error:?}"))),
            }
        })
    }
}

struct RequestId;
impl PathName for RequestId {
    const NAME: &'static str = "request_id";
}

struct VerifyPresentation;

impl ServiceTrait for VerifyPresentation {
    fn name(&self) -> &str {
        "signature_verify"
    }

    fn serve<'a>(
        &'a self,
        request: &'a mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, PortfuError>> + Send + 'a>> {
        Box::pin(async move {
            let payload = extract_or_error!(request, Json<Option<ClientPresentation>>);
            let request_id: Path = extract_or_error!(request, PathImpl<RequestId>).into();
            let signature_client = extract_or_error!(request, State<SignatureClient>);
            let session = extract_or_error!(request, State<RwLock<Session>>);
            let session_signatures = extract_or_error!(request, State<RwLock<SessionSignatures>>);
            let config = extract_or_error!(request, State<SignatureConfig>);
            match verify_presentation(
                payload,
                request_id,
                signature_client,
                session,
                session_signatures,
                config,
            )
            .await
            {
                Ok(()) => Ok(Response::ok("")),
                Err(error) => Ok(Response::internal_error(format!("{error:?}"))),
            }
        })
    }
}

struct VerifyHonestbot;

impl ServiceTrait for VerifyHonestbot {
    fn name(&self) -> &str {
        "signature_honestbot"
    }

    fn serve<'a>(
        &'a self,
        request: &'a mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, PortfuError>> + Send + 'a>> {
        Box::pin(async move {
            if request.method() == http::Method::OPTIONS {
                return Ok(Response::ok(""));
            }
            let headers = request.headers().clone();
            let signature_client = extract_or_error!(request, State<SignatureClient>);
            start_websocket(request, move |socket| async move {
                verify_honestbot(socket, headers, signature_client).await
            })
        })
    }
}

struct CalculateSitePass;

impl ServiceTrait for CalculateSitePass {
    fn name(&self) -> &str {
        "calculate_site_pass"
    }

    fn serve<'a>(
        &'a self,
        request: &'a mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, PortfuError>> + Send + 'a>> {
        Box::pin(async move {
            if request.method() == http::Method::OPTIONS {
                return Ok(Response::ok(""));
            }
            let signature_client = extract_or_error!(request, State<SignatureClient>);
            start_websocket(request, move |socket| async move {
                calculate_site_pass(socket, signature_client).await
            })
        })
    }
}

fn start_websocket<F, Fut>(request: &mut Request, handler: F) -> Result<Response, PortfuError>
where
    F: FnOnce(WebSocket) -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), Error>> + Send + 'static,
{
    use portfu::prelude::tokio_tungstenite::tungstenite::handshake::derive_accept_key;
    use portfu::prelude::tokio_tungstenite::tungstenite::protocol::Role;

    let is_upgrade = matches!(
        request
            .headers()
            .get(http::header::UPGRADE)
            .and_then(|value| value.to_str().ok()),
        Some(value) if value.eq_ignore_ascii_case("websocket")
    );
    if !is_upgrade {
        return Ok(Response::from_status_and_message(
            http::StatusCode::BAD_REQUEST,
            "Expected websocket upgrade request",
        ));
    }

    let Some(key) = request.headers().get("Sec-WebSocket-Key").cloned() else {
        return Ok(Response::from_status_and_message(
            http::StatusCode::BAD_REQUEST,
            "Missing Sec-WebSocket-Key header",
        ));
    };
    let version_ok = request
        .headers()
        .get("Sec-WebSocket-Version")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == "13");
    if !version_ok {
        return Ok(Response::from_status_and_message(
            http::StatusCode::BAD_REQUEST,
            "Unsupported websocket version",
        ));
    }

    let upgrade = match request.request_type() {
        RequestType::Stream(request) => hyper::upgrade::on(request),
        RequestType::Sized(request) => hyper::upgrade::on(request),
        _ => {
            return Ok(Response::from_status_and_message(
                http::StatusCode::BAD_REQUEST,
                "WebSocket upgrade requires a live HTTP request",
            ));
        }
    };
    let accept = derive_accept_key(key.as_bytes());
    let response = http::Response::builder()
        .status(http::StatusCode::SWITCHING_PROTOCOLS)
        .header(http::header::CONNECTION, "upgrade")
        .header(http::header::UPGRADE, "websocket")
        .header("Sec-WebSocket-Accept", accept)
        .body(())
        .map_err(|error| {
            PortfuError::Internal(format!("Failed to build websocket response: {error:?}"))
        })?;

    tokio::spawn(async move {
        match upgrade.await {
            Ok(upgraded) => {
                let websocket =
                    portfu::prelude::tokio_tungstenite::WebSocketStream::from_raw_socket(
                        hyper_util::rt::TokioIo::new(upgraded),
                        Role::Server,
                        None,
                    )
                    .await;
                if let Err(error) = handler(WebSocket::new(websocket)).await {
                    debug!("Websocket handler exited with error: {error}");
                }
            }
            Err(error) => debug!("Websocket upgrade failed: {error:?}"),
        }
    });

    Ok(response.into())
}

async fn proxy_websockets(
    socket: Arc<WebSocket>,
    other_socket: Arc<ClientWebSocket>,
    shutdown_signal: Arc<AtomicBool>,
) -> Result<(), Error> {
    loop {
        tokio::select! {
            source_msg = socket.next_message() => {
                match source_msg {
                    Ok(Some(Message::Binary(bin_msg))) => {
                        other_socket
                            .send(Message::Binary(bin_msg))
                            .await
                            .map_err(Error::connection)?;
                    }
                    Ok(Some(Message::Text(text_msg))) => {
                        other_socket
                            .send(Message::Text(text_msg))
                            .await
                            .map_err(Error::connection)?;
                    }
                    Ok(Some(Message::Ping(ping_data))) => {
                        socket
                            .send(Message::Pong(ping_data))
                            .await
                            .map_err(Error::connection)?;
                    }
                    Ok(Some(Message::Pong(_))) | Ok(Some(Message::Frame(_))) => {}
                    Ok(Some(Message::Close(close_msg))) => {
                        info!("Client websocket closed. Closing upstream websocket.");
                        other_socket
                            .send(Message::Close(close_msg))
                            .await
                            .unwrap_or_default();
                        break;
                    }
                    Ok(None) => {
                        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                    }
                    Err(e) => {
                        info!("IO error on client websocket. Closing upstream websocket.");
                        other_socket
                            .send(Message::Close(None))
                            .await
                            .unwrap_or_default();
                        return Err(Error::io(e));
                    }
                }
            }
            source_msg = other_socket.next_message() => {
                match source_msg {
                    Ok(Some(Message::Binary(bin_msg))) => {
                        socket
                            .send(Message::Binary(bin_msg))
                            .await
                            .map_err(Error::connection)?;
                    }
                    Ok(Some(Message::Text(text_msg))) => {
                        socket
                            .send(Message::Text(text_msg))
                            .await
                            .map_err(Error::connection)?;
                    }
                    Ok(Some(Message::Ping(ping_data))) => {
                        other_socket
                            .send(Message::Pong(ping_data))
                            .await
                            .map_err(Error::connection)?;
                    }
                    Ok(Some(Message::Pong(_))) | Ok(Some(Message::Frame(_))) => {}
                    Ok(Some(Message::Close(close_msg))) => {
                        info!("Upstream websocket closed. Closing client websocket.");
                        socket
                            .send(Message::Close(close_msg))
                            .await
                            .unwrap_or_default();
                        break;
                    }
                    Ok(None) => {
                        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                    }
                    Err(e) => {
                        info!("IO error on upstream websocket. Closing client websocket.");
                        socket
                            .send(Message::Close(None))
                            .await
                            .unwrap_or_default();
                        return Err(Error::io(e));
                    }
                }
            }
        }
        if !shutdown_signal.load(Ordering::SeqCst) {
            break;
        }
    }
    debug!("Shutting down MPC");
    shutdown_signal.store(false, Ordering::SeqCst);
    Ok::<(), Error>(())
}
