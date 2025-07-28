use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use tokio::fs;

use crate::plugins::{PluginResult, PluginError};

/// 全局插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPluginConfig {
    /// 命令前缀
    pub command_prefix: String,
    /// 插件目录
    pub plugins_dir: String,
    /// 是否启用插件系统
    pub enabled: bool,
    /// 最大同时运行的插件数量
    pub max_plugins: usize,
    /// 插件超时时间（秒）
    pub plugin_timeout: u64,
    /// 是否允许热重载
    pub hot_reload: bool,
    /// 日志级别
    pub log_level: String,
    /// 安全设置
    pub security: SecurityConfig,
    /// 性能设置
    pub performance: PerformanceConfig,
}

impl Default for GlobalPluginConfig {
    fn default() -> Self {
        Self {
            command_prefix: "/".to_string(),
            plugins_dir: "plugins".to_string(),
            enabled: true,
            max_plugins: 50,
            plugin_timeout: 30,
            hot_reload: true,
            log_level: "info".to_string(),
            security: SecurityConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl GlobalPluginConfig {
    /// 加载配置文件，如果不存在则创建默认配置
    pub async fn load_or_default() -> PluginResult<Self> {
        let config_path = Self::get_config_path();
        
        if config_path.exists() {
            Self::load_from_file(&config_path).await
        } else {
            let default_config = Self::default();
            default_config.save_to_file(&config_path).await?;
            Ok(default_config)
        }
    }

    /// 从文件加载配置
    pub async fn load_from_file(path: &Path) -> PluginResult<Self> {
        let content = fs::read_to_string(path).await?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| PluginError::ConfigError(format!("解析配置文件失败: {}", e)))?;
        Ok(config)
    }

    /// 保存配置到文件
    pub async fn save_to_file(&self, path: &Path) -> PluginResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| PluginError::ConfigError(format!("序列化配置失败: {}", e)))?;
        
        fs::write(path, content).await?;
        Ok(())
    }

    /// 获取配置文件路径
    pub fn get_config_path() -> PathBuf {
        PathBuf::from("config").join("plugins.toml")
    }

    /// 保存当前配置
    pub async fn save(&self) -> PluginResult<()> {
        let config_path = Self::get_config_path();
        self.save_to_file(&config_path).await
    }

    /// 验证配置
    #[allow(dead_code)]
    pub fn validate(&self) -> PluginResult<()> {
        if self.command_prefix.is_empty() {
            return Err(PluginError::ConfigError("命令前缀不能为空".to_string()));
        }

        if self.max_plugins == 0 {
            return Err(PluginError::ConfigError("最大插件数量必须大于0".to_string()));
        }

        if self.plugin_timeout == 0 {
            return Err(PluginError::ConfigError("插件超时时间必须大于0".to_string()));
        }

        self.security.validate()?;
        self.performance.validate()?;

        Ok(())
    }
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 是否启用沙箱
    pub enable_sandbox: bool,
    /// 是否验证插件签名
    pub verify_signatures: bool,
    /// 允许的文件系统访问路径
    pub allowed_paths: Vec<String>,
    /// 禁止的文件系统访问路径
    pub denied_paths: Vec<String>,
    /// 是否允许网络访问
    pub allow_network: bool,
    /// 允许的网络域名
    pub allowed_domains: Vec<String>,
    /// 最大内存使用量（MB）
    pub max_memory_mb: usize,
    /// 最大CPU使用率（百分比）
    pub max_cpu_percent: f32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_sandbox: true,
            verify_signatures: false,
            allowed_paths: vec![
                "plugins/".to_string(),
                "data/".to_string(),
                "temp/".to_string(),
            ],
            denied_paths: vec![
                "config/".to_string(),
                "system/".to_string(),
            ],
            allow_network: true,
            allowed_domains: Vec::new(),
            max_memory_mb: 256,
            max_cpu_percent: 50.0,
        }
    }
}

impl SecurityConfig {
    #[allow(dead_code)]
    pub fn validate(&self) -> PluginResult<()> {
        if self.max_memory_mb == 0 {
            return Err(PluginError::ConfigError("最大内存使用量必须大于0".to_string()));
        }

        if self.max_cpu_percent <= 0.0 || self.max_cpu_percent > 100.0 {
            return Err(PluginError::ConfigError("CPU使用率必须在0-100之间".to_string()));
        }

        Ok(())
    }
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 工作线程数量
    pub worker_threads: usize,
    /// 消息队列大小
    pub message_queue_size: usize,
    /// 插件启动超时时间（秒）
    pub startup_timeout: u64,
    /// 插件停止超时时间（秒）
    pub shutdown_timeout: u64,
    /// 是否启用性能监控
    pub enable_monitoring: bool,
    /// 性能统计间隔（秒）
    pub stats_interval: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            message_queue_size: 1000,
            startup_timeout: 30,
            shutdown_timeout: 10,
            enable_monitoring: true,
            stats_interval: 60,
        }
    }
}

impl PerformanceConfig {
    #[allow(dead_code)]
    pub fn validate(&self) -> PluginResult<()> {
        if self.worker_threads == 0 {
            return Err(PluginError::ConfigError("工作线程数量必须大于0".to_string()));
        }

        if self.message_queue_size == 0 {
            return Err(PluginError::ConfigError("消息队列大小必须大于0".to_string()));
        }

        Ok(())
    }
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// 插件名称
    pub name: String,
    /// 是否启用
    pub enabled: bool,
    /// 插件设置
    pub settings: HashMap<String, serde_json::Value>,
    /// 插件权限
    pub permissions: PluginPermissions,
    /// 插件资源限制
    pub limits: PluginLimits,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            enabled: true,
            settings: HashMap::new(),
            permissions: PluginPermissions::default(),
            limits: PluginLimits::default(),
        }
    }
}

impl PluginConfig {
    /// 为指定插件加载配置
    pub async fn load_for_plugin(plugin_name: &str) -> PluginResult<Self> {
        let config_path = Self::get_plugin_config_path(plugin_name);
        
        if config_path.exists() {
            Self::load_from_file(&config_path).await
        } else {
            let mut default_config = Self::default();
            default_config.name = plugin_name.to_string();
            default_config.save_to_file(&config_path).await?;
            Ok(default_config)
        }
    }

    /// 从文件加载配置
    pub async fn load_from_file(path: &Path) -> PluginResult<Self> {
        let content = fs::read_to_string(path).await?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| PluginError::ConfigError(format!("解析插件配置失败: {}", e)))?;
        Ok(config)
    }

    /// 保存配置到文件
    pub async fn save_to_file(&self, path: &Path) -> PluginResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| PluginError::ConfigError(format!("序列化插件配置失败: {}", e)))?;
        
        fs::write(path, content).await?;
        Ok(())
    }

    /// 获取插件配置文件路径
    pub fn get_plugin_config_path(plugin_name: &str) -> PathBuf {
        PathBuf::from("plugins").join(plugin_name).join("config.toml")
    }

    /// 保存当前配置
    pub async fn save(&self) -> PluginResult<()> {
        let config_path = Self::get_plugin_config_path(&self.name);
        self.save_to_file(&config_path).await
    }

    /// 获取设置值
    #[allow(dead_code)]
    pub fn get_setting<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        self.settings.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// 设置配置值
    #[allow(dead_code)]
    pub fn set_setting<T>(&mut self, key: &str, value: T) -> PluginResult<()>
    where
        T: serde::Serialize,
    {
        let json_value = serde_json::to_value(value)?;
        self.settings.insert(key.to_string(), json_value);
        Ok(())
    }

    /// 移除设置
    #[allow(dead_code)]
    pub fn remove_setting(&mut self, key: &str) -> Option<serde_json::Value> {
        self.settings.remove(key)
    }

    /// 检查是否有指定设置
    #[allow(dead_code)]
    pub fn has_setting(&self, key: &str) -> bool {
        self.settings.contains_key(key)
    }
}

/// 插件权限
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PluginPermissions {
    /// 是否允许发送消息
    pub send_messages: bool,
    /// 是否允许读取消息
    pub read_messages: bool,
    /// 是否允许管理群组
    pub manage_groups: bool,
    /// 是否允许文件系统访问
    pub file_system: bool,
    /// 是否允许网络访问
    pub network: bool,
    /// 是否允许执行系统命令
    pub system_commands: bool,
    /// 自定义权限
    pub custom: HashMap<String, bool>,
}

impl Default for PluginPermissions {
    fn default() -> Self {
        Self {
            send_messages: true,
            read_messages: true,
            manage_groups: false,
            file_system: true,
            network: true,
            system_commands: false,
            custom: HashMap::new(),
        }
    }
}

/// 插件资源限制
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PluginLimits {
    /// 最大内存使用量（MB）
    pub max_memory_mb: usize,
    /// 最大CPU使用率（百分比）
    pub max_cpu_percent: f32,
    /// 最大文件大小（MB）
    pub max_file_size_mb: usize,
    /// 最大网络带宽（KB/s）
    pub max_bandwidth_kbps: usize,
    /// 最大运行时间（秒，0表示无限制）
    pub max_runtime_seconds: u64,
}

impl Default for PluginLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 128,
            max_cpu_percent: 25.0,
            max_file_size_mb: 10,
            max_bandwidth_kbps: 1024,
            max_runtime_seconds: 0,
        }
    }
}

/// 配置管理器
#[allow(dead_code)]
pub struct ConfigManager {
    global_config: GlobalPluginConfig,
    plugin_configs: HashMap<String, PluginConfig>,
}

impl ConfigManager {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            global_config: GlobalPluginConfig::default(),
            plugin_configs: HashMap::new(),
        }
    }

    /// 初始化配置管理器
    #[allow(dead_code)]
    pub async fn initialize(&mut self) -> PluginResult<()> {
        self.global_config = GlobalPluginConfig::load_or_default().await?;
        self.global_config.validate()?;
        Ok(())
    }

    /// 获取全局配置
    #[allow(dead_code)]
    pub fn get_global_config(&self) -> &GlobalPluginConfig {
        &self.global_config
    }

    /// 更新全局配置
    #[allow(dead_code)]
    pub async fn update_global_config(&mut self, config: GlobalPluginConfig) -> PluginResult<()> {
        config.validate()?;
        config.save().await?;
        self.global_config = config;
        Ok(())
    }

    /// 获取插件配置
    #[allow(dead_code)]
    pub async fn get_plugin_config(&mut self, plugin_name: &str) -> PluginResult<&PluginConfig> {
        if !self.plugin_configs.contains_key(plugin_name) {
            let config = PluginConfig::load_for_plugin(plugin_name).await?;
            self.plugin_configs.insert(plugin_name.to_string(), config);
        }
        
        Ok(self.plugin_configs.get(plugin_name).unwrap())
    }

    /// 更新插件配置
    #[allow(dead_code)]
    pub async fn update_plugin_config(&mut self, config: PluginConfig) -> PluginResult<()> {
        config.save().await?;
        self.plugin_configs.insert(config.name.clone(), config);
        Ok(())
    }

    /// 移除插件配置
    #[allow(dead_code)]
    pub fn remove_plugin_config(&mut self, plugin_name: &str) {
        self.plugin_configs.remove(plugin_name);
    }
}
