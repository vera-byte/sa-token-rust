# Event Listener Quick Start

English | [中文](/zh/guide/event-listener-quickstart.md)

---

## Overview

sa-token-rust provides powerful event listening capabilities to monitor login, logout, kick-out, and other operations.

## Quick Start

### 1. Create Custom Listener

```rust
use async_trait::async_trait;
use sa_token_core::SaTokenListener;

struct MyListener;

#[async_trait]
impl SaTokenListener for MyListener {
    async fn on_login(&self, login_id: &str, token: &str, login_type: &str) {
        println!("User {} logged in", login_id);
        // Add your business logic here
    }

    async fn on_logout(&self, login_id: &str, token: &str, login_type: &str) {
        println!("User {} logged out", login_id);
    }

    async fn on_kick_out(&self, login_id: &str, token: &str, login_type: &str) {
        println!("User {} was kicked out", login_id);
    }
}
```

### 2. Register Listener

#### ⭐ Recommended: Register via Builder Pattern

```rust
use sa_token_core::SaTokenConfig;
use sa_token_storage_memory::MemoryStorage;
use std::sync::Arc;

// One-line initialization: create manager + register listeners + initialize StpUtil!
SaTokenConfig::builder()
    .storage(Arc::new(MemoryStorage::new()))
    .timeout(7200)
    .register_listener(Arc::new(MyListener))  // Register listener here!
    .register_listener(Arc::new(AnotherListener))  // Support multiple listeners!
    .build();  // Auto-complete all initialization!

// StpUtil is ready to use immediately!
```

#### Alternative: Manual Registration

```rust
use sa_token_core::{SaTokenManager, StpUtil};

// Method 1: Register via Manager
let manager = SaTokenManager::new(storage, config);
manager.event_bus().register(Arc::new(MyListener));  // No .await needed!

// Method 2: Register via StpUtil (after manager is created)
StpUtil::register_listener(Arc::new(MyListener));  // No .await needed!
```

### 3. Use Built-in Logging Listener

```rust
use sa_token_core::LoggingListener;

// Via Builder
SaTokenConfig::builder()
    .storage(Arc::new(MemoryStorage::new()))
    .register_listener(Arc::new(LoggingListener))
    .build();

// Or via Manager
manager.event_bus().register(Arc::new(LoggingListener));
```

### 4. Automatic Event Triggering

Once listeners are registered, events are automatically triggered:

```rust
// Login - triggers Login event
let token = StpUtil::login("user_123").await?;

// Logout - triggers Logout event
StpUtil::logout(&token).await?;

// Kick out - triggers KickOut event
StpUtil::kick_out("user_123").await?;
```

## Supported Event Types

- **Login**: Login event (triggered by `login()`, including WebSocket login)
- **Logout**: Logout event (triggered by `logout()`)
- **KickOut**: Kick-out event (triggered by `kick_out()`)
- **RenewTimeout**: Token renewal event (triggered by `renew_timeout()`)
- **Replaced**: Replaced event (triggered when a user is replaced by another device)
- **Banned**: Banned event (triggered when a user is banned)

## Key Features

✅ **Zero Configuration** - `build()` auto-initializes everything  
✅ **Synchronous Registration** - No `.await` needed  
✅ **Builder Pattern** - Clean and fluent API  
✅ **Multiple Listeners** - Register as many as you need  
✅ **Type Safe** - Compile-time checks  

## Run Example

```bash
cargo run --example event_listener_example
```

## More Information

See full documentation: [EVENT_LISTENER.md](/guide/event-listener.md)
