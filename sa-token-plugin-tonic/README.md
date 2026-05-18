# sa-token-plugin-tonic

Tonic gRPC framework integration for sa-token-rust.

## Overview

This plugin provides authentication and authorization support for gRPC services built with the [Tonic](https://github.com/hyperium/tonic) framework. It allows you to use sa-token's powerful authentication features in your gRPC services without relying on traditional web frameworks.

## Features

- **gRPC Interceptors**: Authenticate requests using token validation
- **Request Adaptation**: Convert gRPC metadata to Sa-Token requests
- **State Management**: Shared state across all gRPC services
- **Permission Checking**: Built-in helpers for role and permission verification
- **Error Types**: gRPC-specific authentication errors

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sa-token-plugin-tonic = "0.1.15"  # Default using memory storage

# Or with specific features
sa-token-plugin-tonic = { version = "0.1.15", features = ["redis"] }
```

### Features

- `tonic-012` (default): Enable Tonic 0.12 support
- `memory` (default): Enable in-memory storage
- `redis`: Enable Redis storage
- `database`: Enable database storage
- `full`: Enable all storage backends

## Quick Start

### 1. Initialize State

```rust
use sa_token_plugin_tonic::{SaTokenState, MemoryStorage};
use std::sync::Arc;

let state = SaTokenState::builder()
    .storage(Arc::new(MemoryStorage::new()))
    .timeout(7200)  // Token timeout in seconds
    .build();
```

### 2. Create and Use Interceptor

```rust
use sa_token_plugin_tonic::GrpcServerInterceptor;
use tonic::{Request, Status};

let interceptor = GrpcServerInterceptor::new(state);

// Use with tonic server
tonic::transport::Server::builder()
    .add_service(my_service)
    .build()
    .await?;
```

### 3. Access Login ID in Service Implementation

```rust
use sa_token_plugin_tonic::get_login_id_from_request;
use tonic::{Request, Response, Status};

async fn my_unary_call(
    request: Request<()>,
) -> Result<Response<MyResponse>, Status> {
    // Get the authenticated login_id from request extensions
    let login_id = get_login_id_from_request(&request)
        .ok_or_else(|| Status::auth("Not authenticated"))?;
    
    // Use login_id for authorization
    println!("Authenticated user: {}", login_id);
    
    Ok(Response::new(MyResponse { /* ... */ }))
}
```

### 4. Check Permissions

```rust
use sa_token_plugin_tonic::{check_permission, check_role};

async fn admin_endpoint(request: Request<()>) -> Result<Response<()>, Status> {
    let login_id = get_login_id_from_request(&request)
        .ok_or_else(|| Status::auth("Not authenticated"))?;
    
    // Check permission
    if !check_permission(&login_id, "admin:write").await {
        return Err(Status::permission_denied("Insufficient permissions"));
    }
    
    Ok(Response::new(()))
}
```

## Usage with tower (Advanced)

For more complex use cases, you can work directly with the adapter:

```rust
use sa_token_plugin_tonic::{
    TonicRequestAdapter, 
    create_request_adapter,
    SaTokenState,
};

fn handle_request(
    metadata: &tonic::metadata::MetadataMap,
    state: &SaTokenState,
) {
    let adapter = create_request_adapter(metadata, "POST", "/my/rpc");
    
    // Access headers
    let auth_header = adapter.get_header("authorization");
    
    // Access path and method
    let path = adapter.get_path();
    let method = adapter.get_method();
}
```

## gRPC Metadata

The interceptor reads authentication tokens from gRPC metadata in the following order:

1. Header with the configured token name (default: `satoken`)
2. `authorization` header
3. `Authorization` header

Tokens should be provided as Bearer tokens:
```
metadata: { "satoken": "Bearer xxx" }
```

Or simply:
```
metadata: { "satoken": "xxx" }
```

## Error Handling

The plugin provides gRPC-specific error types:

| gRPC Status | Condition |
|------------|-----------|
| `Unauthenticated` | Missing or invalid token |
| `PermissionDenied` | Token valid but insufficient permissions |

## Comparison with HTTP Plugins

Unlike web framework plugins (axum, actix-web, etc.), this plugin:

- Does not use HTTP Layer/Middleware concepts directly
- Works with gRPC's native interceptor mechanism
- Uses gRPC metadata instead of HTTP headers for token transport
- Provides synchronous interceptor interface (required by tonic)

## License

MIT OR Apache-2.0
