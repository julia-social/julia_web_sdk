use dg_xch_core::errors::ClvmError;
use std::fmt::{Display, Formatter};

/// Safe failure returned only by the opt-in `SignatureClient` v1 methods.
///
/// `status` preserves the upstream HTTP status when a response was received.
/// Transport and invalid-contract failures use `ServerError` and no correlation
/// ID. Only a validated v1 envelope can supply an upstream code and UUID.
/// Messages are SDK-owned text; upstream messages and transport details are never
/// included, even in `Debug` or `Display` output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureFailure {
    pub version: u8,
    pub code: String,
    pub message: String,
    pub correlation_id: Option<String>,
    pub status: Option<u16>,
}

impl SignatureFailure {
    pub(crate) fn internal(status: Option<u16>) -> Self {
        Self {
            version: 1,
            code: "ServerError".into(),
            message: "The request could not be completed".into(),
            correlation_id: None,
            status,
        }
    }

    pub(crate) fn from_envelope(body: &[u8], status: u16) -> Option<Self> {
        #[derive(serde::Deserialize)]
        struct Envelope {
            version: u8,
            code: String,
            // Required by the contract, but never trusted as safe output.
            #[serde(rename = "message")]
            _message: String,
            correlation_id: String,
        }

        let envelope: Envelope = serde_json::from_slice(body).ok()?;
        if envelope.version != 1
            || envelope._message.trim().is_empty()
            || envelope.code.len() > 64
            || !envelope.code.starts_with(|c: char| c.is_ascii_uppercase())
            || !envelope.code.bytes().all(|c| c.is_ascii_alphanumeric())
        {
            return None;
        }
        // Require the canonical hyphenated UUID shape, not arbitrary body text.
        let correlation = uuid::Uuid::parse_str(&envelope.correlation_id).ok()?;
        if !correlation
            .hyphenated()
            .to_string()
            .eq_ignore_ascii_case(&envelope.correlation_id)
        {
            return None;
        }
        let message = match envelope.code.as_str() {
            "InvalidSession" => "Invalid session",
            "SessionExpired" => "Session expired",
            "ExistingIdentity" => "An identity already exists; recovery is required",
            "PresentationNotFound" => "Presentation not found",
            "MissingCredential" => "A required credential was not presented",
            "SitePassRecalculationRequired" => "Site pass must be recalculated",
            "AlreadyUpgraded" => "Identity already upgraded",
            "NotNewPerson" => "Identity cannot be upgraded",
            "UpgradeTooSoon" => "Please wait before retrying the upgrade",
            "RecoveryDataNotFound" => "Recovery data not found",
            "RequestNotActive" => "Request is no longer active",
            "MetadataUnavailable" => "Signature metadata is unavailable",
            "ProtocolUnsupported" => "Unsupported server protocol",
            "NonceNotFound" => "Device linking code was not found",
            "UnsupportedPresentation" => "Unsupported presentation version",
            // Generic and future wire codes receive no application-specific
            // meaning. Preserve only the bounded identifier, never raw text.
            _ => "The request could not be completed",
        };
        Some(Self {
            version: 1,
            code: envelope.code,
            message: message.into(),
            correlation_id: Some(envelope.correlation_id),
            status: Some(status),
        })
    }
}

impl Display for SignatureFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.code, self.message)
    }
}

impl std::error::Error for SignatureFailure {}

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub code: ErrorCode,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?} - {}", self.code, self.message))
    }
}

impl From<ClvmError> for Error {
    fn from(e: ClvmError) -> Self {
        Self {
            message: e.to_string(),
            code: ErrorCode::Clvm,
        }
    }
}

impl std::error::Error for Error {}
impl Error {
    pub fn new<S: Display>(code: ErrorCode, message: S) -> Self {
        Self {
            message: message.to_string(),
            code,
        }
    }
    pub fn input<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::InvalidInput,
        }
    }
    pub fn parser<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::Parser,
        }
    }
    pub fn hasher<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::Hasher,
        }
    }
    pub fn argon<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::Argon2,
        }
    }
    pub fn client<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::Connection,
        }
    }
    pub fn connection<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::Connection,
        }
    }
    pub fn peers<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::NoPeers,
        }
    }
    pub fn garbler<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::Garbler,
        }
    }
    pub fn evaluator<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::Evaluator,
        }
    }
    pub fn retries<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::TooManyRetries,
        }
    }
    pub fn signature<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::InvalidSignature,
        }
    }
    pub fn credential<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::InvalidCredential,
        }
    }
    pub fn proof<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::InvalidProof,
        }
    }
    pub fn not_found<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::NotFound,
        }
    }
    pub fn io<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::IO,
        }
    }
    pub fn database<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::Database,
        }
    }
    pub fn join<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::Join,
        }
    }
    pub fn clvm<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::Clvm,
        }
    }
    pub fn sqids<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::Sqids,
        }
    }
    pub fn encryption<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::Encryption,
        }
    }
    pub fn compression<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::Compression,
        }
    }
    pub fn validation<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::Validation,
        }
    }
    pub fn unreachable<T>(_: T) -> Self {
        Self {
            message: "This Error Should Never Happen".to_string(),
            code: ErrorCode::Unreachable,
        }
    }
    pub fn timed_out<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::Validation,
        }
    }
    pub fn other<S: Display>(message: S) -> Self {
        Self {
            message: message.to_string(),
            code: ErrorCode::Other,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
#[repr(i64)]
pub enum ErrorCode {
    // Continue/Retry 3xx
    TimedOut = 300,
    Database = 301,

    // Client Fatal 4xx
    InvalidInput = 400,
    NotFound = 404,
    BodyRead = 409,
    InvalidJson = 422,
    TooManyRetries = 429,
    NoPeers = 497,
    Connection = 498,
    CreateClient = 499,

    // Server Fatal 5xx
    ServerError = 500,
    Unreachable = 503,
    IO = 510,
    Join = 511,
    Encryption = 512,
    Compression = 513,
    Hasher = 514,
    Parser = 515,
    Argon2 = 516,
    Sqids = 517,
    Validation = 518,

    // Chia Fatal 6xx
    Clvm = 600,
    //MPC
    Garbler = 601,
    Evaluator = 602,
    //BLS
    InvalidSignature = 603,
    InvalidCredential = 604,
    InvalidProof = 605,

    //Other
    Other = 999,
}
