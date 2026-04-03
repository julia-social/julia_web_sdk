from fastapi import FastAPI
from starlette.middleware.sessions import SessionMiddleware

from julia_web_sdk import CLAIM_PROPERTIES, FastAPIAuthAdapter, create_signature_client_from_env

app = FastAPI()
app.add_middleware(
    SessionMiddleware,
    secret_key="replace-me",
    https_only=False,
)

adapter = FastAPIAuthAdapter(
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


@app.get("/health")
async def health() -> dict[str, bool]:
    return {"ok": True}
