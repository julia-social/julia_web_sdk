from __future__ import annotations

import asyncio
import inspect
import time
from collections.abc import Awaitable, Callable, MutableMapping
from typing import Any

import websockets
from fastapi import APIRouter, HTTPException, Request, Response, WebSocket

from .client import SignatureClient, create_signature_client_from_env
from .models import (
    ClientPresentation,
    GeneratePresentationRequest,
    ServerPresentation,
    SignatureRequest,
    StartSignatureRequest,
    VerifySignatureRequest,
    VerifySignatureResponse,
)

OnSuccess = Callable[[VerifySignatureResponse, MutableMapping[str, Any]], Awaitable[None] | None]
OnFailure = Callable[[Exception], Awaitable[None] | None]
ResolveSession = Callable[[str | None, Request], Awaitable[MutableMapping[str, Any] | None] | MutableMapping[str, Any] | None]


def _to_bytes(input_text: str) -> list[int]:
    return list(input_text.encode("utf-8"))


def _safe_request_session(request: Request) -> MutableMapping[str, Any]:
    try:
        return request.session
    except AssertionError:
        return {}


async def _maybe_await(result: Awaitable[Any] | Any) -> Any:
    if inspect.isawaitable(result):
        return await result
    return result


class FastAPIAuthAdapter:
    def __init__(
        self,
        signature_client: SignatureClient | None = None,
        requested_claims: list[str] | None = None,
        require_site_pass: bool = False,
        message_generator: Callable[[], str] | None = None,
        on_success: OnSuccess | None = None,
        on_failure: OnFailure | None = None,
        expire_time_seconds: int = 3600,
        session_key: str = "julia_signature_verification",
        session_cookie_name: str = "session",
        resolve_session: ResolveSession | None = None,
    ) -> None:
        self.signature_client = signature_client or create_signature_client_from_env()
        self.requested_claims = requested_claims or []
        self.require_site_pass = require_site_pass
        self.message_generator = message_generator or (lambda: "")
        self.on_success = on_success or self._default_on_success
        self.on_failure = on_failure or self._default_on_failure
        self.expire_time_seconds = expire_time_seconds
        self.session_key = session_key
        self.session_cookie_name = session_cookie_name
        self.resolve_session = resolve_session
        self.session_authorizations: dict[str, str | None] = {}

        self.router = APIRouter()
        self._register_routes()

    async def _default_on_success(
        self, verify_response: VerifySignatureResponse, session_data: MutableMapping[str, Any]
    ) -> None:
        session_data[self.session_key] = verify_response.model_dump()

    async def _default_on_failure(self, _error: Exception) -> None:
        return

    def _register_routes(self) -> None:
        @self.router.get("/auth/notbot")
        async def get_auth_url(request: Request) -> str:
            session_data = _safe_request_session(request)
            session_data.pop(self.session_key, None)

            try:
                response = await self.signature_client.start_signature(
                    StartSignatureRequest(
                        requested_credentials=self.requested_claims,
                        require_site_pass=self.require_site_pass,
                        required_alias_launcher=None,
                        requested_message=_to_bytes(self.message_generator()),
                        expires=int(time.time()) + self.expire_time_seconds,
                    )
                )
            except Exception as exc:  # noqa: BLE001
                await _maybe_await(self.on_failure(exc))
                raise HTTPException(status_code=502, detail=str(exc)) from exc

            self.session_authorizations[response.request_id] = request.cookies.get(
                self.session_cookie_name
            )
            return response.request_id

        @self.router.get("/auth/status")
        async def get_auth_status(request: Request) -> bool:
            session_data = _safe_request_session(request)
            return self.session_key in session_data

        @self.router.post("/auth/notbot/{request_id}")
        async def get_request_presentation(
            request_id: str, payload: SignatureRequest
        ) -> ServerPresentation:
            try:
                response = await self.signature_client.generate_presentation(
                    GeneratePresentationRequest(request_id=request_id, nonce=payload.nonce)
                )
            except Exception as exc:  # noqa: BLE001
                await _maybe_await(self.on_failure(exc))
                raise HTTPException(status_code=502, detail=str(exc)) from exc
            return ServerPresentation(
                compressed_presentation=response.compressed_presentation
            )

        @self.router.post("/auth/verify/{request_id}", status_code=204)
        async def verify_presentation(
            request: Request, request_id: str, payload: ClientPresentation
        ) -> Response:
            try:
                verify_response = await self.signature_client.verify_presentation(
                    VerifySignatureRequest(
                        request_id=request_id,
                        presentation=payload.presentation,
                    )
                )
            except Exception as exc:  # noqa: BLE001
                await _maybe_await(self.on_failure(exc))
                raise HTTPException(status_code=422, detail=str(exc)) from exc

            session_data = _safe_request_session(request)
            auth_session_id = self.session_authorizations.pop(request_id, None)
            target_session = session_data

            if self.resolve_session is not None:
                resolved = await _maybe_await(self.resolve_session(auth_session_id, request))
                if resolved is not None:
                    target_session = resolved

            try:
                await _maybe_await(self.on_success(verify_response, target_session))
            except Exception as exc:  # noqa: BLE001
                await _maybe_await(self.on_failure(exc))
                raise HTTPException(status_code=500, detail=str(exc)) from exc

            return Response(status_code=204)

        @self.router.websocket("/auth/honestbot")
        async def verify_honestbot(client_socket: WebSocket) -> None:
            await self._proxy_websocket(
                client_socket,
                upstream_path="/signature/honestbot",
                forwarded_header="x-presentation-hash",
            )

        @self.router.websocket("/calculate_site_pass")
        async def calculate_site_pass(client_socket: WebSocket) -> None:
            await self._proxy_websocket(
                client_socket,
                upstream_path="/calculate_site_pass",
                forwarded_header="x-site-pass",
            )

    async def _proxy_websocket(
        self,
        client_socket: WebSocket,
        upstream_path: str,
        forwarded_header: str,
    ) -> None:
        await client_socket.accept()

        upstream_url = (
            self.signature_client.base_url.replace("http://", "ws://")
            .replace("https://", "wss://")
            .rstrip("/")
            + upstream_path
        )

        headers: list[tuple[str, str]] = []
        header_value = client_socket.headers.get(forwarded_header)
        if header_value:
            headers.append((forwarded_header, header_value))

        try:
            async with websockets.connect(upstream_url, extra_headers=headers or None) as upstream:
                async def from_client() -> None:
                    while True:
                        message = await client_socket.receive()
                        if message["type"] == "websocket.disconnect":
                            break
                        text_value = message.get("text")
                        bytes_value = message.get("bytes")
                        if text_value is not None:
                            await upstream.send(text_value)
                        elif bytes_value is not None:
                            await upstream.send(bytes_value)

                async def from_upstream() -> None:
                    while True:
                        message = await upstream.recv()
                        if isinstance(message, str):
                            await client_socket.send_text(message)
                        else:
                            await client_socket.send_bytes(message)

                tasks = [
                    asyncio.create_task(from_client()),
                    asyncio.create_task(from_upstream()),
                ]
                done, pending = await asyncio.wait(
                    tasks,
                    return_when=asyncio.FIRST_COMPLETED,
                )
                for task in pending:
                    task.cancel()
                await asyncio.gather(*pending, return_exceptions=True)
                for task in done:
                    task.result()
        except Exception as exc:  # noqa: BLE001
            await _maybe_await(self.on_failure(exc))
        finally:
            await client_socket.close()


def create_fastapi_router(**kwargs: Any) -> APIRouter:
    adapter = FastAPIAuthAdapter(**kwargs)
    return adapter.router


__all__ = [
    "FastAPIAuthAdapter",
    "create_fastapi_router",
]
