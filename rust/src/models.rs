use dg_xch_core::blockchain::sized_bytes::Bytes32;
use serde::{Deserialize, Serialize};

pub type ServerSecret = [u8; 32];

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StartSignatureRequest {
    pub requested_credentials: Vec<String>,
    pub require_site_pass: bool,
    pub required_alias_launcher: Option<Bytes32>,
    pub requested_message: Vec<u8>,
    pub expires: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StartSignatureResponse {
    pub request_id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GeneratePresentationRequest {
    pub request_id: String,
    pub nonce: Bytes32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GeneratePresentationResponse {
    pub compressed_presentation: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerPresentation {
    pub compressed_presentation: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ClientPresentation {
    pub presentation: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VerifySignatureRequest {
    pub request_id: String,
    pub presentation: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RejectSignatureRequest {
    pub request_id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Claim {
    pub property: String,
    pub value: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DIDInfo {
    pub launcher_id: [u8; 32],
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VerifySignatureResponse {
    pub alias_did: DIDInfo,
    pub site_pass: Option<Bytes32>,
    pub claims: Vec<Claim>,
    pub timestamp: i64,
    pub presentation: Vec<u8>,
    pub petname: String,
    pub lifehash: Vec<u8>,
    pub reserve_names: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SignatureRequest {
    pub requested_credentials: Vec<String>, //By Property Name
    pub require_site_pass: bool,
    pub required_alias_launcher: Option<Bytes32>,
    pub requested_message: Vec<u8>,
    pub expires: i64, //Unix UTC Timestamp
    pub nonce: Bytes32,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct VerifyHonestBotRequest {
    pub julia_challenge: Bytes32,
    pub presentation_hash: Bytes32,
}
