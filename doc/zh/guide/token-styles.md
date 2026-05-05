# Token 风格

[English](/guide/token-styles.md) | 中文文档

sa-token-rust 支持多种 Token 生成风格，满足不同场景需求。

```rust
use sa_token_core::SaTokenConfig;
use sa_token_core::config::TokenStyle;

let config = SaTokenConfig::builder()
    .token_style(TokenStyle::Tik)  // 选择你喜欢的风格
    .build_config();
```

## 可用的 Token 风格

| 风格 | 长度 | 示例 | 使用场景 |
|------|------|------|----------|
| **Uuid** | 36 字符 | `550e8400-e29b-41d4-a716-446655440000` | 标准 UUID 格式，通用性强 |
| **SimpleUuid** | 32 字符 | `550e8400e29b41d4a716446655440000` | 无横杠的 UUID，更紧凑 |
| **Random32** | 32 字符 | `a3f5c9d8e2b7f4a6c1e8d3b9f2a7c5e1` | 随机十六进制字符串，安全性好 |
| **Random64** | 64 字符 | `a3f5c9d8...` | 更长的随机字符串，安全性更高 |
| **Random128** | 128 字符 | `a3f5c9d8...` | 最长随机字符串，超高安全性 |
| **Jwt** | 可变长度 | `eyJhbGc...` | 自包含令牌，带有声明信息 |
| **Hash** | 64 字符 | `472c7dce...` | SHA256 哈希，包含用户信息，可追溯 |
| **Timestamp** | ~30 字符 | `1760404107094_a8f4f17d88fcddb8` | 包含时间戳，易于追踪 |
| **Tik** | 8 字符 | `GIxYHHD5` | 短小精悍，适合分享 |

## Token 风格示例

```rust
// Uuid 风格（默认）
.token_style(TokenStyle::Uuid)
// 输出: 550e8400-e29b-41d4-a716-446655440000

// Hash 风格 - 哈希中包含用户信息
.token_style(TokenStyle::Hash)
// 输出: 472c7dceee2b3079a1ae70746f43ba99b91636292ba7811b3bc8985a1148836f

// Timestamp 风格 - 包含毫秒级时间戳
.token_style(TokenStyle::Timestamp)
// 输出: 1760404107094_a8f4f17d88fcddb8

// Tik 风格 - 短小的8位字符 token
.token_style(TokenStyle::Tik)
// 输出: GIxYHHD5

// JWT 风格 - 自包含令牌
.token_style(TokenStyle::Jwt)
.jwt_secret_key("your-secret-key")
// 输出: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## 如何选择 Token 风格

- **Uuid/SimpleUuid**: 标准选择，兼容性广
- **Random32/64/128**: 需要特定长度的随机 token 时
- **JWT**: 需要自包含令牌，内嵌信息时
- **Hash**: 需要可追溯到用户信息的 token 时
- **Timestamp**: 需要知道 token 创建时间时
- **Tik**: 需要短小 token 用于分享（URL、二维码等）时

## 风格详解

### Uuid（默认）
标准 UUID v4 格式。36 个字符，带连字符。通用性强，与大多数系统兼容。

### SimpleUuid
无连字符的 UUID v4。32 个字符，存储和 URL 使用更紧凑，同时保持相同的唯一性保证。

### Random32/64/128
密码学随机十六进制字符串。更长的字符串提供更高的熵值：
- **Random32**：128 位熵
- **Random64**：256 位熵
- **Random128**：512 位熵

### Jwt
自包含的 JSON Web Token，支持自定义声明。支持 8 种算法（HS256/384/512、RS256/384/512、ES256/384）。详见 [JWT 指南](/zh/guide/jwt.md)。

### Hash
SHA256 哈希，结合用户 ID、时间戳和随机盐值生成。生成的 token 可以通过服务器存储的映射追溯到原始用户。

### Timestamp
以毫秒精度的 Unix 时间戳为前缀，后跟随机组件。对于审计和调试非常有用——仅通过查看即可知道 token 的签发时间。

### Tik
超短 8 字符字母数字 token。专为需要口头分享、打印在二维码上或在短 URL 中使用 token 的场景设计。权衡：长度更短意味着熵值更低——仅在便利性超过安全影响时使用。

## 运行示例

```bash
cargo run --example token_styles_example
```
