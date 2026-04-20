# Julia Web SDK

Multi-language SDK and server adapters for Julia verification flows.

## Scope

- One client interface per language: `SignatureClient`
- Signature route support: `/signature/*`
- Server adapters for:
  - JavaScript (Express)
  - Python (FastAPI)
  - Java (Spring MVC/WebSocket)
  - Dart (shelf)
  - Rust (`ServiceBuilder`)

## Integration Guidance

- For host websites/services, use the language-specific server adapter first so `/signature/*` routes, session tracking, and websocket proxying are handled consistently.
- Use `SignatureClient` directly when you need manual endpoint calls (custom flow, scripts, or tests).

## Endpoint Contract

Signature endpoints exposed by adapters:

- `GET /signature/notbot`
- `GET /signature/status`
- `POST /signature/notbot/{request_id}`
- `POST /signature/verify/{request_id}`
- `WS /signature/honestbot`
- `WS /calculate_site_pass`

Upstream signature endpoints consumed by clients/adapters:

- `POST /signature/start`
- `POST /signature/presentation`
- `POST /signature/verify`
- `WS /signature/honestbot`
- `WS /calculate_site_pass`

Full contract: `./shared/api_contract.md`  
Claim constants: `./shared/claim_properties.txt`

## Signature Flow Summary

1. Host calls `GET /signature/notbot` on an SDK adapter.
2. Adapter calls upstream `POST /signature/start`, stores `request_id -> session_id`, and returns `request_id`.
3. Client/not.bot submits nonce to `POST /signature/notbot/{request_id}`.
4. Adapter calls upstream `POST /signature/presentation` and returns the compressed server presentation.
5. Client/not.bot submits signed presentation to `POST /signature/verify/{request_id}`.
6. Adapter calls upstream `POST /signature/verify`; on success, callback/session state is updated.
7. Host checks signature state with `GET /signature/status`.
8. Websocket routes `/signature/honestbot` and `/calculate_site_pass` are proxied to upstream signature services.

## Repository Layout

- `./rust`: Rust crate, `ServiceBuilder`, and `SignatureClient`
- `./javascript`: JavaScript client + Express adapter
- `./python`: Python client + FastAPI adapter
- `./java`: Java client + Spring adapter
- `./dart`: Dart client + shelf adapter
- `./examples`: runnable integration/client examples for each language

## Quick Start

- JavaScript: `cd javascript && npm install`
- Python: `cd python && pip install -e .`
- Java: `cd java && mvn -q package`
- Dart: `cd dart && dart pub get`
- Rust: `cd rust && cargo check`

## Example Index

- JavaScript: `./examples/javascript/express_server.mjs`, `./examples/javascript/client_usage.mjs`
- Python: `./examples/python/fastapi_server.py`, `./examples/python/client_usage.py`
- Java: `./examples/java/SpringSdkConfig.java`, `./examples/java/SignatureClientExample.java`
- Dart: `./examples/dart/shelf_server.dart`, `./examples/dart/client_usage.dart`
- Rust: `./examples/rust/server_integration.rs`, `./examples/rust/client_usage.rs`

## Environment Variables

- `SIGNATURE_HOSTNAME` (default: `localhost`)
- `SIGNATURE_PORT` (default: `8080`)
- `SIGNATURE_API_KEY` (default: `CHANGE_ME`)

## Notes

- The Rust implementation is the reference path used for core service behavior.
- Example URLs use `https://example.com` as placeholders.
- Websocket proxy safety: configure `SignatureClient` / adapter upstream base URL to the real upstream signature service, not the same host serving SDK adapter routes. Pointing both to the same `/signature/*` host can create proxy-to-self loops for `/signature/honestbot`.
