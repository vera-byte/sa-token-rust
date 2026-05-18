// Author: 金书记
//
// 中文 | English
// gRPC 认证拦截器模块 | gRPC authentication interceptor module
//
//! 中文: 本模块提供 gRPC 专用的认证拦截器，可与 tonic 的拦截器机制配合使用
//! English: This module provides gRPC-specific authentication interceptors that can be used with tonic's interceptor mechanism

use tonic::{Request, Status, Code};

use crate::state::SaTokenState;
use crate::adapter::TonicRequestAdapter;
use sa_token_adapter::utils::extract_bearer_or_value;

// ============================================================================
// 中文: gRPC 服务端拦截器（使用 tonic 内置机制）
// English: gRPC Server Interceptor (using tonic's built-in mechanism)
// ============================================================================

/// 中文: gRPC 服务端认证拦截器
/// English: gRPC server interceptor for authentication
///
/// 中文: 这是向 tonic 服务添加认证的推荐方式
/// English: This is the recommended way to add authentication to tonic services
///
/// # 示例 | Example
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
/// // ... 使用拦截器配置 tonic 服务
/// ```
#[derive(Clone)]
pub struct GrpcServerInterceptor {
    state: SaTokenState,
}

impl GrpcServerInterceptor {
    /// 中文: 创建新的服务端拦截器
    /// English: Create a new server interceptor
    pub fn new(state: SaTokenState) -> Self {
        Self { state }
    }

    /// 中文: 验证并从 gRPC 请求中提取 Token
    /// English: Validate and extract token from gRPC request
    ///
    /// 中文: 认证成功后返回登录 ID
    /// English: Returns the login_id if authentication succeeds
    pub async fn validate_request(&self, request: &Request<()>) -> Result<String, Status> {
        let metadata = request.metadata();
        let token_name = self.state.manager.config.token_name.as_str();

        let mut token_str: Option<String> = None;

        // 中文: 按优先级尝试读取 Token
        // English: Try to read token by priority
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

        // 中文: 验证 Token 并获取登录信息
        // English: Validate token and get login info
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

    /// 中文: 获取认证状态
    /// English: Get the authentication state
    pub fn state(&self) -> &SaTokenState {
        &self.state
    }
}

impl tonic::service::Interceptor for GrpcServerInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        // 中文: 从请求中提取 Token（同步操作，因为只是读取 header）
        // English: Extract token from request (sync operation, just reading headers)
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

        // 中文: 将 token 字符串存储在请求扩展中，验证将在服务方法中异步进行
        // English: Store token string in request extensions, validation will happen async in service methods
        let mut req = request;
        if let Some(token) = token_str {
            req.extensions_mut().insert(token);
        }
        Ok(req)
    }
}

// ============================================================================
// 中文: 权限检查辅助函数
// English: Permission Check Helper Functions
// ============================================================================

/// 中文: 检查登录 ID 是否具有指定权限
/// English: Check if a login_id has the specified permission
pub async fn check_permission(login_id: &str, permission: &str) -> bool {
    sa_token_core::StpUtil::has_permission(login_id, permission).await
}

/// 中文: 检查登录 ID 是否具有任意一个指定权限
/// English: Check if a login_id has any of the specified permissions
pub async fn check_permissions(login_id: &str, permissions: &[&str]) -> bool {
    for perm in permissions {
        if sa_token_core::StpUtil::has_permission(login_id, perm).await {
            return true;
        }
    }
    false
}

/// 中文: 检查登录 ID 是否具有指定角色
/// English: Check if a login_id has the specified role
pub async fn check_role(login_id: &str, role: &str) -> bool {
    sa_token_core::StpUtil::has_role(login_id, role).await
}

/// 中文: 检查登录 ID 是否具有任意一个指定角色
/// English: Check if a login_id has any of the specified roles
pub async fn check_roles(login_id: &str, roles: &[&str]) -> bool {
    for r in roles {
        if sa_token_core::StpUtil::has_role(login_id, r).await {
            return true;
        }
    }
    false
}

// ============================================================================
// 中文: 请求上下文辅助函数
// English: Request Context Helper Functions
// ============================================================================

/// 中文: 从 gRPC metadata 创建请求适配器
/// English: Create a request adapter from gRPC metadata
pub fn create_request_adapter(
    metadata: &tonic::metadata::MetadataMap,
    method: &str,
    path: &str,
) -> TonicRequestAdapter {
    TonicRequestAdapter::from_metadata(metadata, method.to_string(), path.to_string())
}

/// 中文: 从请求扩展中提取登录 ID
/// English: Extract login_id from request extensions
pub fn get_login_id_from_request<T>(request: &Request<T>) -> Option<String> {
    request.extensions().get::<String>().cloned()
}

/// 中文: 从请求中验证 token 并获取登录 ID（异步）
/// English: Validate token from request and get login ID (async)
pub async fn validate_token_from_request<T>(
    state: &SaTokenState,
    request: &Request<T>,
) -> Result<String, Status> {
    let token_str = request
        .extensions()
        .get::<String>()
        .cloned()
        .ok_or_else(|| Status::unauthenticated("No token found in request"))?;

    let token = sa_token_core::token::TokenValue::new(token_str);

    if state.manager.is_valid(&token).await {
        if let Ok(token_info) = state.manager.get_token_info(&token).await {
            return Ok(token_info.login_id);
        }
    }

    Err(Status::unauthenticated("Invalid or expired token"))
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

        // 中文: 先登录
        // English: Login first
        let login_id = "test_user";
        let _token = sa_token_core::StpUtil::login(login_id.to_string()).await;
        let _ = sa_token_core::StpUtil::add_role(login_id, "admin").await;

        // 中文: 检查权限（用户还没有任何权限）
        // English: Check permission (user has no permissions yet)
        assert!(!check_permission(login_id, "user:read").await);

        // 中文: 添加权限
        // English: Add permission
        let _ = sa_token_core::StpUtil::add_permission(login_id, "user:read").await;
        assert!(check_permission(login_id, "user:read").await);
    }
}
