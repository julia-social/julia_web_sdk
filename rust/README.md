# Julia Web SDK (Rust)

Rust crate for Julia verification routes, plus async clients for both upstream and SDK `/signature/*` endpoints.

## Crate

- Name: `julia_web_sdk`
- Version: `1.0.0`

## Main APIs

- `ServiceBuilder`: registers signature routes and websocket proxy routes.
- `signature_client::SignatureClient`: single client for upstream and SDK `/signature/*`.
- `models`: shared request/response payload structs used by services and clients.

## Recommended Integration

- Use `ServiceBuilder` in your backend service as the default integration path.
- Use `signature_client::SignatureClient` directly for custom/manual endpoint calls or automation scripts.

## Client Methods

- Signature SDK:
  - `get_signature_request_id() -> Result<String, Error>`
  - `get_signature_status() -> Result<bool, Error>`
  - `generate_signature_presentation(request_id, nonce) -> Result<ServerPresentation, Error>`
  - `verify_signature_presentation(request_id, presentation) -> Result<(), Error>`
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
let request_id = client.get_signature_request_id().await?;
let status = client.get_signature_status().await?;

// Nonce/presentation are placeholders
let nonce: dg_xch_core::blockchain::sized_bytes::Bytes32 = todo!("32-byte nonce");
let server_presentation = client.generate_signature_presentation(&request_id, nonce).await?;
client.verify_signature_presentation(&request_id, vec![]).await?;
```

## Examples

- `../examples/rust/server_integration.rs`
- `../examples/rust/client_usage.rs`
