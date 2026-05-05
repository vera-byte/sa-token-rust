# Online User Management & Real-time Push

## English

### Overview

The Online User Management module provides real-time tracking of user online status and message push capabilities. Perfect for building chat applications, live notifications, and real-time collaboration tools.

### Key Features

- **Online Status Tracking** - Track user connections in real-time
- **Multi-Device Support** - Users can connect from multiple devices
- **Real-time Push** - Send messages to specific users or broadcast to all
- **Kick-Out Notifications** - Force logout with notifications
- **Activity Tracking** - Monitor user activity timestamps
- **Extensible Pushers** - Implement custom push mechanisms

### Quick Start

```rust
use sa_token_core::{OnlineManager, OnlineUser, InMemoryPusher};
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create online manager
    let manager = Arc::new(OnlineManager::new());
    
    // Register message pusher
    let pusher = Arc::new(InMemoryPusher::new());
    manager.register_pusher(pusher.clone()).await;
    
    // Mark user as online
    let user = OnlineUser {
        login_id: "user123".to_string(),
        token: "token123".to_string(),
        device: "web".to_string(),
        connect_time: chrono::Utc::now(),
        last_activity: chrono::Utc::now(),
        metadata: HashMap::new(),
    };
    manager.mark_online(user).await;
    
    // Push message to user
    manager.push_to_user("user123", "Hello!".to_string()).await?;
    
    // Broadcast to all users
    manager.broadcast("System announcement".to_string()).await?;
    
    // Check online status
    if manager.is_online("user123").await {
        println!("User is online");
    }
    
    Ok(())
}
```

### API Reference

#### OnlineManager Methods

- `new()` - Create manager
- `mark_online(user)` - Mark user online
- `mark_offline(login_id, token)` - Mark specific session offline
- `mark_offline_all(login_id)` - Mark all user sessions offline
- `is_online(login_id)` - Check if user is online
- `get_online_count()` - Get total online users
- `get_online_users()` - Get list of online user IDs
- `push_to_user(login_id, content)` - Push to single user
- `push_to_users(login_ids, content)` - Push to multiple users
- `broadcast(content)` - Push to all online users
- `kick_out_notify(login_id, reason)` - Force logout with notification

### Message Types

- `MessageType::Text` - Plain text
- `MessageType::Binary` - Binary data
- `MessageType::KickOut` - Logout notification
- `MessageType::Notification` - System notification
- `MessageType::Custom(String)` - Custom type

---

## Related Documentation

- [WebSocket Authentication](/guide/websocket-auth.md)
- [Distributed Session](/guide/distributed-session.md)
- [Event Listener Guide](/guide/event-listener.md)

## License

MIT OR Apache-2.0

