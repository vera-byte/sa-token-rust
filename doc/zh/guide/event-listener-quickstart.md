# 事件监听快速开始

[English](/guide/event-listener-quickstart.md) | 中文

---

## 概述

sa-token-rust 提供了强大的事件监听功能，可以监听登录、登出、踢出下线等操作。

## 快速开始

### 1. 创建自定义监听器

```rust
use async_trait::async_trait;
use sa_token_core::SaTokenListener;

struct MyListener;

#[async_trait]
impl SaTokenListener for MyListener {
    async fn on_login(&self, login_id: &str, token: &str, login_type: &str) {
        println!("用户 {} 登录了", login_id);
        // 在这里添加您的业务逻辑
    }

    async fn on_logout(&self, login_id: &str, token: &str, login_type: &str) {
        println!("用户 {} 登出了", login_id);
    }

    async fn on_kick_out(&self, login_id: &str, token: &str, login_type: &str) {
        println!("用户 {} 被踢出下线", login_id);
    }
}
```

### 2. 注册监听器

#### ⭐ 推荐方式：使用 Builder 模式注册

```rust
use sa_token_core::SaTokenConfig;
use sa_token_storage_memory::MemoryStorage;
use std::sync::Arc;

// 一行代码完成初始化：创建 manager + 注册监听器 + 初始化 StpUtil！
SaTokenConfig::builder()
    .storage(Arc::new(MemoryStorage::new()))
    .timeout(7200)
    .register_listener(Arc::new(MyListener))  // 在这里注册监听器！
    .register_listener(Arc::new(AnotherListener))  // 支持注册多个！
    .build();  // 自动完成所有初始化！

// StpUtil 立即可用！
```

#### 传统方式：手动注册

```rust
use sa_token_core::{SaTokenManager, StpUtil};

// 方式一：通过 Manager 注册
let manager = SaTokenManager::new(storage, config);
manager.event_bus().register(Arc::new(MyListener));  // 不需要 .await！

// 方式二：通过 StpUtil 注册（在 manager 创建后）
StpUtil::register_listener(Arc::new(MyListener));  // 不需要 .await！
```

### 3. 使用内置的日志监听器

```rust
use sa_token_core::LoggingListener;

// 通过 Builder
SaTokenConfig::builder()
    .storage(Arc::new(MemoryStorage::new()))
    .register_listener(Arc::new(LoggingListener))
    .build();

// 或通过 Manager
manager.event_bus().register(Arc::new(LoggingListener));
```

### 4. 自动触发事件

一旦注册了监听器，相关操作会自动触发事件：

```rust
// 登录 - 触发 Login 事件
let token = StpUtil::login("user_123").await?;

// 登出 - 触发 Logout 事件
StpUtil::logout(&token).await?;

// 踢出下线 - 触发 KickOut 事件
StpUtil::kick_out("user_123").await?;
```

## 支持的事件类型

- **Login**: 登录事件（由 `login()` 触发，包括 WebSocket 登录）
- **Logout**: 登出事件（由 `logout()` 触发）
- **KickOut**: 踢出下线事件（由 `kick_out()` 触发）
- **RenewTimeout**: Token 续期事件（由 `renew_timeout()` 触发）
- **Replaced**: 被顶下线事件（当用户在其他设备登录导致当前设备被替换时触发）
- **Banned**: 被封禁事件（当用户被封禁时触发）

## 核心特性

✅ **零配置** - `build()` 自动完成所有初始化  
✅ **同步注册** - 不需要 `.await`  
✅ **Builder 模式** - 简洁流畅的 API  
✅ **多监听器** - 想注册多少就注册多少  
✅ **类型安全** - 编译时检查  

## 运行示例

```bash
cargo run --example event_listener_example
```

## 更多信息

查看完整文档：[EVENT_LISTENER_zh-CN.md](/zh/guide/event-listener.md)
