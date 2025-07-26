use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use tauri::Manager;

/// 服务器配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub access_token: Option<String>,
    pub enabled: bool,  // 是否启用（重启时会重置为false）
    pub auto_start: bool, // 是否自动启动
    pub created_at: i64,
    pub updated_at: i64,
}

impl ServerConfig {
    pub fn new(id: String, name: String, host: String, port: u16, access_token: Option<String>) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            name,
            host,
            port,
            access_token,
            enabled: false,
            auto_start: false,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: String,
    pub servers: HashMap<String, ServerConfig>,
    pub settings: AppSettings,
}

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub auto_start_servers: bool,
    pub log_level: String,
    pub max_connections_per_server: u32,
    // 日志相关设置
    pub show_heartbeat_logs: bool,    // 是否显示心跳包日志
    pub auto_scroll_logs: bool,       // 是否自动滚动日志
    pub max_log_entries: u32,         // 最大日志条目数
    pub log_buffer_size: u32,         // 日志缓冲区大小
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            servers: HashMap::new(),
            settings: AppSettings {
                auto_start_servers: false,
                log_level: "info".to_string(),
                max_connections_per_server: 10,
                show_heartbeat_logs: false,  // 默认不显示心跳包
                auto_scroll_logs: true,      // 默认自动滚动
                max_log_entries: 1000,       // 最大1000条日志
                log_buffer_size: 100,        // 缓冲区100条
            },
        }
    }
}

/// 配置管理器
pub struct ConfigManager {
    config_path: PathBuf,
    config: AppConfig,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // 获取应用数据目录
        let app_data_dir = app_handle
            .path()
            .app_config_dir()
            .map_err(|e| format!("获取应用目录失败: {}", e))?;
        
        // 确保配置目录存在
        fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
        
        let config_path = app_data_dir.join("config.json");
        
        let mut manager = Self {
            config_path,
            config: AppConfig::default(), // 初始化为默认配置
        };
        
        // 加载配置
        manager.load_config()?;
        
        Ok(manager)
    }
    
    /// 加载配置文件
    fn load_config(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.config_path.exists() {
            let config_str = fs::read_to_string(&self.config_path)?;
            
            // 尝试解析配置文件
            match serde_json::from_str::<AppConfig>(&config_str) {
                Ok(mut config) => {
                    // 重置所有服务器的enabled状态为false
                    for server in config.servers.values_mut() {
                        server.enabled = false;
                    }
                    self.config = config;
                }
                Err(e) => {
                    println!("解析配置文件失败，尝试兼容性处理: {}", e);
                    
                    // 尝试解析为旧版本配置格式
                    match serde_json::from_str::<serde_json::Value>(&config_str) {
                        Ok(mut value) => {
                            // 添加缺失的设置字段
                            if let Some(settings) = value.get_mut("settings") {
                                if !settings.as_object().unwrap().contains_key("show_heartbeat_logs") {
                                    settings["show_heartbeat_logs"] = serde_json::Value::Bool(false);
                                }
                                if !settings.as_object().unwrap().contains_key("auto_scroll_logs") {
                                    settings["auto_scroll_logs"] = serde_json::Value::Bool(true);
                                }
                                if !settings.as_object().unwrap().contains_key("max_log_entries") {
                                    settings["max_log_entries"] = serde_json::Value::Number(serde_json::Number::from(1000));
                                }
                                if !settings.as_object().unwrap().contains_key("log_buffer_size") {
                                    settings["log_buffer_size"] = serde_json::Value::Number(serde_json::Number::from(100));
                                }
                            }
                            
                            // 重新尝试解析
                            match serde_json::from_value::<AppConfig>(value) {
                                Ok(mut config) => {
                                    // 重置所有服务器的enabled状态为false
                                    for server in config.servers.values_mut() {
                                        server.enabled = false;
                                    }
                                    self.config = config;
                                    
                                    // 保存更新后的配置
                                    self.save_config()?;
                                    println!("配置文件已升级到新版本");
                                }
                                Err(e2) => {
                                    println!("配置文件兼容性处理失败: {}", e2);
                                    println!("使用默认配置");
                                    self.config = AppConfig::default();
                                    self.save_config()?;
                                }
                            }
                        }
                        Err(e2) => {
                            println!("配置文件格式无效: {}, 使用默认配置", e2);
                            self.config = AppConfig::default();
                            self.save_config()?;
                        }
                    }
                }
            }
        } else {
            self.config = AppConfig::default();
        }
        
        Ok(())
    }
    
    /// 保存配置文件
    fn save_config(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::to_string_pretty(&self.config)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        
        fs::write(&self.config_path, content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;
        
        Ok(())
    }
    
    /// 获取所有服务器配置
    pub fn get_servers(&self) -> Vec<ServerConfig> {
        self.config.servers.values().cloned().collect()
    }
    
    /// 添加服务器配置
    pub fn add_server(&mut self, server: ServerConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.config.servers.insert(server.id.clone(), server);
        self.save_config()?;
        Ok(())
    }
    
    /// 更新服务器配置
    pub fn update_server(&mut self, server: ServerConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(existing) = self.config.servers.get_mut(&server.id) {
            *existing = server;
            existing.updated_at = chrono::Utc::now().timestamp();
            self.save_config()?;
        }
        Ok(())
    }
    
    /// 删除服务器配置
    pub fn remove_server(&mut self, server_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.config.servers.remove(server_id);
        self.save_config()?;
        Ok(())
    }
    
    /// 设置服务器启用状态
    pub fn set_server_enabled(&mut self, server_id: &str, enabled: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(server) = self.config.servers.get_mut(server_id) {
            server.enabled = enabled;
            server.updated_at = chrono::Utc::now().timestamp();
            self.save_config()?;
        }
        Ok(())
    }
    
    /// 获取指定服务器配置
    #[allow(dead_code)]
    pub fn get_server(&self, server_id: &str) -> Option<&ServerConfig> {
        self.config.servers.get(server_id)
    }
    
    /// 获取配置文件路径
    pub fn get_config_path(&self) -> PathBuf {
        self.config_path.clone()
    }
    
    /// 获取应用设置
    #[allow(dead_code)]
    pub fn get_settings(&self) -> &AppSettings {
        &self.config.settings
    }
    
    /// 更新应用设置
    #[allow(dead_code)]
    pub fn update_settings(&mut self, settings: AppSettings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.config.settings = settings;
        self.save_config()
    }
}

/// 日志条目类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: i64,
    pub level: LogLevel,
    pub category: String,  // "message", "heartbeat", "lifecycle", "notice", "request"
    pub content: String,
    pub raw_data: Option<serde_json::Value>,
    // 消息特定字段
    pub message_type: Option<String>,  // "group", "private"
    pub group_id: Option<i64>,
    pub user_id: Option<i64>,
    pub sender_name: Option<String>,
}

impl LogEntry {
    pub fn new(
        level: LogLevel,
        category: String,
        content: String,
        raw_data: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            level,
            category,
            content,
            raw_data,
            message_type: None,
            group_id: None,
            user_id: None,
            sender_name: None,
        }
    }

    pub fn with_message_info(
        mut self,
        message_type: Option<String>,
        group_id: Option<i64>,
        user_id: Option<i64>,
        sender_name: Option<String>,
    ) -> Self {
        self.message_type = message_type;
        self.group_id = group_id;
        self.user_id = user_id;
        self.sender_name = sender_name;
        self
    }
} 