# sa-token-plugin-tonic

中文 | English

Tonic gRPC 框架集成插件 | Tonic gRPC framework integration plugin

---

## 概述 | Overview

中文: 本插件为基于 [Tonic](https://github.com/hyperium/tonic) 框架构建的 gRPC 服务提供认证和授权支持。在不使用传统 Web 框架的情况下实现 sa-token 的强大认证功能。

English: This plugin provides authentication and authorization support for gRPC services built with the [Tonic](https://github.com/hyperium/tonic) framework. It allows you to use sa-token's powerful authentication features in your gRPC services without relying on traditional web frameworks.

---

## 功能特性 | Features

| 中文 | English |
|------|---------|
| gRPC 拦截器 | gRPC Interceptors |
| 请求适配 | Request Adaptation |
| 状态管理 | State Management |
| 权限检查 | Permission Checking |
| 错误类型 | Error Types |

---

## 安装 | Installation

添加到 `Cargo.toml`:

```toml
[dependencies]
# 中文: 默认使用内存存储 | English: Default using memory storage
sa-token-plugin-tonic = "0.1.15"

# 中文: 或使用特定特性 | English: Or with specific features
sa-token-plugin-tonic = { version = "0.1.15", features = ["redis"] }
```

### 特性说明 | Features

| 特性 Feature | 说明 Description |
|-------------|-----------------|
| `tonic-012` (默认/默认) | 启用 Tonic 0.12 支持 Enable Tonic 0.12 support |
| `memory` (默认/默认) | 启用内存存储 Enable in-memory storage |
| `redis` | 启用 Redis 存储 Enable Redis storage |
| `database` | 启用数据库存储 Enable database storage |
| `full` | 启用所有存储后端 Enable all storage backends |

---

## 快速开始 | Quick Start

### 1. 初始化状态 | Initialize State

```rust
use sa_token_plugin_tonic::{SaTokenState, MemoryStorage};
use std::sync::Arc;

let state = SaTokenState::builder()
    .storage(Arc::new(MemoryStorage::new()))
    .timeout(7200)  // 中文: Token 超时时间（秒） | English: Token timeout in seconds
    .build();
```

### 2. 创建并使用拦截器 | Create and Use Interceptor

```rust
use sa_token_plugin_tonic::GrpcServerInterceptor;

let interceptor = GrpcServerInterceptor::new(state);

// 中文: 与 tonic server 配合使用
// English: Use with tonic server
tonic::transport::Server::builder()
    .add_service(my_service)
    .build()
    .await?;
```

### 3. 在服务实现中获取登录 ID | Access Login ID in Service Implementation

```rust
use sa_token_plugin_tonic::get_login_id_from_request;
use tonic::{Request, Response, Status};

async fn my_unary_call(
    request: Request<()>,
) -> Result<Response<MyResponse>, Status> {
    // 中文: 从请求扩展中获取已认证的登录 ID
    // English: Get the authenticated login_id from request extensions
    let login_id = get_login_id_from_request(&request)
        .ok_or_else(|| Status::auth("Not authenticated"))?;

    // 中文: 使用 login_id 进行授权
    // English: Use login_id for authorization
    println!("Authenticated user: {}", login_id);

    Ok(Response::new(MyResponse { /* ... */ }))
}
```

### 4. 权限检查 | Permission Checking

```rust
use sa_token_plugin_tonic::{check_permission, check_role};

async fn admin_endpoint(request: Request<()>) -> Result<Response<()>, Status> {
    let login_id = get_login_id_from_request(&request)
        .ok_or_else(|| Status::auth("Not authenticated"))?;

    // 中文: 检查权限
    // English: Check permission
    if !check_permission(&login_id, "admin:write").await {
        return Err(Status::permission_denied("Insufficient permissions"));
    }

    // 中文: 检查角色
    // English: Check role
    if !check_role(&login_id, "admin").await {
        return Err(Status::permission_denied("Admin role required"));
    }

    Ok(Response::new(()))
}
```

---

## APISIX 透传模式 | APISIX Pass-through Mode

中文: 如果使用 APISIX 作为网关透传 Token，本插件无需任何额外配置。APISIX 将客户端的 gRPC metadata 原样转发到后端服务，插件直接从 metadata 中读取并验证 Token。

English: If using APISIX as a gateway to pass through tokens, no additional configuration is needed for this plugin. APISIX forwards the client's gRPC metadata to the backend service as-is, and the plugin reads and validates the token directly from the metadata.

### 客户端请求示例 | Client Request Example

```bash
# 中文: 方式1: satoken header
# English: Method 1: satoken header
grpcurl -H "satoken: Bearer your-token-here" \
       -plaintext localhost:50051 my.service/Echo

# 中文: 方式2: authorization header
# English: Method 2: authorization header
grpcurl -H "authorization: Bearer your-token-here" \
       -plaintext localhost:50051 my.service/Echo
```

### APISIX 配置示例 | APISIX Configuration Example

```yaml
routes:
  - uri: /grpc.service/*
    plugins:
      grpc-server-proxy:
        proto: "./proto/service.proto"
    upstream:
      type: roundrobin
      nodes:
        - host: 127.0.0.1
          port: 50051
          weight: 100
```

---

## Token 读取优先级 | Token Reading Priority

| 优先级 Priority | Header 名称 Header Name | 支持格式 Supported Format |
|----------------|------------------------|--------------------------|
| 1 | `satoken` (默认/default) | `Bearer xxx` 或 or 直接 `xxx` |
| 2 | `authorization` | `Bearer xxx` |
| 3 | `Authorization` | `Bearer xxx` |

---

## 错误处理 | Error Handling

中文: 插件提供 gRPC 专用错误类型

English: The plugin provides gRPC-specific error types

| gRPC Status | 中文条件 | English Condition |
|------------|---------|-------------------|
| `Unauthenticated` | 缺少或无效 Token | Missing or invalid token |
| `PermissionDenied` | Token 有效但权限不足 | Token valid but insufficient permissions |

---

## 与 HTTP 插件的对比 | Comparison with HTTP Plugins

| 对比项 | Comparison | HTTP 插件 | HTTP Plugin | 本插件 | This Plugin |
|--------|------------|-----------|-------------|--------|-------------|
| 工作方式 | Working Mode | HTTP Layer/Middleware | 直接透传 | Direct Pass-through |
| 认证机制 | Auth Mechanism | gRPC Interceptor | gRPC Interceptor |
| Token 来源 | Token Source | HTTP Headers | gRPC Metadata |

---

## 许可证 | License

MIT OR Apache-2.0
