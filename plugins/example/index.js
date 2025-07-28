// 示例插件 - JavaScript版本
// 这是一个演示插件，展示如何使用LinBot2插件系统

class ExamplePlugin {
    constructor() {
        this.name = "example_plugin";
        this.version = "1.0.0";
        this.description = "示例插件，展示插件系统的基本功能";
    }

    // 插件初始化
    async onInit(context) {
        console.log(`[${this.name}] 插件初始化`);
        this.context = context;
        
        // 注册命令
        this.registerCommands();
    }

    // 插件启动
    async onStart(context) {
        console.log(`[${this.name}] 插件启动`);
        await context.logger.info(this.name, "插件已启动");
    }

    // 插件停止
    async onStop(context) {
        console.log(`[${this.name}] 插件停止`);
        await context.logger.info(this.name, "插件已停止");
    }

    // 注册命令
    registerCommands() {
        // 这里应该调用命令管理器注册命令
        // 由于是示例，暂时只是记录
        console.log(`[${this.name}] 注册命令: /hello, /echo, /time`);
    }

    // 处理消息
    async handleMessage(context, message) {
        const plainText = message.get_plain_text();
        
        // 简单的关键词响应
        if (plainText.includes("你好") || plainText.includes("hello")) {
            await this.sendReply(context, message, "你好！我是示例插件 🤖");
            return true;
        }

        if (plainText.includes("帮助") || plainText.includes("help")) {
            const helpText = `
示例插件帮助：
• 发送 "你好" 或 "hello" - 获得问候
• 发送 "帮助" 或 "help" - 显示此帮助
• 使用命令 /hello - 问候命令
• 使用命令 /echo <文本> - 回显文本
• 使用命令 /time - 获取当前时间
            `.trim();
            
            await this.sendReply(context, message, helpText);
            return true;
        }

        return false; // 没有处理此消息
    }

    // 处理命令
    async handleCommand(context, command, message) {
        const commandName = this.extractCommandName(command.matched_text);
        
        switch (commandName) {
            case "hello":
                await this.handleHelloCommand(context, command, message);
                return true;
                
            case "echo":
                await this.handleEchoCommand(context, command, message);
                return true;
                
            case "time":
                await this.handleTimeCommand(context, command, message);
                return true;
                
            default:
                return false;
        }
    }

    // Hello命令处理
    async handleHelloCommand(context, command, message) {
        const senderName = message.get_sender_nickname() || "朋友";
        const replyText = `你好，${senderName}！欢迎使用LinBot2示例插件！ 👋`;
        await this.sendReply(context, message, replyText);
    }

    // Echo命令处理
    async handleEchoCommand(context, command, message) {
        if (command.args.length === 0) {
            await this.sendReply(context, message, "请提供要回显的文本，例如：/echo 你好世界");
            return;
        }
        
        const echoText = command.args.join(" ");
        await this.sendReply(context, message, `回显：${echoText}`);
    }

    // Time命令处理
    async handleTimeCommand(context, command, message) {
        const now = new Date();
        const timeText = `当前时间：${now.toLocaleString('zh-CN', {
            timeZone: 'Asia/Shanghai',
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit'
        })}`;
        
        await this.sendReply(context, message, timeText);
    }

    // 发送回复
    async sendReply(context, message, text) {
        try {
            if (message.is_group_message()) {
                await context.api.send_group_msg(message.group_id, text);
            } else {
                await context.api.send_private_msg(message.user_id, text);
            }
            
            await context.logger.info(this.name, `发送回复: ${text}`);
        } catch (error) {
            await context.logger.error(this.name, `发送消息失败: ${error.message}`);
        }
    }

    // 提取命令名称
    extractCommandName(matchedText) {
        // 移除前缀 "/"
        return matchedText.replace(/^\//, '').split(' ')[0];
    }

    // 获取插件信息
    getInfo() {
        return {
            name: this.name,
            version: this.version,
            author: "LinBot2 Team",
            description: this.description,
            homepage: "https://github.com/linjianyan0229/linbot2",
            dependencies: [],
            api_version: "1.0.0",
            tags: ["example", "demo"]
        };
    }

    // 获取插件优先级
    getPriority() {
        return 100; // 默认优先级
    }

    // 检查是否应该处理消息
    async shouldHandleMessage(message) {
        // 示例插件处理所有消息
        return true;
    }

    // 检查是否应该处理命令
    async shouldHandleCommand(command) {
        const commandName = this.extractCommandName(command.matched_text);
        return ["hello", "echo", "time"].includes(commandName);
    }

    // 获取状态信息
    async getStatus() {
        return {
            status: "running",
            uptime: Date.now() - (this.startTime || Date.now()),
            messages_processed: this.messagesProcessed || 0,
            commands_executed: this.commandsExecuted || 0
        };
    }

    // 健康检查
    async healthCheck() {
        return true; // 示例插件总是健康的
    }
}

// 导出插件类
if (typeof module !== 'undefined' && module.exports) {
    module.exports = ExamplePlugin;
}

// 浏览器环境
if (typeof window !== 'undefined') {
    window.ExamplePlugin = ExamplePlugin;
}
