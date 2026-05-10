# 过程宏

中文文档 | [English](/guide/permission-matching)

sa-token-rust 提供 8 个过程宏用于声明式认证授权。所有宏在**编译期**工作，将检查逻辑插入函数体。

## 目录

- [概述](#概述)
- [约束条件](#约束条件)
- [宏参考](#宏参考)
- [权限匹配规则](#权限匹配规则)
- [角色匹配规则](#角色匹配规则)
- [最佳实践](#最佳实践)

## 概述

| 宏 | 用途 |
|---|---|
| `#[sa_check_login]` | 要求用户已登录 |
| `#[sa_check_permission("p")]` | 要求特定权限 |
| `#[sa_check_permissions_and("a","b")]` | 要求拥有全部指定权限 |
| `#[sa_check_permissions_or("a","b")]` | 要求拥有任一指定权限 |
| `#[sa_check_role("r")]` | 要求特定角色 |
| `#[sa_check_roles_and("a","b")]` | 要求拥有全部指定角色 |
| `#[sa_check_roles_or("a","b")]` | 要求拥有任一指定角色 |
| `#[sa_ignore]` | 跳过所有认证检查 |

## 约束条件

所有 `#[sa_check_*]` 宏共享以下要求：

1. **函数必须是 `async fn`** — 非 async 函数会触发编译错误
2. **返回值必须是 `Result<T, E>` 且 `E: From<SaTokenError>`** — `?` 操作符传播认证错误
3. **必须配合框架中间件** — 需注册 `SaTokenLayer`（或等价物）注入登录上下文，否则宏无法读取 `login_id`

`#[sa_ignore]` 可应用于函数、结构体或 impl 块，无需 async 约束。

## 宏参考

### `#[sa_check_login]`

检查当前请求是否有有效的登录上下文。

```rust
use sa_token_macro::sa_check_login;

#[sa_check_login]
async fn user_profile() -> Result<impl Responder, StatusCode> {
    // login_id 保证可通过 StpUtil::get_login_id_as_string() 获取
    Ok("个人资料页")
}
```

**展开为：**
```rust
async fn user_profile() -> Result<impl Responder, StatusCode> {
    sa_token_core::StpUtil::check_login_current()?;
    Ok("个人资料页")
}
```

---

### `#[sa_check_permission("权限")]`

检查用户是否拥有精确权限。支持通配符（见[权限匹配规则](#权限匹配规则)）。

```rust
use sa_token_macro::sa_check_permission;

#[sa_check_permission("user:delete")]
async fn delete_user() -> Result<impl Responder, StatusCode> {
    Ok("用户已删除")
}

#[sa_check_permission("admin:*")]
async fn admin_dashboard() -> Result<impl Responder, StatusCode> {
    Ok("管理后台")
}
```

**展开为：**
```rust
async fn delete_user() -> Result<impl Responder, StatusCode> {
    let __login_id = sa_token_core::StpUtil::get_login_id_as_string().await?;
    sa_token_core::StpUtil::check_permission(&__login_id, "user:delete").await?;
    Ok("用户已删除")
}
```

---

### `#[sa_check_permissions_and("a", "b", ...)]`

用户必须拥有全部指定权限。

```rust
#[sa_check_permissions_and("user:read", "user:write")]
async fn manage_users() -> Result<impl Responder, StatusCode> {
    Ok("用户管理")
}
```

**展开为单次 `has_permissions_and` 调用，失败返回 `PermissionDeniedDetail`。**

---

### `#[sa_check_permissions_or("a", "b", ...)]`

用户只需拥有任一指定权限。

```rust
#[sa_check_permissions_or("admin:panel", "super:admin")]
async fn admin_or_super() -> Result<impl Responder, StatusCode> {
    Ok("管理面板")
}
```

**展开为单次 `has_permissions_or` 调用，失败返回 `PermissionDeniedDetail`。**

---

### `#[sa_check_role("角色")]`

检查用户是否拥有精确角色。

```rust
use sa_token_macro::sa_check_role;

#[sa_check_role("admin")]
async fn admin_panel() -> Result<impl Responder, StatusCode> {
    Ok("管理面板")
}
```

**展开为：**
```rust
async fn admin_panel() -> Result<impl Responder, StatusCode> {
    let __login_id = sa_token_core::StpUtil::get_login_id_as_string().await?;
    sa_token_core::StpUtil::check_role(&__login_id, "admin").await?;
    Ok("管理面板")
}
```

---

### `#[sa_check_roles_and("a", "b", ...)]`

用户必须拥有全部指定角色。逐个检查，首个失败即短路。

```rust
#[sa_check_roles_and("admin", "super")]
async fn super_admin_panel() -> Result<impl Responder, StatusCode> {
    Ok("超级管理员面板")
}
```

**首个缺失角色返回 `RoleDenied`。**

---

### `#[sa_check_roles_or("a", "b", ...)]`

用户只需拥有任一指定角色。

```rust
#[sa_check_roles_or("admin", "moderator")]
async fn moderate_content() -> Result<impl Responder, StatusCode> {
    Ok("内容审核")
}
```

**全部不匹配时返回 `RoleDenied`。**

---

### `#[sa_ignore]`

跳过所有 sa-token 认证检查。优先级最高 — 覆盖同一项上的任何其他 `#[sa_check_*]` 宏。

**可应用于：**
- 函数：单个路由跳过认证
- 结构体：控制器所有方法跳过认证
- impl 块：impl 块中所有方法跳过认证

```rust
use sa_token_macro::sa_ignore;

// 公开接口跳过认证
#[sa_ignore]
async fn health_check() -> &'static str {
    "OK"
}

// 整个控制器跳过认证
#[sa_ignore]
struct PublicController;

impl PublicController {
    async fn home() -> &'static str { "首页" }
    async fn about() -> &'static str { "关于" }
}
```

---

## 权限匹配规则

### 匹配算法

1. **精确匹配** — `user:delete` 匹配 `user:delete`
2. **前缀通配** — 以 `:*` 结尾的权限匹配该前缀下所有子项
3. **全局通配** — `*` 匹配一切

### 通配符示例

| 用户权限 | 需求权限 | 结果 |
|---|---|---|
| `user:*` | `user:delete` | ✅ |
| `user:*` | `user:list` | ✅ |
| `user:*` | `admin:list` | ❌ 不同前缀 |
| `*` | `anything:here` | ✅ |

### 实现细节

通配符匹配检测权限是否以 `:*` 结尾，然后验证需求权限是否以对应前缀开头。例如：`user:*` → 前缀为 `user:`，需求 `user:delete` 以 `user:` 开头 → 匹配。

**注意：** 仅支持尾部 `:*` 通配。`admin:*:*` 等模式不支持 — 直接用 `admin:*` 即可匹配 `admin:user:delete`。

---

## 角色匹配规则

角色匹配是**精确匹配**，不支持通配符。

| 用户角色 | 需求角色 | 结果 |
|---|---|---|
| `["admin"]` | `admin` | ✅ |
| `["user", "vip"]` | `admin` | ❌ |
| `["superadmin"]` | `admin` | ❌ 字符串不同 |

---

## 最佳实践

1. **宏与中间件配套使用** — 始终在路由中注册 `SaTokenLayer`（或等价物）
2. **公开路由用 `#[sa_ignore]`** — 登录页、健康检查、静态资源
3. **权限命名用 `模块:操作` 格式** — `user:list`、`user:create`、`order:refund`
4. **全局通配符仅给超级管理员** — `*` 权限范围过大
5. **错误类型实现 `From<SaTokenError>`** — 确保处理器能正确传播认证错误

## 相关文档

- [StpUtil API](/zh/guide/stp-util)
- [框架集成](/zh/guide/framework-integration)
