# Proc Macros

[中文文档](/zh/guide/permission-matching) | English

sa-token-rust provides 8 procedural macros for declarative authentication and authorization. All macros work at **compile time**, inserting the appropriate check logic into the function body before compilation.

## Table of Contents

- [Overview](#overview)
- [Constraints](#constraints)
- [Macro Reference](#macro-reference)
- [Permission Matching Rules](#permission-matching-rules)
- [Role Matching Rules](#role-matching-rules)
- [Best Practices](#best-practices)

## Overview

| Macro | Purpose |
|---|---|
| `#[sa_check_login]` | Require user to be logged in |
| `#[sa_check_permission("p")]` | Require a specific permission |
| `#[sa_check_permissions_and("a","b")]` | Require ALL specified permissions |
| `#[sa_check_permissions_or("a","b")]` | Require ANY specified permission |
| `#[sa_check_role("r")]` | Require a specific role |
| `#[sa_check_roles_and("a","b")]` | Require ALL specified roles |
| `#[sa_check_roles_or("a","b")]` | Require ANY specified role |
| `#[sa_ignore]` | Skip all authentication checks |

## Constraints

All `#[sa_check_*]` macros share the same requirements:

1. **Function must be `async fn`** — compile-time error if not
2. **Return type must be `Result<T, E>` where `E: From<SaTokenError>`** — the `?` operator propagates auth errors
3. **Must use framework middleware** — `SaTokenLayer` (or equivalent) must be registered to inject the login context into the request, otherwise macros cannot read `login_id`

`#[sa_ignore]` can be applied to functions, structs, or impl blocks without the async requirement.

## Macro Reference

### `#[sa_check_login]`

Checks that the current request has a valid login context.

```rust
use sa_token_macro::sa_check_login;

#[sa_check_login]
async fn user_profile() -> Result<impl Responder, StatusCode> {
    // login_id is guaranteed to be available via StpUtil::get_login_id_as_string()
    Ok("Profile page")
}
```

**Expands to:**
```rust
async fn user_profile() -> Result<impl Responder, StatusCode> {
    sa_token_core::StpUtil::check_login_current()?;
    Ok("Profile page")
}
```

---

### `#[sa_check_permission("permission")]`

Checks the user has the exact permission. Supports wildcards (see [Permission Matching](#permission-matching-rules)).

```rust
use sa_token_macro::sa_check_permission;

#[sa_check_permission("user:delete")]
async fn delete_user() -> Result<impl Responder, StatusCode> {
    Ok("User deleted")
}

#[sa_check_permission("admin:*")]
async fn admin_dashboard() -> Result<impl Responder, StatusCode> {
    Ok("Admin dashboard")
}
```

**Expands to:**
```rust
async fn delete_user() -> Result<impl Responder, StatusCode> {
    let __login_id = sa_token_core::StpUtil::get_login_id_as_string().await?;
    sa_token_core::StpUtil::check_permission(&__login_id, "user:delete").await?;
    Ok("User deleted")
}
```

---

### `#[sa_check_permissions_and("a", "b", ...)]`

User must have ALL specified permissions.

```rust
#[sa_check_permissions_and("user:read", "user:write")]
async fn manage_users() -> Result<impl Responder, StatusCode> {
    Ok("User management")
}
```

**Expands to a single `has_permissions_and` + returns `PermissionDeniedDetail` on failure.**

---

### `#[sa_check_permissions_or("a", "b", ...)]`

User must have AT LEAST ONE specified permission.

```rust
#[sa_check_permissions_or("admin:panel", "super:admin")]
async fn admin_or_super() -> Result<impl Responder, StatusCode> {
    Ok("Admin panel")
}
```

**Expands to a single `has_permissions_or` + returns `PermissionDeniedDetail` on failure.**

---

### `#[sa_check_role("role")]`

Checks the user has the exact role.

```rust
use sa_token_macro::sa_check_role;

#[sa_check_role("admin")]
async fn admin_panel() -> Result<impl Responder, StatusCode> {
    Ok("Admin panel")
}
```

**Expands to:**
```rust
async fn admin_panel() -> Result<impl Responder, StatusCode> {
    let __login_id = sa_token_core::StpUtil::get_login_id_as_string().await?;
    sa_token_core::StpUtil::check_role(&__login_id, "admin").await?;
    Ok("Admin panel")
}
```

---

### `#[sa_check_roles_and("a", "b", ...)]`

User must have ALL specified roles. Each role is checked sequentially (short-circuits on first failure).

```rust
#[sa_check_roles_and("admin", "super")]
async fn super_admin_panel() -> Result<impl Responder, StatusCode> {
    Ok("Super admin panel")
}
```

**Returns `RoleDenied` on first missing role.**

---

### `#[sa_check_roles_or("a", "b", ...)]`

User must have AT LEAST ONE specified role.

```rust
#[sa_check_roles_or("admin", "moderator")]
async fn moderate_content() -> Result<impl Responder, StatusCode> {
    Ok("Content moderation")
}
```

**Returns `RoleDenied` if none of the roles match.**

---

### `#[sa_ignore]`

Skips ALL sa-token authentication checks. Has the highest priority — overrides any other `#[sa_check_*]` macros on the same item.

**Can be applied to:**
- Functions: skip auth for a single route handler
- Structs: skip auth for all methods in a controller
- impl blocks: skip auth for all methods in the impl block

```rust
use sa_token_macro::sa_ignore;

// Skip auth for a public endpoint
#[sa_ignore]
async fn health_check() -> &'static str {
    "OK"
}

// Skip auth for an entire controller
#[sa_ignore]
struct PublicController;

impl PublicController {
    async fn home() -> &'static str { "Home" }
    async fn about() -> &'static str { "About" }
}
```

---

## Permission Matching Rules

Permissions use `module:action` format with two-level wildcard support.

### Matching Algorithm

1. **Exact match** — `user:delete` matches `user:delete`
2. **Prefix wildcard** — permission ending in `:*` matches all children of that prefix
3. **Global wildcard** — `*` matches everything

### Wildcard Examples

| User has | Required | Result |
|---|---|---|
| `user:*` | `user:delete` | ✅ matches |
| `user:*` | `user:list` | ✅ matches |
| `user:*` | `admin:list` | ❌ different prefix |
| `admin:*` | `user:delete` | ❌ different prefix |
| `*` | `anything:here` | ✅ global |

### Implementation Detail

The wildcard match checks if a permission ends with `:*` and then verifies the required permission starts with that prefix (excluding the `*`). For example, `user:*` → prefix is `user:`. Required `user:delete` starts with `user:` → match.

**Important:** Only trailing `:*` is supported. Patterns like `admin:*:*` are NOT supported — use `admin:*` instead, which already matches `admin:user:delete`.

---

## Role Matching Rules

Role matching is **exact only** — no wildcards. A user's role list is compared against the required role via string equality.

| User has roles | Required | Result |
|---|---|---|
| `["admin"]` | `admin` | ✅ matches |
| `["user", "vip"]` | `admin` | ❌ no match |
| `["superadmin"]` | `admin` | ❌ no match (different string) |

---

## Best Practices

1. **Pair macros with middleware** — Always register `SaTokenLayer` (or equivalent) in your router so the request context is populated
2. **Use `#[sa_ignore]` for public routes** — login pages, health checks, static assets
3. **Prefer `module:action` naming** — `user:list`, `user:create`, `order:refund`
4. **Limit global wildcards** — `*` should only be used for super-admin accounts
5. **Handle error types** — Ensure your handler's error type implements `From<SaTokenError>`

## Related

- [StpUtil API](/guide/stp-util)
- [Framework Integration](/guide/framework-integration)
