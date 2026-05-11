from __future__ import annotations

from pydantic import BaseModel, Field


class StartSignatureRequest(BaseModel):
    requested_credentials: list[str]
    require_site_pass: bool
    required_alias_launcher: str | None = None
    requested_message: list[int]
    expires: int


class StartSignatureResponse(BaseModel):
    request_id: str


class GeneratePresentationRequest(BaseModel):
    request_id: str
    nonce: str


class GeneratePresentationResponse(BaseModel):
    compressed_presentation: list[int]


class ServerPresentation(BaseModel):
    compressed_presentation: list[int]


class ClientPresentation(BaseModel):
    presentation: list[int]


class VerifySignatureRequest(BaseModel):
    request_id: str
    presentation: list[int]


class Claim(BaseModel):
    property: str
    value: list[int]


class DIDInfo(BaseModel):
    launcher_id: list[int]


class VerifySignatureResponse(BaseModel):
    alias_did: DIDInfo
    site_pass: str | None = None
    claims: list[Claim] = Field(default_factory=list)
    timestamp: int
    presentation: list[int]
    petname: str
    lifehash: list[int]
    reserve_names: list[str]


class SignatureRequest(BaseModel):
    nonce: str
