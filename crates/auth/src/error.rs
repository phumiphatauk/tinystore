//! Authentication error types

use thiserror::Error;

/// Authentication error types
#[derive(Error, Debug, Clone)]
pub enum AuthError {
    #[error("Missing authorization header")]
    MissingAuthorizationHeader,

    #[error("Invalid authorization header: {0}")]
    InvalidAuthorizationHeader(String),

    #[error("Missing credential in authorization header")]
    MissingCredential,

    #[error("Invalid credential format: {0}")]
    InvalidCredentialFormat(String),

    #[error("Unknown access key: {0}")]
    UnknownAccessKey(String),

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Invalid date format: {0}")]
    InvalidDateFormat(String),

    #[error("Invalid HMAC key: {0}")]
    InvalidHmacKey(String),

    #[error("Request timestamp too old or too far in future")]
    InvalidTimestamp,

    #[error("Missing required header: {0}")]
    MissingRequiredHeader(String),

    #[error("Internal authentication error: {0}")]
    InternalError(String),
}

/// Result type for authentication operations
pub type AuthResult<T> = Result<T, AuthError>;

impl AuthError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            AuthError::MissingAuthorizationHeader => 401,
            AuthError::InvalidAuthorizationHeader(_) => 400,
            AuthError::MissingCredential => 400,
            AuthError::InvalidCredentialFormat(_) => 400,
            AuthError::UnknownAccessKey(_) => 403,
            AuthError::SignatureVerificationFailed => 403,
            AuthError::InvalidDateFormat(_) => 400,
            AuthError::InvalidHmacKey(_) => 500,
            AuthError::InvalidTimestamp => 403,
            AuthError::MissingRequiredHeader(_) => 400,
            AuthError::InternalError(_) => 500,
        }
    }

    /// Get the S3 error code for this error
    pub fn s3_code(&self) -> &'static str {
        match self {
            AuthError::MissingAuthorizationHeader => "AccessDenied",
            AuthError::InvalidAuthorizationHeader(_) => "InvalidArgument",
            AuthError::MissingCredential => "InvalidArgument",
            AuthError::InvalidCredentialFormat(_) => "InvalidArgument",
            AuthError::UnknownAccessKey(_) => "InvalidAccessKeyId",
            AuthError::SignatureVerificationFailed => "SignatureDoesNotMatch",
            AuthError::InvalidDateFormat(_) => "InvalidArgument",
            AuthError::InvalidHmacKey(_) => "InternalError",
            AuthError::InvalidTimestamp => "RequestTimeTooSkewed",
            AuthError::MissingRequiredHeader(_) => "InvalidArgument",
            AuthError::InternalError(_) => "InternalError",
        }
    }
}

impl From<hmac::digest::InvalidLength> for AuthError {
    fn from(err: hmac::digest::InvalidLength) -> Self {
        AuthError::InvalidHmacKey(err.to_string())
    }
}
