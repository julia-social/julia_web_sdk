use dg_xch_core::errors::ClvmError;
use std::fmt::{Display, Formatter};

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
