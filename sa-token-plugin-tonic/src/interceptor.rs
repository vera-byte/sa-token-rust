// Author: 金书记
//
//! gRPC interceptors for Sa-Token authentication.
//!
//! This module provides gRPC-specific authentication interceptors that can be used
//! with tonic's interceptor mechanism.

use tonic::{Request, Status, Code};

use crate::state::SaTokenState;
use crate::adapter::TonicRequestAdapter;
use sa_token_adapter::utils::extract_bearer_or_value;

// ============================================================================
// gRPC Server Interceptor (tonic's built-in mechanism)
// ============================================================================

/// gRPC server interceptor for authentication.
///
/// This is the recommended way to add authentication to tonic services.
/// Use with `tonic::transport::Server::builder().intercept(...)`.
///
/// # Example
///
/// ```ignore
/// use sa_token_plugin_tonic::{SaTokenState, MemoryStorage, GrpcServerInterceptor};
/// use std::sync::Arc;
///
/// let state = SaTokenState::builder()
///     .storage(Arc::new(MemoryStorage::new()))
///     .build();
///
/// let interceptor = GrpcServerInterceptor::new(state);
///
/// tonic::transport::Server::builder()
///     .add_service(my_service)
///     .interpolate_services()
///     .build()
///     .await?;
/// ```
#[derive(Clone)]
pub struct GrpcServerInterceptor {
    state: SaTokenState,
}

impl GrpcServerInterceptor {
    /// Create a new server interceptor.
    pub fn new(state: SaTokenState) -> Self {
        Self { state }
    }

    /// Validate and extract token from gRPC request.
    /// Returns the login_id if authentication succeeds.
    pub async fn validate_request(&self, request: &Request<()>) -> Result<String, Status> {
        let metadata = request.metadata();
        let token_name = self.state.manager.config.token_name.as_str();

        let mut token_str: Option<String> = None;

        if let Some(header_val) = metadata.get(token_name)
            .or_else(|| metadata.get("authorization"))
            .or_else(|| metadata.get("Authorization"))
        {
            if let Ok(s) = header_val.to_str() {
                let v = extract_bearer_or_value(s);
                if !v.is_empty() {
                    token_str = Some(v);
                }
            }
        }

        if let Some(token_str) = token_str {
            let token = sa_token_core::token::TokenValue::new(token_str);

            if self.state.manager.is_valid(&token).await {
                if let Ok(token_info) = self.state.manager.get_token_info(&token).await {
                    return Ok(token_info.login_id);
                }
            }
        }

        Err(Status::new(Code::Unauthenticated, "Missing or invalid authentication token"))
    }

    /// Get the authentication state.
    pub fn state(&self) -> &SaTokenState {
        &self.state
    }
}

impl tonic::service::Interceptor for GrpcServerInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        // Use blocking validation for interceptor (synchronous context)
        let rt = tokio::runtime::Handle::current();
        let login_id = rt.block_on(self.validate_request(&request))?;

        let mut req = request;
        req.extensions_mut().insert(login_id);
        Ok(req)
    }
}

// ============================================================================
// Permission Check Helper
// ============================================================================

/// Check if a login_id has the specified permission.
pub async fn check_permission(login_id: &str, permission: &str) -> bool {
    sa_token_core::StpUtil::has_permission(login_id, permission).await
}

/// Check if a login_id has any of the specified permissions.
pub async fn check_permissions(login_id: &str, permissions: &[&str]) -> bool {
    for perm in permissions {
        if sa_token_core::StpUtil::has_permission(login_id, perm).await {
            return true;
        }
    }
    false
}

/// Check if a login_id has the specified role.
pub async fn check_role(login_id: &str, role: &str) -> bool {
    sa_token_core::StpUtil::has_role(login_id, role).await
}

/// Check if a login_id has any of the specified roles.
pub async fn check_roles(login_id: &str, roles: &[&str]) -> bool {
    for r in roles {
        if sa_token_core::StpUtil::has_role(login_id, r).await {
            return true;
        }
    }
    false
}

// ============================================================================
// Request Context Helper
// ============================================================================

/// Create a request adapter from gRPC metadata.
pub fn create_request_adapter(
    metadata: &tonic::metadata::MetadataMap,
    method: &str,
    path: &str,
) -> TonicRequestAdapter {
    TonicRequestAdapter::from_metadata(metadata, method.to_string(), path.to_string())
}

/// Extract login_id from request extensions.
pub fn get_login_id_from_request(request: &Request<()>) -> Option<String> {
    request.extensions().get::<String>().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_permission() {
        use std::sync::Arc;
        use sa_token_storage_memory::MemoryStorage;

        let _state = SaTokenState::builder()
            .storage(Arc::new(MemoryStorage::new()))
            .build();

        // Login first
        let login_id = "test_user";
        let _token = sa_token_core::StpUtil::login(login_id.to_string()).await;
        let _ = sa_token_core::StpUtil::add_role(login_id, "admin").await;

        // Check permission (user has no permissions yet)
        assert!(!check_permission(login_id, "user:read").await);

        // Add permission
        let _ = sa_token_core::StpUtil::add_permission(login_id, "user:read").await;
        assert!(check_permission(login_id, "user:read").await);
    }
}
