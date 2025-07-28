use std::collections::HashMap;
use regex::Regex;
use serde::{Serialize, Deserialize};

use crate::plugins::{PluginResult, PluginError};
use crate::plugins::message::ParsedMessage;
use crate::plugins::config::GlobalPluginConfig;

/// 命令权限级别
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// 所有人
    Everyone,
    /// 群管理员
    GroupAdmin,
    /// 群主
    GroupOwner,
    /// 超级用户
    SuperUser,
    /// 自定义权限
    Custom(String),
}

/// 命令权限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPermission {
    /// 权限级别
    pub level: PermissionLevel,
    /// 允许的用户ID列表
    pub allowed_users: Vec<i64>,
    /// 禁止的用户ID列表
    pub denied_users: Vec<i64>,
    /// 允许的群组ID列表
    pub allowed_groups: Vec<i64>,
    /// 禁止的群组ID列表
    pub denied_groups: Vec<i64>,
    /// 是否只允许私聊使用
    pub private_only: bool,
    /// 是否只允许群聊使用
    pub group_only: bool,
}

impl Default for CommandPermission {
    fn default() -> Self {
        Self {
            level: PermissionLevel::Everyone,
            allowed_users: Vec::new(),
            denied_users: Vec::new(),
            allowed_groups: Vec::new(),
            denied_groups: Vec::new(),
            private_only: false,
            group_only: false,
        }
    }
}

impl CommandPermission {
    /// 检查用户是否有权限执行命令
    pub fn check_permission(&self, message: &ParsedMessage, user_role: &str) -> bool {
        // 检查用户黑名单
        if self.denied_users.contains(&message.user_id) {
            return false;
        }

        // 检查群组黑名单
        if let Some(group_id) = message.group_id {
            if self.denied_groups.contains(&group_id) {
                return false;
            }
        }

        // 检查消息类型限制
        if self.private_only && message.is_group_message() {
            return false;
        }
        if self.group_only && message.is_private_message() {
            return false;
        }

        // 检查用户白名单
        if !self.allowed_users.is_empty() && !self.allowed_users.contains(&message.user_id) {
            return false;
        }

        // 检查群组白名单
        if !self.allowed_groups.is_empty() {
            if let Some(group_id) = message.group_id {
                if !self.allowed_groups.contains(&group_id) {
                    return false;
                }
            } else {
                return false; // 私聊但要求在特定群组
            }
        }

        // 检查权限级别
        match &self.level {
            PermissionLevel::Everyone => true,
            PermissionLevel::GroupAdmin => {
                user_role == "admin" || user_role == "owner"
            }
            PermissionLevel::GroupOwner => {
                user_role == "owner"
            }
            PermissionLevel::SuperUser => {
                // 这里需要从配置中获取超级用户列表
                false // 暂时返回false，需要实现超级用户检查
            }
            PermissionLevel::Custom(_) => {
                // 自定义权限检查，需要插件自己实现
                true
            }
        }
    }
}

/// 命令模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandPattern {
    /// 精确匹配
    Exact(String),
    /// 前缀匹配
    Prefix(String),
    /// 正则表达式匹配
    Regex(String),
    /// 关键词匹配
    Keywords(Vec<String>),
}

impl CommandPattern {
    /// 检查消息是否匹配此模式
    pub fn matches(&self, message: &str, prefix: &str) -> PluginResult<Option<CommandMatch>> {
        let trimmed_message = message.trim();
        
        match self {
            CommandPattern::Exact(cmd) => {
                let full_cmd = format!("{}{}", prefix, cmd);
                if trimmed_message == full_cmd {
                    Ok(Some(CommandMatch {
                        pattern: self.clone(),
                        matched_text: full_cmd,
                        args: Vec::new(),
                        raw_args: String::new(),
                    }))
                } else {
                    Ok(None)
                }
            }
            CommandPattern::Prefix(cmd) => {
                let full_cmd = format!("{}{}", prefix, cmd);
                if trimmed_message.starts_with(&full_cmd) {
                    let args_start = full_cmd.len();
                    let raw_args = if args_start < trimmed_message.len() {
                        trimmed_message[args_start..].trim().to_string()
                    } else {
                        String::new()
                    };
                    
                    let args: Vec<String> = if raw_args.is_empty() {
                        Vec::new()
                    } else {
                        raw_args.split_whitespace().map(|s| s.to_string()).collect()
                    };

                    Ok(Some(CommandMatch {
                        pattern: self.clone(),
                        matched_text: full_cmd,
                        args,
                        raw_args,
                    }))
                } else {
                    Ok(None)
                }
            }
            CommandPattern::Regex(pattern) => {
                let regex = Regex::new(pattern)
                    .map_err(|e| PluginError::CommandMatchError(format!("正则表达式错误: {}", e)))?;
                
                if let Some(captures) = regex.captures(trimmed_message) {
                    let matched_text = captures.get(0).unwrap().as_str().to_string();
                    let args: Vec<String> = captures.iter()
                        .skip(1) // 跳过完整匹配
                        .filter_map(|m| m.map(|m| m.as_str().to_string()))
                        .collect();
                    
                    Ok(Some(CommandMatch {
                        pattern: self.clone(),
                        matched_text,
                        args: args.clone(),
                        raw_args: args.join(" "),
                    }))
                } else {
                    Ok(None)
                }
            }
            CommandPattern::Keywords(keywords) => {
                let message_lower = trimmed_message.to_lowercase();
                for keyword in keywords {
                    if message_lower.contains(&keyword.to_lowercase()) {
                        return Ok(Some(CommandMatch {
                            pattern: self.clone(),
                            matched_text: keyword.clone(),
                            args: Vec::new(),
                            raw_args: String::new(),
                        }));
                    }
                }
                Ok(None)
            }
        }
    }
}

/// 命令匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMatch {
    /// 匹配的模式
    pub pattern: CommandPattern,
    /// 匹配的文本
    pub matched_text: String,
    /// 解析的参数
    pub args: Vec<String>,
    /// 原始参数字符串
    pub raw_args: String,
}

impl CommandMatch {
    /// 获取参数数量
    #[allow(dead_code)]
    pub fn arg_count(&self) -> usize {
        self.args.len()
    }

    /// 获取指定位置的参数
    #[allow(dead_code)]
    pub fn get_arg(&self, index: usize) -> Option<&String> {
        self.args.get(index)
    }

    /// 获取参数作为整数
    #[allow(dead_code)]
    pub fn get_arg_as_i64(&self, index: usize) -> Option<i64> {
        self.get_arg(index)?.parse().ok()
    }

    /// 获取参数作为浮点数
    #[allow(dead_code)]
    pub fn get_arg_as_f64(&self, index: usize) -> Option<f64> {
        self.get_arg(index)?.parse().ok()
    }

    /// 获取从指定位置开始的所有参数
    #[allow(dead_code)]
    pub fn get_args_from(&self, start: usize) -> Vec<&String> {
        self.args.iter().skip(start).collect()
    }

    /// 获取从指定位置开始的参数作为字符串
    #[allow(dead_code)]
    pub fn get_args_from_as_string(&self, start: usize) -> String {
        self.get_args_from(start)
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .join(" ")
    }
}

/// 命令定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CommandDefinition {
    /// 命令名称
    pub name: String,
    /// 命令描述
    pub description: String,
    /// 命令模式
    pub patterns: Vec<CommandPattern>,
    /// 命令权限
    pub permission: CommandPermission,
    /// 命令别名
    pub aliases: Vec<String>,
    /// 使用示例
    pub examples: Vec<String>,
    /// 命令分类
    pub category: String,
    /// 是否启用
    pub enabled: bool,
    /// 冷却时间（秒）
    pub cooldown: u64,
    /// 命令优先级
    pub priority: i32,
}

impl Default for CommandDefinition {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            patterns: Vec::new(),
            permission: CommandPermission::default(),
            aliases: Vec::new(),
            examples: Vec::new(),
            category: "default".to_string(),
            enabled: true,
            cooldown: 0,
            priority: 100,
        }
    }
}

/// 命令管理器
pub struct CommandManager {
    /// 命令前缀
    prefix: String,
    /// 注册的命令
    commands: HashMap<String, CommandDefinition>,
    /// 命令冷却记录
    cooldowns: HashMap<String, std::time::Instant>,
    /// 是否已初始化
    initialized: bool,
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            prefix: "/".to_string(),
            commands: HashMap::new(),
            cooldowns: HashMap::new(),
            initialized: false,
        }
    }

    /// 初始化命令管理器
    pub async fn initialize(&mut self, global_config: &GlobalPluginConfig) -> PluginResult<()> {
        if self.initialized {
            return Ok(());
        }

        // 从配置中加载命令前缀
        self.prefix = global_config.command_prefix.clone();

        self.initialized = true;
        Ok(())
    }

    /// 设置命令前缀
    #[allow(dead_code)]
    pub fn set_prefix(&mut self, prefix: String) {
        self.prefix = prefix;
    }

    /// 获取命令前缀
    #[allow(dead_code)]
    pub fn get_prefix(&self) -> &str {
        &self.prefix
    }

    /// 注册命令
    #[allow(dead_code)]
    pub fn register_command(&mut self, command: CommandDefinition) -> PluginResult<()> {
        if self.commands.contains_key(&command.name) {
            return Err(PluginError::Other(format!("命令已存在: {}", command.name)));
        }

        self.commands.insert(command.name.clone(), command);
        Ok(())
    }

    /// 注销命令
    #[allow(dead_code)]
    pub fn unregister_command(&mut self, name: &str) -> PluginResult<()> {
        self.commands.remove(name)
            .ok_or_else(|| PluginError::Other(format!("命令不存在: {}", name)))?;
        Ok(())
    }

    /// 匹配命令
    pub async fn match_command(&self, message: &ParsedMessage) -> PluginResult<Option<CommandMatch>> {
        let plain_text = message.get_plain_text();
        
        // 按优先级排序命令
        let mut sorted_commands: Vec<_> = self.commands.values()
            .filter(|cmd| cmd.enabled)
            .collect();
        sorted_commands.sort_by_key(|cmd| cmd.priority);

        for command in sorted_commands {
            // 检查冷却时间
            if command.cooldown > 0 {
                let cooldown_key = format!("{}:{}", command.name, message.user_id);
                if let Some(last_use) = self.cooldowns.get(&cooldown_key) {
                    if last_use.elapsed().as_secs() < command.cooldown {
                        continue; // 还在冷却中
                    }
                }
            }

            // 尝试匹配命令模式
            for pattern in &command.patterns {
                if let Some(command_match) = pattern.matches(&plain_text, &self.prefix)? {
                    // 检查权限
                    let user_role = self.get_user_role(message);
                    if command.permission.check_permission(message, &user_role) {
                        return Ok(Some(command_match));
                    }
                }
            }

            // 检查别名
            for alias in &command.aliases {
                let alias_pattern = CommandPattern::Prefix(alias.clone());
                if let Some(command_match) = alias_pattern.matches(&plain_text, &self.prefix)? {
                    let user_role = self.get_user_role(message);
                    if command.permission.check_permission(message, &user_role) {
                        return Ok(Some(command_match));
                    }
                }
            }
        }

        Ok(None)
    }

    /// 获取用户角色
    fn get_user_role(&self, message: &ParsedMessage) -> String {
        // 这里需要实现获取用户在群组中的角色
        // 暂时返回默认值
        if message.is_group_message() {
            // 从sender中获取role信息
            message.sender.get("role")
                .and_then(|v| v.as_str())
                .unwrap_or("member")
                .to_string()
        } else {
            "user".to_string()
        }
    }

    /// 记录命令使用
    #[allow(dead_code)]
    pub fn record_command_use(&mut self, command_name: &str, user_id: i64) {
        let cooldown_key = format!("{}:{}", command_name, user_id);
        self.cooldowns.insert(cooldown_key, std::time::Instant::now());
    }

    /// 获取所有命令
    #[allow(dead_code)]
    pub fn get_all_commands(&self) -> &HashMap<String, CommandDefinition> {
        &self.commands
    }

    /// 获取命令帮助信息
    #[allow(dead_code)]
    pub fn get_command_help(&self, command_name: &str) -> Option<String> {
        self.commands.get(command_name).map(|cmd| {
            let mut help = format!("命令: {}\n", cmd.name);
            help.push_str(&format!("描述: {}\n", cmd.description));
            
            if !cmd.examples.is_empty() {
                help.push_str("示例:\n");
                for example in &cmd.examples {
                    help.push_str(&format!("  {}\n", example));
                }
            }
            
            help
        })
    }

    /// 获取命令列表
    #[allow(dead_code)]
    pub fn get_command_list(&self, category: Option<&str>) -> Vec<&CommandDefinition> {
        self.commands.values()
            .filter(|cmd| {
                cmd.enabled && category.map_or(true, |cat| cmd.category == cat)
            })
            .collect()
    }
}
