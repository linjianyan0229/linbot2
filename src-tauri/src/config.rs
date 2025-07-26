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
        
        // 加载或创建配置
        let config = if config_path.exists() {
            Self::load_config(&config_path)?
        } else {
            let default_config = AppConfig::default();
            Self::save_config(&config_path, &default_config)?;
            default_config
        };
        
        Ok(Self {
            config_path,
            config,
        })
    }
    
    /// 加载配置文件
    fn load_config(path: &PathBuf) -> Result<AppConfig, Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;
        
        let mut config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?;
        
        // 重启时重置所有服务器的enabled状态为false
        for server in config.servers.values_mut() {
            server.enabled = false;
        }
        
        Ok(config)
    }
    
    /// 保存配置文件
    fn save_config(path: &PathBuf, config: &AppConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        
        fs::write(path, content)
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
        self.save()?;
        Ok(())
    }
    
    /// 更新服务器配置
    pub fn update_server(&mut self, server: ServerConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(existing) = self.config.servers.get_mut(&server.id) {
            *existing = server;
            existing.updated_at = chrono::Utc::now().timestamp();
            self.save()?;
        }
        Ok(())
    }
    
    /// 删除服务器配置
    pub fn remove_server(&mut self, server_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.config.servers.remove(server_id);
        self.save()?;
        Ok(())
    }
    
    /// 设置服务器启用状态
    pub fn set_server_enabled(&mut self, server_id: &str, enabled: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(server) = self.config.servers.get_mut(server_id) {
            server.enabled = enabled;
            server.updated_at = chrono::Utc::now().timestamp();
            self.save()?;
        }
        Ok(())
    }
    
    /// 获取指定服务器配置
    #[allow(dead_code)]
    pub fn get_server(&self, server_id: &str) -> Option<&ServerConfig> {
        self.config.servers.get(server_id)
    }
    
    /// 保存配置
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Self::save_config(&self.config_path, &self.config)
    }
    
    /// 获取配置文件路径
    pub fn get_config_path(&self) -> &PathBuf {
        &self.config_path
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
        self.save()?;
        Ok(())
    }
} 