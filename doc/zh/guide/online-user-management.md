# 在线用户管理与实时推送

## 中文

### 概述

在线用户管理模块提供实时的用户在线状态跟踪和消息推送功能。非常适合构建聊天应用、实时通知和实时协作工具。

### 核心功能

- **在线状态跟踪** - 实时跟踪用户连接
- **多设备支持** - 用户可从多个设备连接
- **实时推送** - 向特定用户或所有用户发送消息
- **强制下线通知** - 强制登出并发送通知
- **活动跟踪** - 监控用户活动时间戳
- **可扩展推送器** - 实现自定义推送机制

### 快速开始

```rust
use sa_token_core::{OnlineManager, OnlineUser, InMemoryPusher};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建在线管理器
    let manager = Arc::new(OnlineManager::new());
    
    // 注册消息推送器
    let pusher = Arc::new(InMemoryPusher::new());
    manager.register_pusher(pusher.clone()).await;
    
    // 标记用户上线
    let user = OnlineUser {
        login_id: "user123".to_string(),
        token: "token123".to_string(),
        device: "web".to_string(),
        connect_time: chrono::Utc::now(),
        last_activity: chrono::Utc::now(),
        metadata: HashMap::new(),
    };
    manager.mark_online(user).await;
    
    // 推送消息给用户
    manager.push_to_user("user123", "你好！".to_string()).await?;
    
    // 广播给所有用户
    manager.broadcast("系统公告".to_string()).await?;
    
    Ok(())
}
```

### API 参考

#### OnlineManager 方法

- `new()` - 创建管理器
- `mark_online(user)` - 标记用户上线
- `mark_offline(login_id, token)` - 标记特定会话离线
- `mark_offline_all(login_id)` - 标记用户所有会话离线
- `is_online(login_id)` - 检查用户是否在线
- `get_online_count()` - 获取在线用户总数
- `push_to_user(login_id, content)` - 推送给单个用户
- `broadcast(content)` - 推送给所有在线用户
- `kick_out_notify(login_id, reason)` - 强制登出并通知

---


## Related Documentation

- [WebSocket Authentication](/zh/guide/websocket-auth.md)
- [Distributed Session](/zh/guide/distributed-session.md)
- [Event Listener Guide](/zh/guide/event-listener.md)

## License

MIT OR Apache-2.0

