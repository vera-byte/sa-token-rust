# WebSocket 认证指南

## 中文

### 概述

WebSocket 认证模块为 sa-token-rust 中的 WebSocket 连接提供安全认证。它支持多种 Token 提取方法，并与核心认证系统无缝集成。

### 功能特性

- **多种 Token 来源**
  - Authorization 请求头（Bearer Token）
  - WebSocket Protocol 请求头
  - 查询参数
- **Token 验证** - 自动过期检查
- **会话管理** - 每个连接的唯一会话 ID
- **可扩展** - 自定义 Token 提取器

### 快速开始

#### 1. 基本用法

```rust
use sa_token_core::{SaTokenManager, SaTokenConfig, WsAuthManager};
use sa_token_storage_memory::MemoryStorage;
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化管理器
    let storage = Arc::new(MemoryStorage::new());
    let config = SaTokenConfig::default();
    let manager = Arc::new(SaTokenManager::new(storage, config));
    
    // 创建 WebSocket 认证管理器
    let ws_auth = WsAuthManager::new(manager.clone());
    
    // 用户登录
    let token = manager.login("user123").await?;
    
    // 认证 WebSocket 连接
    let mut headers = HashMap::new();
    headers.insert(
        "Authorization".to_string(),
        format!("Bearer {}", token.as_str())
    );
    
    let auth_info = ws_auth.authenticate(&headers, &HashMap::new()).await?;
    
    println!("用户 {} 已连接", auth_info.login_id);
    println!("会话 ID: {}", auth_info.session_id);
    
    Ok(())
}
```

#### 2. 从查询参数提取 Token

```rust
// 从 URL 查询参数提取 Token
let mut query = HashMap::new();
query.insert("token".to_string(), token.as_str().to_string());

let auth_info = ws_auth.authenticate(&HashMap::new(), &query).await?;
```

#### 3. 自定义 Token 提取器

```rust
use sa_token_core::WsTokenExtractor;
use async_trait::async_trait;

struct CustomExtractor;

#[async_trait]
impl WsTokenExtractor for CustomExtractor {
    async fn extract_token(
        &self,
        headers: &HashMap<String, String>,
        query: &HashMap<String, String>
    ) -> Option<String> {
        // 自定义提取逻辑
        headers.get("X-Custom-Token").cloned()
    }
}

// 使用自定义提取器
let custom_extractor = Arc::new(CustomExtractor);
let ws_auth = WsAuthManager::with_extractor(manager, custom_extractor);
```

### API 参考

#### WsAuthManager

**方法:**
- `new(manager)` - 使用默认提取器创建
- `with_extractor(manager, extractor)` - 使用自定义提取器创建
- `authenticate(headers, query)` - 认证连接
- `verify_token(token)` - 验证 Token 有效性
- `refresh_ws_session(auth_info)` - 刷新会话

#### WsAuthInfo

**字段:**
- `login_id` - 用户标识符
- `token` - 认证 Token
- `session_id` - 唯一会话 ID
- `connect_time` - 连接时间戳
- `metadata` - 自定义元数据

### 最佳实践

1. **始终在重新连接时验证 Token**
2. **在生产环境中使用 HTTPS/WSS**
3. **为长连接实现 Token 刷新**
4. **优雅地处理 Token 过期**
5. **记录认证事件以进行安全审计**

---


## Related Documentation

- [Online User Management](/zh/guide/online-user-management.md)
- [Distributed Session](/zh/guide/distributed-session.md)
- [Event Listener Guide](/zh/guide/event-listener.md)
- [JWT Guide](/zh/guide/jwt.md)

## License

MIT OR Apache-2.0

