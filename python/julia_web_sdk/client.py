from __future__ import annotations

import os
from dataclasses import dataclass
from typing import Any

import httpx

from .models import (
    GeneratePresentationRequest,
    GeneratePresentationResponse,
    SignatureRequest,
    StartSignatureRequest,
    StartSignatureResponse,
    VerifySignatureRequest,
    VerifySignatureResponse,
)


class JuliaWebSdkError(Exception):
    def __init__(self, message: str, status_code: int | None = None, body: Any = None) -> None:
        super().__init__(message)
        self.status_code = status_code
        self.body = body


@dataclass
class SignatureClientConfig:
    base_url: str
    api_key: str
    timeout_seconds: float = 180.0


class SignatureClient:
    def __init__(
        self,
        config: SignatureClientConfig | str,
        api_key: str | httpx.AsyncClient | None = None,
        timeout_seconds: float = 180.0,
        client: httpx.AsyncClient | None = None,
    ) -> None:
        resolved_client = client
        resolved_api_key = api_key
        if isinstance(api_key, httpx.AsyncClient) and client is None:
            resolved_client = api_key
            resolved_api_key = None

        if isinstance(config, SignatureClientConfig):
            self._config = config
        else:
            self._config = SignatureClientConfig(
                base_url=config,
                api_key=resolved_api_key or os.getenv("SIGNATURE_API_KEY", "CHANGE_ME"),
                timeout_seconds=timeout_seconds,
            )
        self._external_client = resolved_client
        self._client = resolved_client or httpx.AsyncClient(timeout=self._config.timeout_seconds)

    @property
    def base_url(self) -> str:
        return self._config.base_url.rstrip("/")

    @property
    def api_key(self) -> str:
        return self._config.api_key

    async def close(self) -> None:
        if self._external_client is None:
            await self._client.aclose()

    async def start_signature(self, request: StartSignatureRequest) -> StartSignatureResponse:
        payload = await self._request_signature("/signature/start", request.model_dump())
        return StartSignatureResponse.model_validate(payload)

    async def generate_presentation(
        self, request: GeneratePresentationRequest
    ) -> GeneratePresentationResponse:
        payload = await self._request_signature("/signature/presentation", request.model_dump())
        return GeneratePresentationResponse.model_validate(payload)

    async def verify_presentation(
        self, request: VerifySignatureRequest
    ) -> VerifySignatureResponse:
        payload = await self._request_signature("/signature/verify", request.model_dump())
        return VerifySignatureResponse.model_validate(payload)

    async def get_auth_request_id(self) -> str:
        payload = await self._request_auth("GET", "/auth/notbot")
        if isinstance(payload, str):
            return payload
        raise JuliaWebSdkError("Invalid /auth/notbot response type", body=payload)

    async def get_auth_status(self) -> bool:
        payload = await self._request_auth("GET", "/auth/status")
        if isinstance(payload, bool):
            return payload
        raise JuliaWebSdkError("Invalid /auth/status response type", body=payload)

    async def generate_auth_presentation(
        self, request_id: str, nonce: str
    ) -> GeneratePresentationResponse:
        payload = await self._request_auth(
            "POST",
            f"/auth/notbot/{request_id}",
            json=SignatureRequest(nonce=nonce).model_dump(),
        )
        return GeneratePresentationResponse.model_validate(payload)

    async def verify_auth_presentation(self, request_id: str, presentation: list[int]) -> None:
        payload = {"presentation": presentation}
        await self._request_auth("POST", f"/auth/verify/{request_id}", json=payload)

    async def _request_signature(self, path: str, payload: dict[str, Any]) -> dict[str, Any]:
        url = f"{self.base_url}{path}"
        try:
            response = await self._client.post(
                url,
                json=payload,
                headers={"api-key": self._config.api_key},
            )
        except Exception as exc:  # noqa: BLE001
            raise JuliaWebSdkError(f"Error connecting to signature service: {exc}") from exc

        data: Any
        raw = response.text
        try:
            data = response.json() if raw else None
        except Exception as exc:  # noqa: BLE001
            raise JuliaWebSdkError(
                f"Error parsing signature response JSON: {exc} - {raw}",
                status_code=response.status_code,
                body=raw,
            ) from exc

        if response.is_error:
            raise JuliaWebSdkError(
                f"Signature service error {response.status_code}: {raw}",
                status_code=response.status_code,
                body=data,
            )

        if not isinstance(data, dict):
            raise JuliaWebSdkError(
                "Signature service returned non-object response",
                status_code=response.status_code,
                body=data,
            )

        return data

    async def _request_auth(
        self, method: str, path: str, json: dict[str, Any] | None = None
    ) -> Any:
        try:
            response = await self._client.request(
                method,
                f"{self.base_url}{path}",
                json=json,
            )
        except Exception as exc:  # noqa: BLE001
            raise JuliaWebSdkError(f"Error connecting to auth service: {exc}") from exc

        raw = response.text
        data: Any
        try:
            data = response.json() if raw else None
        except Exception as exc:  # noqa: BLE001
            raise JuliaWebSdkError(
                f"Error parsing auth response JSON: {exc} - {raw}",
                status_code=response.status_code,
                body=raw,
            ) from exc

        if response.is_error:
            raise JuliaWebSdkError(
                f"Auth service error {response.status_code}: {raw}",
                status_code=response.status_code,
                body=data,
            )

        return data


def create_signature_client_from_env() -> SignatureClient:
    host = os.getenv("SIGNATURE_HOSTNAME", "localhost")
    port = int(os.getenv("SIGNATURE_PORT", "8080"))
    api_key = os.getenv("SIGNATURE_API_KEY", "CHANGE_ME")
    secure = host != "localhost"
    scheme = "https" if secure else "http"
    return SignatureClient(
        SignatureClientConfig(base_url=f"{scheme}://{host}:{port}", api_key=api_key)
    )


__all__ = [
    "JuliaWebSdkError",
    "SignatureClient",
    "SignatureClientConfig",
    "create_signature_client_from_env",
    "GeneratePresentationRequest",
    "GeneratePresentationResponse",
    "StartSignatureRequest",
    "StartSignatureResponse",
    "VerifySignatureRequest",
    "VerifySignatureResponse",
]
