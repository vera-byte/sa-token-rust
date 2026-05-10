# Storage Backends

[中文文档](/zh/guide/storage) | English

sa-token-rust supports 4 storage backends via the `SaStorage` trait. This page documents each backend, its configuration, and how to implement a custom backend.

## SaStorage Trait

All storage backends implement the `SaStorage` trait from `sa-token-adapter`:

```rust
use sa_token_adapter::storage::{SaStorage, StorageResult, StorageError};

#[async_trait]
pub trait SaStorage: Send + Sync {
    // Required methods
    async fn get(&self, key: &str) -> StorageResult<Option<String>>;
    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> StorageResult<()>;
    async fn delete(&self, key: &str) -> StorageResult<()>;
    async fn exists(&self, key: &str) -> StorageResult<bool>;
    async fn expire(&self, key: &str, ttl: Duration) -> StorageResult<()>;
    async fn ttl(&self, key: &str) -> StorageResult<Option<Duration>>;
    async fn clear(&self) -> StorageResult<()>;

    // Methods with default implementations
    async fn mget(&self, keys: &[&str]) -> StorageResult<Vec<Option<String>>>;
    async fn mset(&self, items: &[(&str, &str)], ttl: Option<Duration>) -> StorageResult<()>;
    async fn mdel(&self, keys: &[&str]) -> StorageResult<()>;
    async fn incr(&self, key: &str) -> StorageResult<i64>;
    async fn decr(&self, key: &str) -> StorageResult<i64>;
    async fn keys(&self, pattern: &str) -> StorageResult<Vec<String>>;
}
```

### Method Descriptions

| Method | Description |
|---|---|
| `get` | Retrieve a value. Returns `None` if key doesn't exist or is expired. |
| `set` | Store a value with optional TTL. `None` for TTL means no expiration. |
| `delete` | Remove a key. |
| `exists` | Check if a key exists and is not expired. |
| `expire` | Set or extend TTL on an existing key. |
| `ttl` | Get remaining time-to-live. `None` means no expiration. |
| `clear` | Delete all keys. Use with caution. |
| `mget` / `mset` / `mdel` | Bulk operations. Default: loop over single-key ops. Override for atomicity. |
| `incr` / `decr` | Atomic increment/decrement. Default: read→parse→write. |
| `keys` | Get all keys matching a pattern (supports `*` wildcard). Default: empty vec. |

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

In-memory storage with `Arc<RwLock<HashMap>>`. Best for **development and testing**.

```toml
[dependencies]
sa-token-storage-memory = "0.1.14"
```

```rust
use sa_token_storage_memory::MemoryStorage;
use std::sync::Arc;

let storage = Arc::new(MemoryStorage::new());

// Clean up expired entries
storage.cleanup_expired().await;
```

### `keys()` Implementation

Memory storage converts `*` in patterns to `.*` and uses regex matching internally. For example, `sa:token:*` matches all keys starting with `sa:token:`.

### Characteristics

- ✅ Fastest (no network overhead)
- ✅ No external dependency
- ✅ TTL support with auto-cleanup in `get`
- ❌ Data lost on restart
- ❌ Not shared across processes/Pods
- ❌ `keys()` uses regex, not Redis-style glob

---

## RedisStorage

Production-grade Redis storage with builder-pattern configuration. Requires the `redis` crate.

```toml
[dependencies]
sa-token-storage-redis = "0.1.14"
```

### Quick Start

```rust
use sa_token_storage_redis::RedisStorage;
use std::sync::Arc;

// Method 1: URL string
let storage = RedisStorage::new(
    "redis://:password@localhost:6379/0",
    "sa-token:",  // key prefix
).await?;

// Method 2: Config struct
let config = RedisConfig {
    host: "localhost".into(),
    port: 6379,
    password: Some("password".into()),
    database: 0,
    ..Default::default()
};
let storage = RedisStorage::from_config(config, "sa-token:").await?;
```

### Builder Pattern

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

### RedisConfig Fields

| Field | Type | Default | Description |
|---|---|---|---|
| `host` | `String` | `"localhost"` | Redis server host |
| `port` | `u16` | `6379` | Redis server port |
| `password` | `Option<String>` | `None` | Authentication password |
| `database` | `u8` | `0` | Database number (0-15) |
| `pool_size` | `u32` | `10` | Reserved for future pooling support |

### URL Format

```
redis://:password@host:port/database
redis://localhost:6379/0                          # no password
redis://:mypass@localhost:6379/0                  # with password
redis://:Aq23-hjPwFB3mBDNFp3W1@localhost:6379/0   # complex password
```

### Notes

- All `SaStorage` trait methods are implemented with native Redis commands (`GET`, `SET`, `DEL`, `EXISTS`, `EXPIRE`, `TTL`, `INCR`, `DECR`)
- `mset` uses a Redis pipeline for atomic batch operations
- `clear()` uses `KEYS` command (consider `SCAN` for production with large datasets)
- `keys()` returns keys matching the pattern with the configured prefix prepended

---

## DatabaseStorage

Placeholder stub — **not yet implemented**. All trait methods return `StorageError::InternalError("Not implemented")`.

```toml
[dependencies]
sa-token-storage-database = "0.1.14"
```

```rust
use sa_token_storage_database::DatabaseStorage;

// Currently always returns an error
let storage = DatabaseStorage::new("postgres://localhost/db").await?;
// → Err(StorageError::InternalError("Not implemented"))
```

---

## Custom Storage

Implement `SaStorage` for your own backend:

```rust
use sa_token_adapter::storage::{SaStorage, StorageResult};
use std::time::Duration;

struct CustomStorage {
    // your state (connection pool, etc.)
}

#[async_trait]
impl SaStorage for CustomStorage {
    async fn get(&self, key: &str) -> StorageResult<Option<String>> {
        // Your implementation
        Ok(None)
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> StorageResult<()> {
        // Your implementation
        Ok(())
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        // Your implementation
        Ok(())
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        // Your implementation
        Ok(false)
    }

    async fn expire(&self, key: &str, ttl: Duration) -> StorageResult<()> {
        // Your implementation
        Ok(())
    }

    async fn ttl(&self, key: &str) -> StorageResult<Option<Duration>> {
        // Your implementation
        Ok(None)
    }

    async fn clear(&self) -> StorageResult<()> {
        // Your implementation
        Ok(())
    }
}
```

Pass your custom storage to the state builder:

```rust
let storage = Arc::new(CustomStorage::new());
let state = SaTokenState::builder()
    .storage(storage)
    .build();
```

## Choosing a Backend

| Backend | Use Case |
|---|---|
| **MemoryStorage** | Development, testing, single-instance deployments |
| **RedisStorage** | Production, distributed services, horizontal scaling |
| **DatabaseStorage** | Not yet available (placeholder) |
| **Custom** | Specialized needs (memcached, DynamoDB, etc.) |

## Related

- [Quick Start](/guide/quick-start)
- [Framework Integration](/guide/framework-integration)
