# Julia Web SDK (Python)

Python package for Julia web verification flows.

## Install

```bash
cd python
pip install -e .
```

## What this package provides

- `SignatureClient`: async client for both upstream and SDK `/signature/*` endpoints.
- `FastAPISignatureAdapter`: FastAPI router for signature routes and websocket proxying.
- `CLAIM_PROPERTIES`: claim constants.

## Recommended Integration

- Use `FastAPISignatureAdapter` in your backend service as the default integration path.
- Use `SignatureClient` directly for custom/manual endpoint calls or automation scripts.

## Client Methods

- Signature SDK:
  - `get_signature_request_id()`
  - `get_signature_status()`
  - `generate_signature_presentation(request_id, nonce)`
  - `verify_signature_presentation(request_id, presentation)`
- Signature:
  - `start_signature(request)`
  - `generate_presentation(request)`
  - `verify_presentation(request)`

## FastAPI Integration

```python
from fastapi import FastAPI
from starlette.middleware.sessions import SessionMiddleware

from julia_web_sdk import CLAIM_PROPERTIES, FastAPISignatureAdapter, create_signature_client_from_env

app = FastAPI()
app.add_middleware(SessionMiddleware, secret_key="replace-me")

adapter = FastAPISignatureAdapter(
    signature_client=create_signature_client_from_env(),
    requested_claims=[
        CLAIM_PROPERTIES["Notbot0"],
        CLAIM_PROPERTIES["SitePass"],
        CLAIM_PROPERTIES["FirstName"],
        CLAIM_PROPERTIES["AgeOver18"],
    ],
    require_site_pass=True,
    message_generator=lambda: "Verifying My Identity with example.com",
)

app.include_router(adapter.router)
```

## Client Usage

```python
import asyncio

from julia_web_sdk import SignatureClient


async def main() -> None:
    client = SignatureClient("https://example.com")
    request_id = await client.get_signature_request_id()
    status = await client.get_signature_status()
    presentation = await client.generate_signature_presentation(
        request_id,
        "0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
    )
    await client.verify_signature_presentation(request_id, [])
    print(request_id, status)
    await client.close()


asyncio.run(main())
```

## Examples

- `../examples/python/fastapi_server.py`
- `../examples/python/client_usage.py`
