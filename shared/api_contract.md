# Julia Web SDK API Contract (v1.0.0)

This document defines the shared contract implemented by all language adapters and clients in this folder.

## Data Encoding

- `Bytes32`: 0x-prefixed 64-char lowercase hex string.
- `Vec<u8>` / `byte[]`: JSON array of integers (`0..255`) to match Rust `serde` defaults.

## Upstream Signature Service Endpoints

These are called by server adapters and signature clients.

### `POST /signature/start`
Request:
```json
{
  "requested_credentials": ["julia://./v1/pii/age_over_18"],
  "require_site_pass": true,
  "required_alias_launcher": null,
  "requested_message": [86, 101, 114, 105, 102, 121],
  "expires": 1730000000
}
```
Response:
```json
{
  "request_id": "req_123"
}
```

### `POST /signature/presentation`
Request:
```json
{
  "request_id": "req_123",
  "nonce": "0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
}
```
Response:
```json
{
  "compressed_presentation": [1, 2, 3]
}
```

### `POST /signature/verify`
Request:
```json
{
  "request_id": "req_123",
  "presentation": [1, 2, 3]
}
```
Response:
```json
{
  "alias_did": {
    "launcher_id": [0, 1, 2, 3]
  },
  "site_pass": "0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
  "claims": [
    {
      "property": "julia://./v1/pii/age_over_18",
      "value": [1]
    }
  ],
  "timestamp": 1730000000,
  "presentation": [1, 2, 3]
}
```

### `WS /signature/honestbot`
- Header passthrough: `x-presentation-hash`.
- Binary/text frames are proxied in both directions.

### `WS /calculate_site_pass`
- Header passthrough: `x-site-pass`.
- Binary/text frames are proxied in both directions.

## SDK Auth Endpoints

These are exposed by each language server adapter.

### `GET /auth/notbot`
Starts a signature request and returns a request id.

Response:
```json
"req_123"
```

### `GET /auth/status`
Returns `true` when the current session already has verification data.

Response:
```json
true
```

### `POST /auth/notbot/{requestId}`
Request:
```json
{
  "nonce": "0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
}
```
Response:
```json
{
  "compressed_presentation": [1, 2, 3]
}
```

### `POST /auth/verify/{requestId}`
Request:
```json
{
  "presentation": [1, 2, 3]
}
```
Response:
- `204` on success.

### `WS /auth/honestbot`
Proxies frames to upstream `WS /signature/honestbot`.

### `WS /calculate_site_pass`
Proxies frames to upstream `WS /calculate_site_pass`.

## Required Environment Variables

- `SIGNATURE_HOSTNAME` (default: `localhost`)
- `SIGNATURE_PORT` (default: `8080`)
- `SIGNATURE_API_KEY` (default: `CHANGE_ME`)

## Session Behavior

- `request_id -> session_id` mapping must be stored server-side.
- On verify success, adapter calls `on_success(verify_response, session)`.
- On verify failure, adapter calls `on_failure(error)`.
