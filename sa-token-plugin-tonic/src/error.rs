// Author: 金书记
//
//! gRPC authentication errors.

use sa_token_core::{error::messages, token::TokenValue};
use tonic::{Code, Status};

/// Token data extracted from gRPC metadata.
#[derive(Clone, Debug)]
pub struct GrpcTokenData {
    pub token: Option<TokenValue>,
    pub login_id: Option<String>,
}

/// gRPC authentication error.
#[derive(Debug)]
pub struct AuthError {
    message: &'static str,
}

impl AuthError {
    /// Create a new authentication error.
    pub fn new() -> Self {
        Self {
            message: messages::AUTH_ERROR,
        }
    }

    /// Create from custom message.
    pub fn with_message(msg: &'static str) -> Self {
        Self { message: msg }
    }

    /// Convert to tonic Status.
    pub fn into_status(self) -> Status {
        Status::new(Code::Unauthenticated, self.message)
    }
}

impl Default for AuthError {
    fn default() -> Self {
        Self::new()
    }
}

impl From<AuthError> for Status {
    fn from(err: AuthError) -> Self {
        err.into_status()
    }
}

/// gRPC permission error.
#[derive(Debug)]
pub struct PermissionError {
    message: &'static str,
}

impl PermissionError {
    /// Create a new permission error.
    pub fn new() -> Self {
        Self {
            message: messages::PERMISSION_REQUIRED,
        }
    }

    /// Create from custom message.
    pub fn with_message(msg: &'static str) -> Self {
        Self { message: msg }
    }

    /// Convert to tonic Status.
    pub fn into_status(self) -> Status {
        Status::new(Code::PermissionDenied, self.message)
    }
}

impl Default for PermissionError {
    fn default() -> Self {
        Self::new()
    }
}

impl From<PermissionError> for Status {
    fn from(err: PermissionError) -> Self {
        err.into_status()
    }
}
