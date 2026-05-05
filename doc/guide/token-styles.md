# Token Styles

[中文文档](/zh/guide/token-styles.md) | English

sa-token-rust supports multiple token generation styles to meet different scenarios.

```rust
use sa_token_core::SaTokenConfig;
use sa_token_core::config::TokenStyle;

let config = SaTokenConfig::builder()
    .token_style(TokenStyle::Tik)  // Choose your preferred style
    .build_config();
```

## Available Token Styles

| Style | Length | Example | Use Case |
|-------|--------|---------|----------|
| **Uuid** | 36 chars | `550e8400-e29b-41d4-a716-446655440000` | Standard UUID format, universally recognized |
| **SimpleUuid** | 32 chars | `550e8400e29b41d4a716446655440000` | UUID without hyphens, more compact |
| **Random32** | 32 chars | `a3f5c9d8e2b7f4a6c1e8d3b9f2a7c5e1` | Random hex string, good security |
| **Random64** | 64 chars | `a3f5c9d8...` | Longer random string, higher security |
| **Random128** | 128 chars | `a3f5c9d8...` | Maximum random length, ultra-secure |
| **Jwt** | Variable | `eyJhbGc...` | Self-contained token with claims |
| **Hash** | 64 chars | `472c7dce...` | SHA256 hash with user info, traceable |
| **Timestamp** | ~30 chars | `1760404107094_a8f4f17d88fcddb8` | Includes timestamp, easy to track |
| **Tik** | 8 chars | `GIxYHHD5` | Short and shareable, perfect for URLs |

## Token Style Examples

```rust
// Uuid style (default)
.token_style(TokenStyle::Uuid)
// Output: 550e8400-e29b-41d4-a716-446655440000

// Hash style - includes user information in hash
.token_style(TokenStyle::Hash)
// Output: 472c7dceee2b3079a1ae70746f43ba99b91636292ba7811b3bc8985a1148836f

// Timestamp style - includes millisecond timestamp
.token_style(TokenStyle::Timestamp)
// Output: 1760404107094_a8f4f17d88fcddb8

// Tik style - short 8-character token
.token_style(TokenStyle::Tik)
// Output: GIxYHHD5

// JWT style - self-contained token with claims
.token_style(TokenStyle::Jwt)
.jwt_secret_key("your-secret-key")
// Output: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## Choosing the Right Token Style

- **Uuid/SimpleUuid**: Standard choice, widely compatible
- **Random32/64/128**: When you need random tokens with specific length
- **JWT**: When you need self-contained tokens with embedded information
- **Hash**: When you need tokens that can be traced back to user info
- **Timestamp**: When you need to know when the token was created
- **Tik**: When you need short tokens for sharing (URLs, QR codes, etc.)

## Detailed Style Descriptions

### Uuid (Default)
Standard UUID v4 format. 36 characters with hyphens. Universally recognized and compatible with most systems.

### SimpleUuid
UUID v4 without hyphens. 32 characters, more compact for storage and URL usage while maintaining the same uniqueness guarantees.

### Random32/64/128
Cryptographically random hex strings. Longer strings provide higher entropy:
- **Random32**: 128 bits of entropy
- **Random64**: 256 bits of entropy
- **Random128**: 512 bits of entropy

### Jwt
Self-contained JSON Web Token with customizable claims. Supports 8 algorithms (HS256/384/512, RS256/384/512, ES256/384). See [JWT Guide](/guide/jwt.md) for details.

### Hash
SHA256 hash combining user ID, timestamp, and random salt. The resulting token can be traced back to the originating user through the server's stored mapping.

### Timestamp
Token prefixed with a millisecond-precision Unix timestamp, followed by a random component. Useful for auditing and debugging - you can tell exactly when a token was issued just by looking at it.

### Tik
Ultra-short 8-character alphanumeric token. Designed for scenarios where tokens need to be shared verbally, printed on QR codes, or used in short URLs. Trade-off: shorter length means lower entropy - only use where the convenience outweighs the security implications.

## Run the Example

```bash
cargo run --example token_styles_example
```
