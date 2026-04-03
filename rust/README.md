# Julia Web SDK (Rust)

Rust crate for Julia verification routes, plus async clients for both `/signature/*` and `/auth/*` endpoints.

## Crate

- Name: `julia_web_sdk`
- Version: `1.0.0`

## Main APIs

- `ServiceBuilder`: registers auth routes and websocket proxy routes.
- `signature_client::SignatureClient`: single client for `/signature/*` and `/auth/*`.
- `models`: shared request/response payload structs used by services and clients.

## Recommended Integration

- Use `ServiceBuilder` in your backend service as the default integration path.
- Use `signature_client::SignatureClient` directly for custom/manual endpoint calls or automation scripts.

## Client Methods

- Auth:
  - `get_auth_request_id() -> Result<String, Error>`
  - `get_auth_status() -> Result<bool, Error>`
  - `generate_auth_presentation(request_id, nonce) -> Result<ServerPresentation, Error>`
  - `verify_auth_presentation(request_id, presentation) -> Result<(), Error>`
- Signature:
  - `start_signature(StartSignatureRequest) -> Result<StartSignatureResponse, Error>`
  - `generate_presentation(GeneratePresentationRequest) -> Result<GeneratePresentationResponse, Error>`
  - `verify_presentation(VerifySignatureRequest) -> Result<VerifySignatureResponse, Error>`

## Backend Integration Pattern

```rust
use julia_web_sdk::claims::ClaimProperties;
use julia_web_sdk::ServiceBuilder;

let signature_service = ServiceBuilder::new()
    .request_claims(vec![
        ClaimProperties::Notbot0,
        ClaimProperties::SitePass,
        ClaimProperties::FirstName,
        ClaimProperties::AgeOver18,
    ])
    .require_site_pass(true)
    .message_generator(Box::new(|| "Verifying My Identity with example.com".to_string()))
    .build();
```

## Client Pattern

```rust
use julia_web_sdk::signature_client::SignatureClient;

let client = SignatureClient::with_url("https://example.com");
let request_id = client.get_auth_request_id().await?;
let status = client.get_auth_status().await?;

// Nonce/presentation are placeholders
let nonce: dg_xch_core::blockchain::sized_bytes::Bytes32 = todo!("32-byte nonce");
let server_presentation = client.generate_auth_presentation(&request_id, nonce).await?;
client.verify_auth_presentation(&request_id, vec![]).await?;
```

## Examples

- `../examples/rust/server_integration.rs`
- `../examples/rust/client_usage.rs`
