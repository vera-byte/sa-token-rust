# 框架适配器

中文文档 | [English](/guide/adapter)

sa-token-rust 通过 `sa-token-adapter` 中定义的适配器 trait 与 Web 框架通信。本页介绍这些底层接口 — 适用于实现自定义框架插件或调试 Token 提取逻辑。

## SaRequest

`SaRequest` trait 抽象 HTTP 请求数据。每个框架插件为其请求类型提供实现。

```rust
pub trait SaRequest {
    // 必须实现
    fn get_header(&self, name: &str) -> Option<String>;
    fn get_cookie(&self, name: &str) -> Option<String>;
    fn get_param(&self, name: &str) -> Option<String>;
    fn get_path(&self) -> String;
    fn get_method(&self) -> String;

    // 有默认实现
    fn get_headers(&self) -> HashMap<String, String> { HashMap::new() }
    fn get_cookies(&self) -> HashMap<String, String> { HashMap::new() }
    fn get_params(&self) -> HashMap<String, String> { HashMap::new() }
    fn get_uri(&self) -> String { self.get_path() }
    fn get_body_json<T: DeserializeOwned>(&self) -> Option<T> { None }
    fn get_client_ip(&self) -> Option<String> { None }
    fn get_user_agent(&self) -> Option<String> { self.get_header("user-agent") }
}
```

### 方法表

| 方法 | 是否必须 | 说明 |
|---|---|---|
| `get_header` | 是 | 读取单个 HTTP 头 |
| `get_cookie` | 是 | 按名称读取单个 Cookie |
| `get_param` | 是 | 读取单个查询参数 |
| `get_path` | 是 | 获取请求 URL 路径 |
| `get_method` | 是 | 获取 HTTP 方法 |
| `get_headers` | 否 | 读取所有请求头 |
| `get_cookies` | 否 | 读取所有 Cookie |
| `get_params` | 否 | 读取所有查询参数 |
| `get_uri` | 否 | 获取完整请求 URI（默认：path） |
| `get_body_json` | 否 | 解析 JSON 请求体 |
| `get_client_ip` | 否 | 获取客户端 IP |
| `get_user_agent` | 否 | 获取 User-Agent 头 |

---

## SaResponse

设置响应数据的抽象。

```rust
pub trait SaResponse {
    // 必须实现
    fn set_header(&mut self, name: &str, value: &str);
    fn set_cookie(&mut self, name: &str, value: &str, options: CookieOptions);
    fn set_status(&mut self, status: u16);
    fn set_json_body<T: Serialize>(&mut self, body: T) -> Result<(), serde_json::Error>;

    // 有默认实现
    fn delete_cookie(&mut self, name: &str) {
        self.set_cookie(name, "", CookieOptions { max_age: Some(0), ..Default::default() });
    }
}
```

---

## CookieOptions

配置 `set_cookie` 的 Cookie 属性。

```rust
pub struct CookieOptions {
    pub domain: Option<String>,     // Cookie 域名
    pub path: Option<String>,       // Cookie 路径
    pub max_age: Option<i64>,       // 最大有效期（秒）
    pub http_only: bool,            // HttpOnly 标记
    pub secure: bool,               // Secure 标记（仅 HTTPS）
    pub same_site: Option<SameSite>, // SameSite 属性
}

pub enum SameSite {
    Strict,
    Lax,
    None,
}
```

**示例：**

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

## 工具函数

`sa_token_adapter::utils` 模块提供跨插件使用的辅助函数。

### `parse_cookies`

```rust
pub fn parse_cookies(cookie_header: &str) -> HashMap<String, String>
```

将原始 `Cookie` 头字符串（`"key1=value1; key2=value2"`）解析为键值映射。**不**进行 URL 解码。

### `parse_query_string`

```rust
pub fn parse_query_string(query: &str) -> HashMap<String, String>
```

将查询字符串（`"key1=value1&key2=value2"`）解析为键值映射。**会**对键和值进行 URL 解码。

### `build_cookie_string`

```rust
pub fn build_cookie_string(name: &str, value: &str, options: CookieOptions) -> String
```

根据名称、值和选项构建完整的 `Set-Cookie` 头值。

### `strip_bearer_prefix`

```rust
pub fn strip_bearer_prefix(auth_header: &str) -> Option<String>
```

严格模式：仅剥离 `Bearer ` 前缀，不存在则返回 `None`。用于需要区分 Bearer 和其他认证方案的场景。

### `extract_bearer_or_value`

```rust
pub fn extract_bearer_or_value(s: &str) -> String
```

宽松模式：有 `Bearer ` 则剥离，否则返回修剪后的输入。框架插件默认使用此方法从 Authorization 头提取 Token。

### `strip_bearer_or_passthrough`

```rust
pub fn strip_bearer_or_passthrough(s: &str) -> String
```

`extract_bearer_or_value` 的别名。新代码建议使用 `extract_bearer_or_value`。

---

## FrameworkAdapter

框架插件的最小生命周期 trait。

```rust
pub trait FrameworkAdapter: Send + Sync {
    fn name(&self) -> &str;
    async fn initialize(&self) -> Result<(), String>;
    async fn shutdown(&self) -> Result<(), String> { Ok(()) }
}
```

基本使用无需关心。用于框架插件需要初始化逻辑时（如 Redis 连接池设置）。

---

## 编写自定义插件

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
        // 解析 Cookie 头
        self.get_header("cookie")
            .map(|h| parse_cookies(&h))
            .and_then(|c| c.get(name).cloned())
    }

    fn get_param(&self, name: &str) -> Option<String> {
        // 从 self.path 的查询字符串解析
        None
    }

    fn get_path(&self) -> String { self.path.clone() }
    fn get_method(&self) -> String { self.method.clone() }
}
```

## 相关文档

- [存储后端](/zh/guide/storage)
- [框架集成](/zh/guide/framework-integration)
- [StpUtil API](/zh/guide/stp-util)
