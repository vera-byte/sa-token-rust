# Framework Adapters

[中文文档](/zh/guide/adapter) | English

sa-token-rust communicates with web frameworks through adapter traits defined in `sa-token-adapter`. This page documents these low-level interfaces — useful for implementing custom framework plugins or debugging token extraction.

## SaRequest

The `SaRequest` trait abstracts HTTP request data. Each framework plugin provides an implementation for its request type.

```rust
pub trait SaRequest {
    // Required
    fn get_header(&self, name: &str) -> Option<String>;
    fn get_cookie(&self, name: &str) -> Option<String>;
    fn get_param(&self, name: &str) -> Option<String>;
    fn get_path(&self) -> String;
    fn get_method(&self) -> String;

    // With defaults
    fn get_headers(&self) -> HashMap<String, String> { HashMap::new() }
    fn get_cookies(&self) -> HashMap<String, String> { HashMap::new() }
    fn get_params(&self) -> HashMap<String, String> { HashMap::new() }
    fn get_uri(&self) -> String { self.get_path() }
    fn get_body_json<T: DeserializeOwned>(&self) -> Option<T> { None }
    fn get_client_ip(&self) -> Option<String> { None }
    fn get_user_agent(&self) -> Option<String> { self.get_header("user-agent") }
}
```

### Method Table

| Method | Required | Description |
|---|---|---|
| `get_header` | Yes | Read a single HTTP header |
| `get_cookie` | Yes | Read a single cookie by name |
| `get_param` | Yes | Read a single query parameter |
| `get_path` | Yes | Get the request URL path |
| `get_method` | Yes | Get the HTTP method |
| `get_headers` | No | Read all headers as a map |
| `get_cookies` | No | Read all cookies as a map |
| `get_params` | No | Read all query parameters as a map |
| `get_uri` | No | Get full request URI (default: path) |
| `get_body_json` | No | Parse JSON body if available |
| `get_client_ip` | No | Get client IP address |
| `get_user_agent` | No | Get User-Agent header |

---

## SaResponse

Abstraction for setting response data.

```rust
pub trait SaResponse {
    // Required
    fn set_header(&mut self, name: &str, value: &str);
    fn set_cookie(&mut self, name: &str, value: &str, options: CookieOptions);
    fn set_status(&mut self, status: u16);
    fn set_json_body<T: Serialize>(&mut self, body: T) -> Result<(), serde_json::Error>;

    // With default
    fn delete_cookie(&mut self, name: &str) {
        self.set_cookie(name, "", CookieOptions { max_age: Some(0), ..Default::default() });
    }
}
```

---

## CookieOptions

Configure cookie attributes for `set_cookie`.

```rust
pub struct CookieOptions {
    pub domain: Option<String>,    // Cookie domain
    pub path: Option<String>,      // Cookie path
    pub max_age: Option<i64>,      // Max age in seconds
    pub http_only: bool,           // HttpOnly flag
    pub secure: bool,              // Secure flag (HTTPS only)
    pub same_site: Option<SameSite>, // SameSite attribute
}

pub enum SameSite {
    Strict,
    Lax,
    None,
}
```

**Example:**

```rust
let opts = CookieOptions {
    path: Some("/".into()),
    max_age: Some(86400),
    http_only: true,
    secure: true,
    same_site: Some(SameSite::Lax),
    ..Default::default()
};
```

---

## Utility Functions

The `sa_token_adapter::utils` module provides helpers used across all plugins.

### `parse_cookies`

```rust
pub fn parse_cookies(cookie_header: &str) -> HashMap<String, String>
```

Parses a raw `Cookie` header string (`"key1=value1; key2=value2"`) into a key-value map. Does **not** URL-decode values.

### `parse_query_string`

```rust
pub fn parse_query_string(query: &str) -> HashMap<String, String>
```

Parses a query string (`"key1=value1&key2=value2"`) into a key-value map. **Does** URL-decode both keys and values.

### `build_cookie_string`

```rust
pub fn build_cookie_string(name: &str, value: &str, options: CookieOptions) -> String
```

Builds a complete `Set-Cookie` header value from name, value, and options. Automatically includes `Domain`, `Path`, `Max-Age`, `HttpOnly`, `Secure`, and `SameSite` attributes as appropriate.

### `strip_bearer_prefix`

```rust
pub fn strip_bearer_prefix(auth_header: &str) -> Option<String>
```

Strict: strips `Bearer ` prefix only. Returns `None` if not present. Use when you need to distinguish between Bearer tokens and other authorization schemes.

### `extract_bearer_or_value`

```rust
pub fn extract_bearer_or_value(s: &str) -> String
```

Lenient: strips `Bearer ` if present, otherwise returns trimmed input. This is what framework plugins use for token extraction from Authorization headers.

### `strip_bearer_or_passthrough`

```rust
pub fn strip_bearer_or_passthrough(s: &str) -> String
```

Alias of `extract_bearer_or_value`. Prefer `extract_bearer_or_value` in new code.

---

## FrameworkAdapter

Minimal lifecycle trait for framework plugins.

```rust
pub trait FrameworkAdapter: Send + Sync {
    fn name(&self) -> &str;
    async fn initialize(&self) -> Result<(), String>;
    async fn shutdown(&self) -> Result<(), String> { Ok(()) }
}
```

Not required for basic usage. Used when a framework plugin needs initialization logic (e.g., Redis connection pool setup).

---

## Writing a Custom Plugin

```rust
use sa_token_adapter::context::{SaRequest, SaResponse, CookieOptions};
use sa_token_adapter::utils::{extract_bearer_or_value, parse_cookies};

struct MyRequestAdapter {
    headers: HashMap<String, String>,
    cookies: HashMap<String, String>,
    path: String,
    method: String,
}

impl SaRequest for MyRequestAdapter {
    fn get_header(&self, name: &str) -> Option<String> {
        self.headers.get(name).cloned()
    }

    fn get_cookie(&self, name: &str) -> Option<String> {
        // Parse the Cookie header
        self.get_header("cookie")
            .map(|h| parse_cookies(&h))
            .and_then(|c| c.get(name).cloned())
    }

    fn get_param(&self, name: &str) -> Option<String> {
        // Parse from self.path query string if present
        None
    }

    fn get_path(&self) -> String { self.path.clone() }
    fn get_method(&self) -> String { self.method.clone() }
}
```

## Related

- [Storage Backends](/guide/storage)
- [Framework Integration](/guide/framework-integration)
- [StpUtil API](/guide/stp-util)
