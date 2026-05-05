# WebSocket Authentication Guide

## English

### Overview

The WebSocket Authentication module provides secure authentication for WebSocket connections in sa-token-rust. It supports multiple token extraction methods and integrates seamlessly with the core authentication system.

### Features

- **Multiple Token Sources**
  - Authorization Header (Bearer Token)
  - WebSocket Protocol Header
  - Query Parameters
- **Token Validation** - Automatic expiration checking
- **Session Management** - Unique session IDs for each connection
- **Extensible** - Custom token extractors

### Quick Start

#### 1. Basic Usage

```rust
use sa_token_core::{SaTokenManager, SaTokenConfig, WsAuthManager};
use sa_token_storage_memory::MemoryStorage;
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize manager
    let storage = Arc::new(MemoryStorage::new());
    let config = SaTokenConfig::default();
    let manager = Arc::new(SaTokenManager::new(storage, config));
    
    // Create WebSocket auth manager
    let ws_auth = WsAuthManager::new(manager.clone());
    
    // User logs in
    let token = manager.login("user123").await?;
    
    // Authenticate WebSocket connection
    let mut headers = HashMap::new();
    headers.insert(
        "Authorization".to_string(),
        format!("Bearer {}", token.as_str())
    );
    
    let auth_info = ws_auth.authenticate(&headers, &HashMap::new()).await?;
    
    println!("User {} connected", auth_info.login_id);
    println!("Session ID: {}", auth_info.session_id);
    
    Ok(())
}
```

#### 2. Token from Query Parameter

```rust
// Extract token from URL query parameter
let mut query = HashMap::new();
query.insert("token".to_string(), token.as_str().to_string());

let auth_info = ws_auth.authenticate(&HashMap::new(), &query).await?;
```

#### 3. Custom Token Extractor

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
        // Custom extraction logic
        headers.get("X-Custom-Token").cloned()
    }
}

// Use custom extractor
let custom_extractor = Arc::new(CustomExtractor);
let ws_auth = WsAuthManager::with_extractor(manager, custom_extractor);
```

### API Reference

#### WsAuthManager

**Methods:**
- `new(manager)` - Create with default extractor
- `with_extractor(manager, extractor)` - Create with custom extractor
- `authenticate(headers, query)` - Authenticate connection
- `verify_token(token)` - Verify token validity
- `refresh_ws_session(auth_info)` - Refresh session

#### WsAuthInfo

**Fields:**
- `login_id` - User identifier
- `token` - Authentication token
- `session_id` - Unique session ID
- `connect_time` - Connection timestamp
- `metadata` - Custom metadata

### Best Practices

1. **Always verify tokens on reconnection**
2. **Use HTTPS/WSS in production**
3. **Implement token refresh for long-lived connections**
4. **Handle token expiration gracefully**
5. **Log authentication events for security auditing**

---

## Related Documentation

- [Online User Management](/guide/online-user-management.md)
- [Distributed Session](/guide/distributed-session.md)
- [Event Listener Guide](/guide/event-listener.md)
- [JWT Guide](/guide/jwt.md)

## License

MIT OR Apache-2.0

