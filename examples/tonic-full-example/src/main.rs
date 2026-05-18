// Author: 金书记
//
// 中文 | English
// sa-token-rust Tonic gRPC 完整示例
// sa-token-rust Tonic gRPC Full Example
//
// 展示如何：| Demonstrates how to:
// 1. 配置 sa-token | Configure sa-token
// 2. 加载用户权限和角色 | Load user permissions and roles
// 3. 使用 gRPC 拦截器 | Use gRPC interceptors
// 4. 实现完整的认证流程 | Implement complete authentication flow

use sa_token_plugin_tonic::*;
use std::sync::Arc;
use tonic::{Code, Request, Response, Status};

pub mod auth {
    tonic::include_proto!("auth");
}

// ============================================================================
// API 响应结构 | API Response Structure
// ============================================================================
// 注意: 这些结构体在代码中直接使用 proto 类型，未使用这些中间结构
// Note: These structs are not used directly, proto types are used instead

// ============================================================================
// gRPC 服务实现 | gRPC Service Implementation
// ============================================================================

/// 中文: 认证服务实现
/// English: Authentication service implementation
pub struct AuthServiceImpl {
    state: SaTokenState,
}

impl AuthServiceImpl {
    /// 中文: 创建新的服务实例
    /// English: Create new service instance
    pub fn new(state: SaTokenState) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl auth::auth_service_server::AuthService for AuthServiceImpl {
    /// 中文: 健康检查（公开接口）
    /// English: Health check (public endpoint)
    async fn health_check(
        &self,
        _request: Request<auth::HealthCheckRequest>,
    ) -> Result<Response<auth::HealthCheckResponse>, Status> {
        tracing::info!("🏥 Health check requested");

        Ok(Response::new(auth::HealthCheckResponse {
            status: "OK".to_string(),
        }))
    }

    /// 中文: 登录接口（公开接口）
    /// English: Login endpoint (public endpoint)
    async fn login(
        &self,
        request: Request<auth::LoginRequest>,
    ) -> Result<Response<auth::LoginResponse>, Status> {
        let req = request.into_inner();
        tracing::info!("🔑 Login request: username={}", req.username);

        // 中文: 验证用户名密码（这里简化处理）
        // English: Validate username and password (simplified for demo)
        let (user_id, valid) = match req.username.as_str() {
            "admin" if req.password == "admin123" => ("admin", true),
            "user" if req.password == "user123" => ("user", true),
            "guest" if req.password == "guest123" => ("guest", true),
            _ => ("", false),
        };

        if !valid {
            tracing::warn!(
                "❌ Login failed: invalid credentials for user {}",
                req.username
            );
            return Err(Status::new(
                Code::Unauthenticated,
                "Invalid username or password",
            ));
        }

        // 中文: 执行登录
        // English: Perform login
        let token = self.state.manager.login(user_id).await.map_err(|e| {
            tracing::error!("❌ Login failed: {}", e);
            Status::internal(format!("Login failed: {}", e))
        })?;

        // 中文: 获取用户权限和角色
        // English: Get user permissions and roles
        let permissions = sa_token_core::StpUtil::get_permissions(user_id).await;
        let roles = sa_token_core::StpUtil::get_roles(user_id).await;

        tracing::info!(
            "✅ User {} logged in successfully, permissions: {:?}, roles: {:?}",
            user_id,
            permissions,
            roles
        );

        Ok(Response::new(auth::LoginResponse {
            token: token.as_str().to_string(),
            user_id: user_id.to_string(),
            permissions,
            roles,
        }))
    }

    /// 中文: 获取用户信息（需要登录）
    /// English: Get user info (requires login)
    async fn get_user_info(
        &self,
        request: Request<auth::UserInfoRequest>,
    ) -> Result<Response<auth::UserInfoResponse>, Status> {
        // 中文: 验证 token 并获取登录 ID
        // English: Validate token and get login ID
        let login_id = validate_token_from_request(&self.state, &request).await?;

        tracing::info!("📋 GetUserInfo request from user: {}", login_id);

        // 中文: 获取用户权限和角色
        // English: Get user permissions and roles
        let permissions = sa_token_core::StpUtil::get_permissions(&login_id).await;
        let roles = sa_token_core::StpUtil::get_roles(&login_id).await;

        Ok(Response::new(auth::UserInfoResponse {
            user_id: login_id,
            permissions,
            roles,
        }))
    }

    /// 中文: 获取权限列表（需要登录）
    /// English: Get permissions list (requires login)
    async fn get_permissions(
        &self,
        request: Request<auth::PermissionsListRequest>,
    ) -> Result<Response<auth::PermissionsListResponse>, Status> {
        // 中文: 验证 token 并获取登录 ID
        // English: Validate token and get login ID
        let login_id = validate_token_from_request(&self.state, &request).await?;

        tracing::info!("🔐 GetPermissions request from user: {}", login_id);

        let permissions = sa_token_core::StpUtil::get_permissions(&login_id).await;

        Ok(Response::new(auth::PermissionsListResponse { permissions }))
    }

    /// 中文: 获取角色列表（需要登录）
    /// English: Get roles list (requires login)
    async fn get_roles(
        &self,
        request: Request<auth::RolesListRequest>,
    ) -> Result<Response<auth::RolesListResponse>, Status> {
        // 中文: 验证 token 并获取登录 ID
        // English: Validate token and get login ID
        let login_id = validate_token_from_request(&self.state, &request).await?;

        tracing::info!("👥 GetRoles request from user: {}", login_id);

        let roles = sa_token_core::StpUtil::get_roles(&login_id).await;

        Ok(Response::new(auth::RolesListResponse { roles }))
    }
}

// ============================================================================
// 初始化测试数据 | Initialize Test Data
// ============================================================================

/// 中文: 初始化测试用户的权限和角色
/// English: Initialize test user permissions and roles
async fn init_test_permissions() {
    tracing::info!("🔐 Initializing test user permissions...");

    // 中文: 管理员用户
    // English: Admin user
    sa_token_core::StpUtil::set_permissions(
        "admin",
        vec![
            "user:list".to_string(),
            "user:create".to_string(),
            "user:update".to_string(),
            "user:delete".to_string(),
            "system:config".to_string(),
            "admin:*".to_string(),
        ],
    )
    .await
    .unwrap();

    sa_token_core::StpUtil::set_roles("admin", vec!["admin".to_string(), "user".to_string()])
        .await
        .unwrap();

    tracing::info!("  ✓ Admin (admin) permissions initialized");

    // 中文: 普通用户
    // English: Regular user
    sa_token_core::StpUtil::set_permissions(
        "user",
        vec!["user:list".to_string(), "user:view".to_string()],
    )
    .await
    .unwrap();

    sa_token_core::StpUtil::set_roles("user", vec!["user".to_string()])
        .await
        .unwrap();

    tracing::info!("  ✓ Regular user (user) permissions initialized");

    // 中文: 访客用户
    // English: Guest user
    sa_token_core::StpUtil::set_permissions("guest", vec!["user:view".to_string()])
        .await
        .unwrap();

    sa_token_core::StpUtil::set_roles("guest", vec!["guest".to_string()])
        .await
        .unwrap();

    tracing::info!("  ✓ Guest user (guest) permissions initialized");
    tracing::info!("✅ All test user permissions initialized!\n");
}

// ============================================================================
// 主函数 | Main Function
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 中文: 初始化日志
    // English: Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("🚀 Starting sa-token-rust Tonic gRPC Full Example");
    tracing::info!("=========================================\n");

    // 中文: 1. 使用构建器模式创建 sa-token 状态
    // English: 1. Create sa-token state using builder pattern
    let sa_token_state = SaTokenState::builder()
        .storage(Arc::new(sa_token_plugin_tonic::MemoryStorage::new()))
        .token_name("satoken")
        .timeout(86400) // 中文: 24小时 | English: 24 hours
        .build();

    tracing::info!("✅ SaToken state created");

    // 中文: 2. 初始化测试用户的权限和角色
    // English: 2. Initialize test user permissions and roles
    init_test_permissions().await;

    // 中文: 3. 创建认证拦截器
    // English: 3. Create authentication interceptor
    let auth_interceptor = GrpcServerInterceptor::new(sa_token_state.clone());
    tracing::info!("✅ Authentication interceptor created");

    // 中文: 4. 创建 gRPC 服务
    // English: 4. Create gRPC service
    let service = auth::auth_service_server::AuthServiceServer::with_interceptor(
        AuthServiceImpl::new(sa_token_state.clone()),
        auth_interceptor,
    );

    // 中文: 5. 启动服务器
    // English: 5. Start the server
    let addr = "0.0.0.0:3000".parse()?;

    tracing::info!("📡 gRPC server listening on {}", addr);
    tracing::info!("");
    tracing::info!("💡 Test users:");
    tracing::info!("   - admin/admin123  (管理员 | Admin)");
    tracing::info!("   - user/user123    (普通用户 | Regular user)");
    tracing::info!("   - guest/guest123  (访客 | Guest)");
    tracing::info!("");
    tracing::info!("🔧 Test commands:");
    tracing::info!("   # 健康检查 (Health check - public)");
    tracing::info!(
        "   grpcurl -plaintext -proto proto/auth.proto {} auth.AuthService/HealthCheck",
        addr
    );
    tracing::info!("");
    tracing::info!("   # 登录 (Login - public)");
    tracing::info!("   grpcurl -plaintext -proto proto/auth.proto -d '{{\"username\": \"admin\", \"password\": \"admin123\"}}' {} auth.AuthService/Login",addr);
    tracing::info!("");
    tracing::info!("   # 获取用户信息 (Get user info - requires auth)");
    tracing::info!("   grpcurl -plaintext -proto proto/auth.proto -H \"satoken: Bearer <token>\" {} auth.AuthService/GetUserInfo",addr);
    tracing::info!("");
    tracing::info!("=========================================\n");

    tonic::transport::Server::builder()
        .add_service(service)
        .serve(addr)
        .await?;

    Ok(())
}
