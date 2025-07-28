pub mod manager;
pub mod plugin_trait;
pub mod config;
pub mod api;
pub mod message;
pub mod command;
pub mod loader;
pub mod security;
pub mod logger;

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

pub use manager::PluginManager;
pub use plugin_trait::{Plugin, PluginInfo, PluginContext, PluginMetadata};
pub use config::{PluginConfig, GlobalPluginConfig};
pub use api::OneBotApi;
pub use message::MessageParser;
pub use command::CommandManager;

/// 插件状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub enum PluginStatus {
    /// 未加载
    Unloaded,
    /// 已加载但未启用
    Loaded,
    /// 已启用运行中
    Running,
    /// 已暂停
    Paused,
    /// 错误状态
    Error(String),
}

/// 插件运行统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStats {
    /// 处理的消息数量
    pub messages_processed: u64,
    /// 执行的命令数量
    pub commands_executed: u64,
    /// 发送的消息数量
    pub messages_sent: u64,
    /// 错误次数
    pub error_count: u64,
    /// 启动时间
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 最后活动时间
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for PluginStats {
    fn default() -> Self {
        Self {
            messages_processed: 0,
            commands_executed: 0,
            messages_sent: 0,
            error_count: 0,
            start_time: None,
            last_activity: None,
        }
    }
}

/// 插件实例包装器
pub struct PluginInstance {
    /// 插件唯一ID
    pub id: Uuid,
    /// 插件信息
    pub info: PluginInfo,
    /// 插件状态
    pub status: PluginStatus,
    /// 运行统计
    pub stats: PluginStats,
    /// 插件配置
    pub config: PluginConfig,
    /// 插件实例（使用Arc包装以支持多线程访问）
    pub plugin: Option<Arc<dyn Plugin + Send + Sync>>,
}

impl PluginInstance {
    pub fn new(info: PluginInfo, config: PluginConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            info,
            status: PluginStatus::Unloaded,
            stats: PluginStats::default(),
            config,
            plugin: None,
        }
    }

    /// 更新统计信息
    #[allow(dead_code)]
    pub fn update_stats<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut PluginStats),
    {
        updater(&mut self.stats);
        self.stats.last_activity = Some(chrono::Utc::now());
    }

    /// 检查插件是否正在运行
    pub fn is_running(&self) -> bool {
        matches!(self.status, PluginStatus::Running)
    }

    /// 检查插件是否可以处理消息
    pub fn can_process_messages(&self) -> bool {
        self.is_running() && self.plugin.is_some()
    }
}

/// 插件系统错误类型
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum PluginError {
    #[error("插件未找到: {0}")]
    PluginNotFound(String),
    
    #[error("插件已存在: {0}")]
    PluginAlreadyExists(String),
    
    #[error("插件加载失败: {0}")]
    LoadError(String),
    
    #[error("插件配置错误: {0}")]
    ConfigError(String),
    
    #[error("权限不足: {0}")]
    PermissionDenied(String),
    
    #[error("API调用失败: {0}")]
    ApiError(String),
    
    #[error("消息解析失败: {0}")]
    MessageParseError(String),
    
    #[error("命令匹配失败: {0}")]
    CommandMatchError(String),
    
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("序列化错误: {0}")]
    SerdeError(#[from] serde_json::Error),
    
    #[error("其他错误: {0}")]
    Other(String),
}

pub type PluginResult<T> = Result<T, PluginError>;

/// 插件系统全局状态
pub struct PluginSystem {
    /// 插件管理器
    pub manager: Arc<RwLock<PluginManager>>,
    /// 命令管理器
    pub command_manager: Arc<RwLock<CommandManager>>,
    /// OneBot API接口
    #[allow(dead_code)]
    pub onebot_api: Arc<OneBotApi>,
    /// 全局配置
    pub global_config: Arc<RwLock<GlobalPluginConfig>>,
}

impl PluginSystem {
    pub fn new(onebot_api: Arc<OneBotApi>) -> Self {
        Self {
            manager: Arc::new(RwLock::new(PluginManager::new())),
            command_manager: Arc::new(RwLock::new(CommandManager::new())),
            onebot_api,
            global_config: Arc::new(RwLock::new(GlobalPluginConfig::default())),
        }
    }

    /// 初始化插件系统
    pub async fn initialize(&self) -> PluginResult<()> {
        // 创建插件目录
        let plugins_dir = std::path::Path::new("plugins");
        if !plugins_dir.exists() {
            std::fs::create_dir_all(plugins_dir)?;
        }

        // 加载全局配置
        let mut global_config = self.global_config.write().await;
        *global_config = GlobalPluginConfig::load_or_default().await?;

        // 初始化插件管理器
        let mut manager = self.manager.write().await;
        manager.initialize().await?;

        // 初始化命令管理器
        let mut cmd_manager = self.command_manager.write().await;
        cmd_manager.initialize(&global_config).await?;

        Ok(())
    }

    /// 处理OneBot消息
    #[allow(dead_code)]
    pub async fn handle_message(&self, message: &crate::onebot::OneBotEvent) -> PluginResult<()> {
        let manager = self.manager.read().await;
        let cmd_manager = self.command_manager.read().await;

        // 解析消息
        let parsed_message = MessageParser::parse_onebot_event(message)?;

        // 检查是否为命令
        if let Some(command_match) = cmd_manager.match_command(&parsed_message).await? {
            // 处理命令
            manager.handle_command(&command_match, &parsed_message).await?;
        } else {
            // 处理普通消息
            manager.handle_message(&parsed_message).await?;
        }

        Ok(())
    }
}

/// 插件系统初始化函数
pub async fn init_plugin_system(onebot_api: Arc<OneBotApi>) -> PluginResult<Arc<PluginSystem>> {
    let system = Arc::new(PluginSystem::new(onebot_api));
    system.initialize().await?;
    Ok(system)
}
