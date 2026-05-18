// Author: 金书记
//
//! # sa-token-plugin-tonic
//!
//! Tonic gRPC framework integration for sa-token-rust.
//!
//! This plugin provides authentication and authorization support for gRPC services
//! built with the Tonic framework.
//!
//! ## Features
//!
//! - **gRPC Interceptors**: Authenticate requests using token validation
//! - **Request Adaptation**: Convert gRPC metadata to Sa-Token requests
//! - **State Management**: Shared state across all gRPC services
//!
//! ## Usage
//!
//! ```toml
//! [dependencies]
//! sa-token-plugin-tonic = "0.1.15"  # Default using memory storage
//! ```
//!
//! ```rust,ignore
//! use sa_token_plugin_tonic::{SaTokenState, MemoryStorage, GrpcAuthInterceptor};
//! use tonic::{Status, Request, Response};
//! use tower::ServiceBuilder;
//!
//! // 1. Initialize state
//! let state = SaTokenState::builder()
//!     .storage(Arc::new(MemoryStorage::new()))
//!     .timeout(7200)
//!     .build();
//!
//! // 2. Create auth interceptor
//! let auth_interceptor = GrpcAuthInterceptor::new(state.clone());
//!
//! // 3. Apply to service with tower layer
//! let builder = ServiceBuilder::new()
//!     .layer(auth_interceptor.into_layer());
//! ```

#[cfg(not(feature = "tonic-012"))]
compile_error!(
    "sa-token-plugin-tonic: enable feature `tonic-012` (default)."
);

pub mod state;
pub mod adapter;
pub mod interceptor;
pub mod error;

pub use state::{SaTokenState, SaTokenStateBuilder};
pub use adapter::{TonicRequestAdapter, TonicResponseAdapter};
pub use interceptor::GrpcServerInterceptor;
pub use error::{AuthError, PermissionError, GrpcTokenData};

pub use sa_token_core::{self, prelude::*};
pub use sa_token_adapter::{self, storage::SaStorage, framework::FrameworkAdapter};
pub use sa_token_macro::*;

#[cfg(feature = "memory")]
pub use sa_token_storage_memory::MemoryStorage;

#[cfg(feature = "redis")]
pub use sa_token_storage_redis::RedisStorage;

#[cfg(feature = "database")]
pub use sa_token_storage_database::DatabaseStorage;
