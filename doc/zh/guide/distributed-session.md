# 分布式 Session 管理

## 中文

### 概述

分布式 Session 管理模块支持跨多个微服务共享 Session。它提供服务认证、跨服务 Session 访问和属性管理，并自动处理超时。

本模块专为微服务架构设计，允许多个服务无缝共享用户认证状态和会话数据。

### 架构

```text
┌────────────────────────────────────────────────────────────────────┐
│                   微服务架构                                       │
└────────────────────────────────────────────────────────────────────┘

    ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
    │  服务 A      │  │  服务 B      │  │  服务 C      │
    │  (用户 API)  │  │  (订单 API)  │  │  (支付 API)  │
    └──────┬───────┘  └──────┬───────┘  └──────┬───────┘
           │                  │                  │
           └──────────────────┼──────────────────┘
                              │
                    ┌─────────▼──────────┐
                    │  分布式 Session     │
                    │  存储后端           │
                    │  (Redis/数据库)     │
                    └────────────────────┘

每个服务可以：
  - 为用户创建会话
  - 访问其他服务创建的会话
  - 共享用户认证状态
```

### 核心功能

- **跨服务 Session 共享** - 在微服务间共享 Session
- **服务认证** - 使用密钥验证服务凭证
- **Session 属性** - 存储自定义键值对用于用户上下文
- **多 Session 支持** - 一个用户可以有多个 Session（多设备）
- **自动清理** - 基于 TTL 的 Session 过期
- **可插拔存储** - 使用自定义存储后端（Redis、数据库、内存）
- **基于权限的访问** - 通过服务权限进行细粒度控制
- **会话监控** - 跟踪每个用户的所有活跃会话

### 快速开始

```rust
use sa_token_core::{
    DistributedSessionManager, InMemoryDistributedStorage, ServiceCredential
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建分布式 Session 管理器
    let storage = Arc::new(InMemoryDistributedStorage::new());
    let manager = DistributedSessionManager::new(
        storage,
        "service-main".to_string(),
        Duration::from_secs(3600), // 1 小时 TTL
    );
    
    // 注册服务
    let credential = ServiceCredential {
        service_id: "api-gateway".to_string(),
        service_name: "API Gateway".to_string(),
        secret_key: "secret123".to_string(),
        created_at: Utc::now(),
        permissions: vec!["read".to_string(), "write".to_string()],
    };
    manager.register_service(credential).await;
    
    // 验证服务
    let verified = manager.verify_service("api-gateway", "secret123").await?;
    
    // 创建 Session
    let session = manager.create_session(
        "user123".to_string(),
        "token456".to_string(),
    ).await?;
    
    // 设置 Session 属性
    manager.set_attribute(
        &session.session_id,
        "role".to_string(),
        "admin".to_string(),
    ).await?;
    
    Ok(())
}
```

### 服务认证流程

```text
服务 A                      管理器                     服务 B
   |                           |                           |
   |-- 注册服务 ------------->|                           |
   |<----- 已注册 ------------|                           |
   |                           |                           |
   |                           |<-- 验证服务(id, secret) --|
   |                           |--- 检查凭证 ------------->|
   |                           |<----- 已验证 ------------|
```

---


## Use Cases

### 1. Single Sign-On (SSO) Across Services
Users log in once and access multiple services without re-authentication:

```text
User → Service A: Login
  ├─ Create session: session_id = "abc123"
  └─ Save to distributed storage

User → Service B: Request with session_id = "abc123"
  ├─ Service B retrieves session from storage
  ├─ Validates user is authenticated
  └─ Processes request ✅ (No re-login needed!)
```

### 2. Session Sharing for User Context
Services share user context and state:

```text
Service A stores: { "user_role": "admin", "department": "IT" }
Service B reads: Same session attributes available
Service C updates: { "last_order": "order_123" }
→ All services share the same session state!
```

### 3. Multi-Device Session Management
One user can have multiple active sessions:

```text
User: user_123
  ├─ Session 1: Web (Service A)
  ├─ Session 2: Mobile (Service B)
  └─ Session 3: Desktop (Service C)

All sessions can be:
  - Listed: get_sessions_by_login_id()
  - Managed individually
  - Terminated all at once: delete_all_sessions()
```

### 4. Microservices Architecture
Share user sessions across API Gateway, User Service, Order Service, etc.

### 5. Multi-Region Deployment
Synchronize sessions across different geographic regions using shared storage.

### 6. Load Balancing
Maintain session consistency across multiple server instances.

## Storage Backends

### Redis Implementation (Recommended)

```rust
use redis::AsyncCommands;

pub struct RedisDistributedStorage {
    client: redis::Client,
}

#[async_trait]
impl DistributedSessionStorage for RedisDistributedStorage {
    async fn save_session(&self, session: DistributedSession, ttl: Option<Duration>) 
        -> Result<(), SaTokenError> 
    {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("distributed:session:{}", session.session_id);
        let value = serde_json::to_string(&session)?;
        
        if let Some(ttl) = ttl {
            conn.set_ex(&key, value, ttl.as_secs() as usize).await?;
        } else {
            conn.set(&key, value).await?;
        }
        
        // Index by login_id for quick lookup
        let index_key = format!("distributed:login:{}", session.login_id);
        conn.sadd(index_key, &session.session_id).await?;
        
        Ok(())
    }
    
    // ... implement other methods
}
```

### Database Implementation

```rust
use sqlx::PgPool;

pub struct PostgresDistributedStorage {
    pool: PgPool,
}

#[async_trait]
impl DistributedSessionStorage for PostgresDistributedStorage {
    async fn save_session(&self, session: DistributedSession, ttl: Option<Duration>) 
        -> Result<(), SaTokenError> 
    {
        let expires_at = ttl.map(|t| Utc::now() + chrono::Duration::from_std(t).unwrap());
        
        sqlx::query!(
            "INSERT INTO distributed_sessions 
             (session_id, login_id, token, service_id, attributes, expires_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (session_id) DO UPDATE 
             SET attributes = $5, last_access = NOW()",
            session.session_id,
            session.login_id,
            session.token,
            session.service_id,
            serde_json::to_value(&session.attributes)?,
            expires_at,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    // ... implement other methods
}
```

## Best Practices

### 1. Service Registration
Use crypto-secure secret generation for service credentials:

```rust
let credential = ServiceCredential {
    service_id: "user-service".to_string(),
    service_name: "User Management Service".to_string(),
    secret_key: generate_secure_secret(), // Use crypto-secure generation
    created_at: Utc::now(),
    permissions: vec!["user.read".to_string(), "user.write".to_string()],
};
manager.register_service(credential).await;
```

### 2. Session Creation with Context
Add relevant attributes immediately after session creation:

```rust
let session = manager.create_session(login_id, token).await?;

// Add relevant attributes immediately
manager.set_attribute(&session.session_id, "user_role".to_string(), "admin".to_string()).await?;
manager.set_attribute(&session.session_id, "department".to_string(), "IT".to_string()).await?;
manager.set_attribute(&session.session_id, "login_device".to_string(), "web".to_string()).await?;
```

### 3. Cross-Service Access Pattern
Always verify service identity and check permissions:

```rust
// 1. Verify service identity
let service_cred = manager.verify_service("service-b", request.secret).await?;

// 2. Check permissions
if !service_cred.permissions.contains(&"session.read".to_string()) {
    return Err(SaTokenError::PermissionDenied);
}

// 3. Access session
let session = manager.get_session(&request.session_id).await?;

// 4. Refresh to keep session alive
manager.refresh_session(&session.session_id).await?;
```

### 4. Multi-Device Logout
Support both individual and bulk logout:

```rust
// Logout from all devices
manager.delete_all_sessions(&login_id).await?;

// Or logout specific session
manager.delete_session(&session_id).await?;
```

### 5. Session Monitoring
Monitor user's active sessions for security:

```rust
let sessions = manager.get_sessions_by_login_id(&login_id).await?;

for session in sessions {
    println!("Session: {} from service: {}, last active: {}", 
        session.session_id,
        session.service_id,
        session.last_access
    );
    
    // Check for suspicious activity
    if is_suspicious(&session) {
        manager.delete_session(&session.session_id).await?;
    }
}
```

### 6. Security Considerations

- ✅ **Service Authentication**: Each service has unique secret_key
- ✅ **Permission-Based Access**: Services have explicit permissions
- ✅ **Session Timeout**: Configure appropriate TTL
- ✅ **Data Encryption**: Encrypt sensitive session attributes
- ✅ **Audit Logging**: Log session creation/deletion and cross-service access

### 7. Production Recommendations

1. **Use appropriate TTL** - Set session timeout based on security requirements (typically 1-24 hours)
2. **Use persistent storage** - Implement Redis/Database storage for production (not in-memory)
3. **Secure service credentials** - Use strong secret keys and rotate periodically
4. **Monitor session count** - Track active sessions per user to detect anomalies
5. **Implement cleanup** - Use storage TTL features for automatic cleanup
6. **Enable encryption** - Encrypt sensitive session attributes at rest

## Related Documentation

- [WebSocket Authentication](/zh/guide/websocket-auth.md)
- [Online User Management](/zh/guide/online-user-management.md)
- [Event Listener Guide](/zh/guide/event-listener.md)

## License

MIT OR Apache-2.0

