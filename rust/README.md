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
  - `generate_presentation_v1(GeneratePresentationRequest) -> Result<GeneratePresentationResponse, SignatureFailure>`
  - `verify_presentation_v1(VerifySignatureRequest) -> Result<VerifySignatureResponse, SignatureFailure>`

The additive `_v1` methods require a protocol v1 server and send
`x-notbot-protocol: 1`. Existing methods and `ServiceBuilder` retain their
headerless requests and legacy error behavior; upgrading this dependency alone
does not opt existing applications into the new contract.

`julia_web_sdk::error::SignatureFailure` exposes:

```rust
pub struct SignatureFailure {
    pub version: u8,                    // Always 1
    pub code: String,                   // Validated wire code identifier
    pub message: String,                // Fixed safe SDK text
    pub correlation_id: Option<String>, // Validated upstream UUID, if available
    pub status: Option<u16>,            // Original upstream HTTP status, if received
}
```

Only an error response with media type `application/vnd.notbot.error+json`,
version `1`, a bounded code identifier, a string message, and a canonical hyphenated UUID
is accepted as a typed upstream failure. Application codes include
`InvalidSession`, `SessionExpired`, `ExistingIdentity`, `PresentationNotFound`,
`MissingCredential`, `SitePassRecalculationRequired`, `AlreadyUpgraded`,
`NotNewPerson`, `UpgradeTooSoon`, `RecoveryDataNotFound`, `RequestNotActive`,
`MetadataUnavailable`, `ProtocolUnsupported`, `NonceNotFound`, and
`UnsupportedPresentation`, plus generic Julia and relay codes. Branch on `code`,
never on `message` or HTTP status alone. Unknown identifiers are preserved with
generic safe text; callers must map unrecognized codes to a generic failure.
Identifiers must be ASCII alphanumeric, start with an uppercase letter, and be
at most 64 bytes. Extra envelope fields such as `legacy_body_bytes` are ignored.

Malformed envelopes, malformed code identifiers or unknown protocol versions,
invalid success bodies, and transport failures produce `ServerError`, a fixed safe message, and no
correlation ID. `status` retains a received HTTP status even for an invalid
contract; transport failures have `None`. A relay should only use a 4xx or 5xx
status and otherwise fall back to 500. Upstream body text and network diagnostics
are never included in these failures, including their `Debug` and `Display`
representations. Metadata failures are identified by `MetadataUnavailable`,
independently of upstream message text.

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
