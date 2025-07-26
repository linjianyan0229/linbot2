# LinBot2 🤖

一个基于 Tauri + Vue 3 构建的现代化 QQ 机器人管理平台，提供直观的图形界面来管理和监控您的 QQ 机器人。

## ✨ 项目特色

- 🎨 **自然柔和设计风格** - 采用淡米色主题，营造平静舒适的使用体验
- 🖥️ **跨平台桌面应用** - 基于 Tauri 框架，支持 Windows、macOS、Linux
- ⚡ **现代化前端技术** - Vue 3 + Vite，提供快速的开发和构建体验
- 🔧 **完整的机器人管理** - 服务器配置、好友管理、群聊管理、实时日志
- 📱 **响应式界面** - 适配不同屏幕尺寸，固定 1280x720 应用窗口
- 🎯 **实时监控** - 实时日志查看、自动滚动、智能过滤

## 🚀 主要功能

### 📊 监控面板
- 实时显示机器人运行状态
- 服务器连接状态监控
- 消息统计和性能指标
- 系统资源使用情况

### 🔧 服务器管理
- 多服务器配置支持
- 服务器启动/停止控制
- 连接状态实时监控
- 配置参数管理

### 👥 好友管理
- 好友列表查看和搜索
- 好友信息展示
- 私聊消息发送
- 好友状态监控

### 💬 群聊管理
- 群聊列表管理
- 群成员统计
- 群消息发送
- 群聊活跃度分析

### 📝 日志系统
- 实时日志流显示
- 智能日志过滤
- 自动滚动到最新消息
- 日志级别分类
- 心跳包过滤选项

### 🎛️ 许聊列表
- 许可的聊天对象管理
- 权限控制设置
- 黑白名单管理

## 🛠️ 技术栈

### 前端技术
- **Vue 3** - 渐进式 JavaScript 框架
- **Vite** - 下一代前端构建工具
- **Vue Router** - 官方路由管理器
- **CSS3** - 现代化样式设计

### 后端技术
- **Tauri** - 安全的跨平台桌面应用框架
- **Rust** - 系统级编程语言
- **Tokio** - 异步运行时
- **Serde** - 序列化/反序列化框架

### 设计系统
- **自然柔和风格** - 以舒适的视觉体验为核心
- **色彩方案**：
  - 背景色：淡米色 (#f5f5f1)
  - 主要文字：深绿色 (#4a593d)
  - 标题文字：中等绿色 (#6e8b67)
  - 卡片背景：奶白色 (#fffcf6)
  - 边框颜色：淡棕色 (#e4ddd3)
  - 按钮颜色：淡绿色 (#a9c3a6) / 深绿色 (#8fb58b)

## 🎯 设计理念

LinBot2 采用自然柔和的设计风格，旨在为用户提供一个平静、舒适的机器人管理环境：

- **15px 圆角** - 所有组件使用统一的圆角设计
- **30px 圆角** - 按钮采用更大的圆角，增强亲和力
- **轻柔阴影** - 使用微妙的阴影效果增加层次感
- **充足留白** - 合理的间距布局，避免视觉拥挤
- **平滑动画** - 所有交互都有流畅的过渡效果

## 📦 项目结构

```
linbot2/
├── src/                    # 前端源码
│   ├── components/         # Vue 组件
│   ├── views/             # 页面视图
│   ├── router/            # 路由配置
│   ├── assets/            # 静态资源
│   └── main.js            # 应用入口
├── src-tauri/             # Tauri 后端
│   ├── src/               # Rust 源码
│   ├── Cargo.toml         # Rust 依赖配置
│   └── tauri.conf.json    # Tauri 配置
├── public/                # 公共资源
└── package.json           # Node.js 依赖配置
```

## 🚀 快速开始

### 环境要求

- **Node.js** >= 16.0.0
- **Rust** >= 1.70.0
- **Tauri CLI** >= 1.0.0

### 安装依赖

```bash
# 安装前端依赖
npm install

# 安装 Tauri CLI (如果尚未安装)
npm install -g @tauri-apps/cli
```

### 开发模式

```bash
# 启动开发服务器
npm run tauri dev
```

### 构建应用

```bash
# 构建生产版本
npm run tauri build
```

## 🎨 界面预览

LinBot2 提供了直观美观的用户界面：

- **监控面板** - 一目了然的系统状态
- **服务器管理** - 简洁的配置界面
- **好友/群聊管理** - 卡片式布局，信息清晰
- **实时日志** - 专业的日志查看体验
- **响应式设计** - 适配 1280x720 固定窗口

## 🤝 贡献指南

我们欢迎所有形式的贡献！

1. Fork 本仓库
2. 创建您的特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交您的更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开一个 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [Tauri](https://tauri.app/) - 优秀的跨平台应用框架
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
- [Vite](https://vitejs.dev/) - 快速的构建工具

## 📞 联系我们

如果您有任何问题或建议，请通过以下方式联系我们：

- 提交 [Issue](https://github.com/linjianyan0229/linbot2/issues)
- 发送邮件至：2259596781@qq.com

---

## 🛠️ 开发环境设置

### 推荐的 IDE 配置

- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
