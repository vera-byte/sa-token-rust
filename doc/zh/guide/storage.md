# 存储后端

中文文档 | [English](/guide/storage)

sa-token-rust 通过 `SaStorage` trait 支持 4 种存储后端。本页介绍每种后端、其配置以及如何实现自定义后端。

## SaStorage Trait

所有存储后端实现 `sa-token-adapter` 中的 `SaStorage` trait：

```rust
use sa_token_adapter::storage::{SaStorage, StorageResult, StorageError};

#[async_trait]
pub trait SaStorage: Send + Sync {
    // 必须实现的方法
    async fn get(&self, key: &str) -> StorageResult<Option<String>>;
    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> StorageResult<()>;
    async fn delete(&self, key: &str) -> StorageResult<()>;
    async fn exists(&self, key: &str) -> StorageResult<bool>;
    async fn expire(&self, key: &str, ttl: Duration) -> StorageResult<()>;
    async fn ttl(&self, key: &str) -> StorageResult<Option<Duration>>;
    async fn clear(&self) -> StorageResult<()>;

    // 带默认实现的方法
    async fn mget(&self, keys: &[&str]) -> StorageResult<Vec<Option<String>>>;
    async fn mset(&self, items: &[(&str, &str)], ttl: Option<Duration>) -> StorageResult<()>;
    async fn mdel(&self, keys: &[&str]) -> StorageResult<()>;
    async fn incr(&self, key: &str) -> StorageResult<i64>;
    async fn decr(&self, key: &str) -> StorageResult<i64>;
    async fn keys(&self, pattern: &str) -> StorageResult<Vec<String>>;
}
```

### 方法说明

| 方法 | 说明 |
|---|---|
| `get` | 获取值。键不存在或已过期返回 `None`。 |
| `set` | 存储值，可选 TTL。`None` 表示永不过期。 |
| `delete` | 删除键。 |
| `exists` | 检查键是否存在且未过期。 |
| `expire` | 设置或延长已有键的 TTL。 |
| `ttl` | 获取剩余有效期。`None` 表示无过期时间。 |
| `clear` | 删除所有键，谨慎使用。 |
| `mget` / `mset` / `mdel` | 批量操作。默认循环单键操作。可覆盖以实现原子性。 |
| `incr` / `decr` | 原子增减。默认：读取→解析→写入。 |
| `keys` | 获取匹配模式的所有键（支持 `*` 通配）。默认：空列表。 |

### StorageError

```rust
pub enum StorageError {
    OperationFailed(String),
    KeyNotFound(String),
    SerializationError(String),
    ConnectionError(String),
    InternalError(String),
}
```

---

## MemoryStorage

基于 `Arc<RwLock<HashMap>>` 的内存存储。适合**开发和测试**。

```toml
[dependencies]
sa-token-storage-memory = "0.1.14"
```

```rust
use sa_token_storage_memory::MemoryStorage;
use std::sync::Arc;

let storage = Arc::new(MemoryStorage::new());

// 清理过期数据
storage.cleanup_expired().await;
```

### `keys()` 实现

内存存储将模式中的 `*` 替换为 `.*`，内部使用正则匹配。例如 `sa:token:*` 匹配所有以 `sa:token:` 开头的键。

### 特点

- ✅ 最快（无网络开销）
- ✅ 无外部依赖
- ✅ TTL 支持，`get` 时自动清理过期数据
- ❌ 重启数据丢失
- ❌ 不可跨进程/Pod 共享
- ❌ `keys()` 使用正则，非 Redis 风格 glob

---

## RedisStorage

生产级 Redis 存储，支持构建器模式配置。需要 `redis` crate。

```toml
[dependencies]
sa-token-storage-redis = "0.1.14"
```

### 快速开始

```rust
use sa_token_storage_redis::RedisStorage;
use std::sync::Arc;

// 方式 1：URL 字符串
let storage = RedisStorage::new(
    "redis://:password@localhost:6379/0",
    "sa-token:",  // 键前缀
).await?;

// 方式 2：配置结构体
let config = RedisConfig {
    host: "localhost".into(),
    port: 6379,
    password: Some("password".into()),
    database: 0,
    ..Default::default()
};
let storage = RedisStorage::from_config(config, "sa-token:").await?;
```

### 构建器模式

```rust
use sa_token_storage_redis::RedisStorage;

let storage = RedisStorage::builder()
    .host("redis-cluster.example.com")
    .port(6380)
    .password("secure-password")
    .database(1)
    .key_prefix("sa-token:")
    .build()
    .await?;
```

### RedisConfig 字段

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `host` | `String` | `"localhost"` | Redis 主机地址 |
| `port` | `u16` | `6379` | Redis 端口 |
| `password` | `Option<String>` | `None` | 认证密码 |
| `database` | `u8` | `0` | 数据库编号 (0-15) |
| `pool_size` | `u32` | `10` | 连接池大小（预留） |

### URL 格式

```
redis://:password@host:port/database
redis://localhost:6379/0                          # 无密码
redis://:mypass@localhost:6379/0                  # 有密码
redis://:Aq23-hjPwFB3mBDNFp3W1@localhost:6379/0   # 复杂密码
```

### 注意事项

- 所有 `SaStorage` trait 方法均使用原生 Redis 命令实现
- `mset` 使用 Redis pipeline 实现原子批量操作
- `clear()` 使用 `KEYS` 命令（大数据集建议使用 `SCAN`）
- `keys()` 返回匹配模式且带前缀的键列表

---

## DatabaseStorage

占位桩 — **尚未实现**。所有 trait 方法返回 `StorageError::InternalError("Not implemented")`。

```toml
[dependencies]
sa-token-storage-database = "0.1.14"
```

```rust
use sa_token_storage_database::DatabaseStorage;

// 当前始终返回错误
let storage = DatabaseStorage::new("postgres://localhost/db").await?;
// → Err(StorageError::InternalError("Not implemented"))
```

---

## 自定义存储

实现 `SaStorage` trait 来适配自己的后端：

```rust
use sa_token_adapter::storage::{SaStorage, StorageResult};
use std::time::Duration;

struct CustomStorage {
    // 你的状态（连接池等）
}

#[async_trait]
impl SaStorage for CustomStorage {
    async fn get(&self, key: &str) -> StorageResult<Option<String>> {
        Ok(None)
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> StorageResult<()> {
        Ok(())
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        Ok(())
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        Ok(false)
    }

    async fn expire(&self, key: &str, ttl: Duration) -> StorageResult<()> {
        Ok(())
    }

    async fn ttl(&self, key: &str) -> StorageResult<Option<Duration>> {
        Ok(None)
    }

    async fn clear(&self) -> StorageResult<()> {
        Ok(())
    }
}
```

将自定义存储传入状态构建器：

```rust
let storage = Arc::new(CustomStorage::new());
let state = SaTokenState::builder()
    .storage(storage)
    .build();
```

## 选择后端

| 后端 | 适用场景 |
|---|---|
| **MemoryStorage** | 开发、测试、单实例部署 |
| **RedisStorage** | 生产环境、分布式服务、水平扩展 |
| **DatabaseStorage** | 尚未可用（占位） |
| **自定义** | 特殊需求（Memcached、DynamoDB 等） |

## 相关文档

- [快速入门](/zh/guide/quick-start)
- [框架集成](/zh/guide/framework-integration)
