// Author: 金书记
//
// 中文 | English
// sa-token-plugin-tonic 插件主入口 | sa-token-plugin-tonic plugin main entry point
//
//! # sa-token-plugin-tonic
//!
//! 中文: Tonic gRPC 框架集成插件
//! English: Tonic gRPC framework integration for sa-token-rust
//!
//! ## 功能特性 | Features
//!
//! 中文: 提供 gRPC 服务的认证和授权支持
//! English: Provides authentication and authorization support for gRPC services built with the Tonic framework
//!
//! ## 特性列表 | Feature List
//!
//! - **gRPC 拦截器 | gRPC Interceptors**: 使用 Token 验证进行请求认证
//! - **请求适配 | Request Adaptation**: 将 gRPC metadata 转换为 Sa-Token 请求
//! - **状态管理 | State Management**: 跨所有 gRPC 服务共享状态
//! - **权限检查 | Permission Checking**: 内置角色和权限验证辅助函数
//! - **错误类型 | Error Types**: gRPC 专用认证错误类型
//!
//! ## 使用示例 | Usage Example
//!
//! ```toml
//! [dependencies]
//! sa-token-plugin-tonic = "0.1.15"  # 中文: 默认使用内存存储 | English: Default using memory storage
//! ```
//!
//! ```rust,ignore
//! use sa_token_plugin_tonic::{SaTokenState, MemoryStorage, GrpcServerInterceptor};
//! use std::sync::Arc;
//!
//! // 1. 初始化状态 | Initialize state
//! let state = SaTokenState::builder()
//!     .storage(Arc::new(MemoryStorage::new()))
//!     .timeout(7200)
//!     .build();
//!
//! // 2. 创建认证拦截器 | Create auth interceptor
//! let interceptor = GrpcServerInterceptor::new(state);
//! ```

#[cfg(not(feature = "tonic-012"))]
compile_error!(
    "sa-token-plugin-tonic: enable feature `tonic-012` (default)."
);

// 中文: 公开模块 | English: Public modules
pub mod state;
pub mod adapter;
pub mod interceptor;
pub mod error;

// 中文: 公开类型导出 | English: Public type exports
pub use state::{SaTokenState, SaTokenStateBuilder};
pub use adapter::{TonicRequestAdapter, TonicResponseAdapter};
pub use interceptor::{GrpcServerInterceptor, get_login_id_from_request, validate_token_from_request, create_request_adapter};
pub use error::{AuthError, PermissionError, GrpcTokenData};

// 中文: 重新导出核心依赖 | English: Re-export core dependencies
pub use sa_token_core::{self, prelude::*};
pub use sa_token_adapter::{self, storage::SaStorage, framework::FrameworkAdapter};
pub use sa_token_macro::*;

// 中文: 根据特性条件编译存储实现 | English: Storage implementations with feature gates
#[cfg(feature = "memory")]
pub use sa_token_storage_memory::MemoryStorage;

#[cfg(feature = "redis")]
pub use sa_token_storage_redis::RedisStorage;

#[cfg(feature = "database")]
pub use sa_token_storage_database::DatabaseStorage;
