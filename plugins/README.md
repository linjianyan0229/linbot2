# LinBot2 插件系统

LinBot2 插件系统是一个强大且灵活的扩展框架，允许开发者创建自定义插件来扩展机器人的功能。

## 🚀 功能特性

### 核心功能
- **插件生命周期管理** - 完整的插件加载、启动、停止、卸载流程
- **消息处理系统** - 支持文本消息、CQ码、群聊、私聊等多种消息类型
- **命令系统** - 灵活的命令匹配和路由机制
- **权限控制** - 细粒度的用户和群组权限管理
- **配置管理** - 插件独立配置和全局配置支持

### OneBot API集成
- **完整API支持** - 发送消息、群管理、用户信息等
- **NapCat兼容** - 与NapCat框架无缝对接
- **错误处理** - 自动重试和错误恢复机制
- **异步处理** - 非阻塞的消息处理

### 安全特性
- **沙箱机制** - 限制插件的系统访问权限
- **资源监控** - 内存、CPU使用量监控和限制
- **文件系统控制** - 细粒度的文件访问权限
- **网络访问控制** - 域名和端口访问限制

## 📁 目录结构

```
plugins/
├── README.md                 # 本文档
├── example/                  # 示例插件
│   ├── plugin.toml          # 插件配置
│   └── index.js             # 插件代码
└── [your-plugin]/           # 你的插件目录
    ├── plugin.toml          # 插件配置
    ├── index.js             # 入口文件
    ├── config.toml          # 插件配置
    └── data/                # 插件数据目录
```

## 🔧 插件开发

### 1. 创建插件目录

```bash
mkdir plugins/my-plugin
cd plugins/my-plugin
```

### 2. 创建插件配置文件 `plugin.toml`

```toml
[info]
name = "my_plugin"
version = "1.0.0"
author = "Your Name"
description = "我的第一个插件"
api_version = "1.0.0"
tags = ["utility"]

plugin_type = "javascript"
entry_point = "index.js"

[dependencies]
# 插件依赖列表

[environment]
# 环境变量
```

### 3. 创建插件代码 `index.js`

```javascript
class MyPlugin {
    constructor() {
        this.name = "my_plugin";
        this.version = "1.0.0";
    }

    // 插件初始化
    async onInit(context) {
        console.log("插件初始化");
        this.context = context;
    }

    // 插件启动
    async onStart(context) {
        await context.logger.info(this.name, "插件已启动");
    }

    // 处理消息
    async handleMessage(context, message) {
        const text = message.get_plain_text();
        
        if (text.includes("hello")) {
            await this.sendReply(context, message, "Hello World!");
            return true;
        }
        
        return false;
    }

    // 处理命令
    async handleCommand(context, command, message) {
        // 命令处理逻辑
        return false;
    }

    // 发送回复
    async sendReply(context, message, text) {
        if (message.is_group_message()) {
            await context.api.send_group_msg(message.group_id, text);
        } else {
            await context.api.send_private_msg(message.user_id, text);
        }
    }

    // 获取插件信息
    getInfo() {
        return {
            name: this.name,
            version: this.version,
            author: "Your Name",
            description: "我的第一个插件"
        };
    }
}

module.exports = MyPlugin;
```

## 📋 API 参考

### 插件接口

#### 生命周期方法
- `onInit(context)` - 插件初始化
- `onStart(context)` - 插件启动
- `onStop(context)` - 插件停止
- `onUnload(context)` - 插件卸载
- `onConfigUpdate(context)` - 配置更新

#### 消息处理
- `handleMessage(context, message)` - 处理普通消息
- `handleGroupMessage(context, message)` - 处理群消息
- `handlePrivateMessage(context, message)` - 处理私聊消息

#### 命令处理
- `handleCommand(context, command, message)` - 处理命令
- `getCommandHelp(command)` - 获取命令帮助
- `getSupportedCommands()` - 获取支持的命令列表

#### 事件处理
- `handleNotice(context, notice)` - 处理通知事件
- `handleRequest(context, request)` - 处理请求事件
- `handleMetaEvent(context, meta)` - 处理元事件

### 上下文对象 (Context)

```javascript
context = {
    api: OneBotApi,           // OneBot API接口
    config: Map,              // 插件配置
    data_dir: Path,           // 插件数据目录
    logger: PluginLogger      // 日志记录器
}
```

### OneBot API

#### 消息发送
- `send_private_msg(user_id, message)` - 发送私聊消息
- `send_group_msg(group_id, message)` - 发送群聊消息
- `delete_msg(message_id)` - 撤回消息

#### 群管理
- `set_group_kick(group_id, user_id)` - 踢出群成员
- `set_group_ban(group_id, user_id, duration)` - 禁言群成员
- `set_group_admin(group_id, user_id, enable)` - 设置管理员

#### 信息获取
- `get_login_info()` - 获取登录信息
- `get_friend_list()` - 获取好友列表
- `get_group_list()` - 获取群列表
- `get_group_member_list(group_id)` - 获取群成员列表

### 消息对象 (ParsedMessage)

```javascript
message = {
    message_id: number,       // 消息ID
    message_type: string,     // 消息类型 (group/private)
    user_id: number,          // 发送者ID
    group_id: number,         // 群ID (群消息)
    message: string,          // 原始消息
    cq_codes: Array,          // CQ码数组
    
    // 方法
    get_plain_text(),         // 获取纯文本
    is_group_message(),       // 是否群消息
    is_private_message(),     // 是否私聊消息
    get_sender_nickname(),    // 获取发送者昵称
    is_at_bot(bot_id),       // 是否艾特机器人
}
```

### CQ码支持

#### 常用CQ码
- `[CQ:at,qq=123456]` - 艾特用户
- `[CQ:image,file=xxx.jpg]` - 图片
- `[CQ:face,id=123]` - 表情
- `[CQ:record,file=xxx.mp3]` - 语音
- `[CQ:reply,id=123456]` - 回复消息

#### 消息构建器
```javascript
const MessageBuilder = require('./message_builder');

const message = new MessageBuilder()
    .text("Hello ")
    .at(123456)
    .text(" 你好！")
    .image("image.jpg")
    .build();
```

## ⚙️ 配置系统

### 全局配置 `config/plugins.toml`

```toml
command_prefix = "/"
plugins_dir = "plugins"
enabled = true
max_plugins = 50
plugin_timeout = 30
hot_reload = true
log_level = "info"

[security]
enable_sandbox = true
verify_signatures = false
allow_network = true
max_memory_mb = 256
max_cpu_percent = 50.0

[performance]
worker_threads = 4
message_queue_size = 1000
startup_timeout = 30
shutdown_timeout = 10
```

### 插件配置 `plugins/[plugin-name]/config.toml`

```toml
name = "my_plugin"
enabled = true

[settings]
# 插件自定义设置
api_key = "your-api-key"
max_requests = 100

[permissions]
send_messages = true
read_messages = true
manage_groups = false
file_system = true
network = true

[limits]
max_memory_mb = 128
max_cpu_percent = 25.0
max_file_size_mb = 10
```

## 🔒 权限系统

### 命令权限

```javascript
const permission = {
    level: "GroupAdmin",      // Everyone/GroupAdmin/GroupOwner/SuperUser
    allowed_users: [123456],  // 允许的用户ID
    denied_users: [789012],   // 禁止的用户ID
    allowed_groups: [111222], // 允许的群组ID
    denied_groups: [333444],  // 禁止的群组ID
    private_only: false,      // 仅私聊
    group_only: false         // 仅群聊
};
```

### 文件系统权限

插件只能访问以下目录：
- `plugins/[plugin-name]/` - 插件自己的目录
- `plugins/[plugin-name]/data/` - 插件数据目录
- `temp/` - 临时文件目录

### 网络访问权限

可以通过配置限制插件的网络访问：
- 域名白名单
- 端口限制
- 带宽限制

## 📊 监控和日志

### 插件状态监控
- 内存使用量
- CPU使用率
- 消息处理数量
- 错误次数
- 运行时间

### 日志系统
```javascript
// 在插件中使用日志
await context.logger.debug(this.name, "调试信息");
await context.logger.info(this.name, "普通信息");
await context.logger.warn(this.name, "警告信息");
await context.logger.error(this.name, "错误信息");
```

## 🚀 部署和管理

### 安装插件
1. 将插件文件放入 `plugins/` 目录
2. 重启应用或使用热重载
3. 在管理界面中启用插件

### 插件管理命令
- 启用插件：`enable_plugin(plugin_id)`
- 禁用插件：`disable_plugin(plugin_id)`
- 卸载插件：`unload_plugin(plugin_id)`
- 获取插件列表：`get_all_plugins()`
- 获取插件状态：`get_plugin_stats(plugin_id)`

## 🔧 故障排除

### 常见问题

1. **插件加载失败**
   - 检查 `plugin.toml` 配置文件格式
   - 确认入口文件存在
   - 查看错误日志

2. **权限被拒绝**
   - 检查插件权限配置
   - 确认用户/群组权限设置
   - 查看安全配置

3. **消息发送失败**
   - 检查OneBot连接状态
   - 确认API权限
   - 查看网络连接

### 调试技巧

1. **启用调试日志**
   ```toml
   log_level = "debug"
   ```

2. **使用控制台输出**
   ```javascript
   console.log("调试信息");
   ```

3. **检查插件状态**
   ```javascript
   const stats = await getPluginStats(plugin_id);
   console.log(stats);
   ```

## 📚 示例插件

查看 `plugins/example/` 目录中的示例插件，了解完整的插件开发流程。

## 🤝 贡献

欢迎提交插件和改进建议！请遵循以下步骤：

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 创建 Pull Request

## 📄 许可证

本项目采用 MIT 许可证。详见 LICENSE 文件。
