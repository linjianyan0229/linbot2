use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::plugins::{PluginResult, OneBotApi};
use crate::plugins::message::ParsedMessage;
use crate::plugins::command::CommandMatch;

/// 插件信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// 插件名称
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件作者
    pub author: String,
    /// 插件描述
    pub description: String,
    /// 插件主页
    pub homepage: Option<String>,
    /// 插件依赖
    pub dependencies: Vec<String>,
    /// 支持的API版本
    pub api_version: String,
    /// 插件标签
    pub tags: Vec<String>,
    /// 最小系统版本要求
    pub min_system_version: Option<String>,
}

impl Default for PluginInfo {
    fn default() -> Self {
        Self {
            name: "Unknown Plugin".to_string(),
            version: "0.1.0".to_string(),
            author: "Unknown".to_string(),
            description: "No description".to_string(),
            homepage: None,
            dependencies: Vec::new(),
            api_version: "1.0.0".to_string(),
            tags: Vec::new(),
            min_system_version: None,
        }
    }
}

/// 插件上下文，提供插件运行时需要的资源和接口
#[derive(Clone)]
pub struct PluginContext {
    /// OneBot API接口
    #[allow(dead_code)]
    pub api: Arc<OneBotApi>,
    /// 插件配置
    pub config: HashMap<String, serde_json::Value>,
    /// 插件数据目录
    #[allow(dead_code)]
    pub data_dir: std::path::PathBuf,
    /// 日志记录器
    #[allow(dead_code)]
    pub logger: Arc<dyn PluginLogger + Send + Sync>,
}

impl PluginContext {
    pub fn new(
        api: Arc<OneBotApi>,
        config: HashMap<String, serde_json::Value>,
        data_dir: std::path::PathBuf,
        logger: Arc<dyn PluginLogger + Send + Sync>,
    ) -> Self {
        Self {
            api,
            config,
            data_dir,
            logger,
        }
    }

    /// 获取配置值
    #[allow(dead_code)]
    pub fn get_config<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.config.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// 设置配置值
    #[allow(dead_code)]
    pub fn set_config<T>(&mut self, key: &str, value: T) -> PluginResult<()>
    where
        T: Serialize,
    {
        let json_value = serde_json::to_value(value)?;
        self.config.insert(key.to_string(), json_value);
        Ok(())
    }
}

/// 插件日志记录器接口
#[async_trait]
#[allow(dead_code)]
pub trait PluginLogger {
    async fn debug(&self, plugin_name: &str, message: &str);
    async fn info(&self, plugin_name: &str, message: &str);
    async fn warn(&self, plugin_name: &str, message: &str);
    async fn error(&self, plugin_name: &str, message: &str);
}

/// 插件生命周期钩子
#[async_trait]
pub trait PluginLifecycle {
    /// 插件初始化
    async fn on_init(&self, _context: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    /// 插件启动
    async fn on_start(&self, _context: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    /// 插件停止
    async fn on_stop(&self, _context: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    /// 插件卸载
    async fn on_unload(&self, _context: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    /// 配置更新
    #[allow(dead_code)]
    async fn on_config_update(&self, _context: &PluginContext) -> PluginResult<()> {
        Ok(())
    }
}

/// 消息处理接口
#[async_trait]
#[allow(dead_code)]
pub trait MessageHandler {
    /// 处理普通消息
    async fn handle_message(
        &self,
        context: &PluginContext,
        message: &ParsedMessage,
    ) -> PluginResult<bool>;

    /// 处理群消息
    async fn handle_group_message(
        &self,
        context: &PluginContext,
        message: &ParsedMessage,
    ) -> PluginResult<bool> {
        self.handle_message(context, message).await
    }

    /// 处理私聊消息
    async fn handle_private_message(
        &self,
        context: &PluginContext,
        message: &ParsedMessage,
    ) -> PluginResult<bool> {
        self.handle_message(context, message).await
    }
}

/// 命令处理接口
#[async_trait]
#[allow(dead_code)]
pub trait CommandHandler {
    /// 处理命令
    async fn handle_command(
        &self,
        context: &PluginContext,
        command: &CommandMatch,
        message: &ParsedMessage,
    ) -> PluginResult<bool>;

    /// 获取命令帮助信息
    async fn get_command_help(&self, _command: &str) -> Option<String> {
        None
    }

    /// 获取支持的命令列表
    async fn get_supported_commands(&self) -> Vec<String> {
        Vec::new()
    }
}

/// 事件处理接口
#[async_trait]
#[allow(dead_code)]
pub trait EventHandler {
    /// 处理通知事件
    async fn handle_notice(
        &self,
        _context: &PluginContext,
        _notice: &serde_json::Value,
    ) -> PluginResult<bool> {
        Ok(false)
    }

    /// 处理请求事件
    async fn handle_request(
        &self,
        _context: &PluginContext,
        _request: &serde_json::Value,
    ) -> PluginResult<bool> {
        Ok(false)
    }

    /// 处理元事件
    async fn handle_meta_event(
        &self,
        _context: &PluginContext,
        _meta: &serde_json::Value,
    ) -> PluginResult<bool> {
        Ok(false)
    }
}

/// 插件主接口，所有插件都必须实现此trait
#[async_trait]
#[allow(dead_code)]
pub trait Plugin: PluginLifecycle + MessageHandler + CommandHandler + EventHandler + Send + Sync {
    /// 获取插件信息
    fn get_info(&self) -> PluginInfo;

    /// 获取插件优先级（数字越小优先级越高）
    fn get_priority(&self) -> i32 {
        100
    }

    /// 检查插件是否应该处理此消息
    async fn should_handle_message(&self, _message: &ParsedMessage) -> bool {
        true
    }

    /// 检查插件是否应该处理此命令
    async fn should_handle_command(&self, _command: &CommandMatch) -> bool {
        true
    }

    /// 获取插件状态信息
    async fn get_status(&self) -> HashMap<String, serde_json::Value> {
        HashMap::new()
    }

    /// 健康检查
    async fn health_check(&self) -> PluginResult<bool> {
        Ok(true)
    }
}

/// 插件工厂trait，用于创建插件实例
#[allow(dead_code)]
pub trait PluginFactory {
    /// 创建插件实例
    fn create_plugin(&self) -> Box<dyn Plugin>;
    
    /// 获取插件信息
    fn get_plugin_info(&self) -> PluginInfo;
}

/// 插件注册宏
#[macro_export]
macro_rules! register_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn create_plugin() -> *mut dyn $crate::plugins::Plugin {
            let plugin = <$plugin_type>::new();
            Box::into_raw(Box::new(plugin))
        }

        #[no_mangle]
        pub extern "C" fn get_plugin_info() -> $crate::plugins::PluginInfo {
            <$plugin_type>::get_info()
        }
    };
}

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// 插件文件路径
    pub file_path: std::path::PathBuf,
    /// 插件信息
    pub info: PluginInfo,
    /// 是否已加载
    pub loaded: bool,
    /// 是否已启用
    pub enabled: bool,
    /// 加载时间
    pub load_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 最后错误信息
    pub last_error: Option<String>,
}
