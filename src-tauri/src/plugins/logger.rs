use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::plugins::plugin_trait::PluginLogger;

/// 日志级别
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl ToString for LogLevel {
    fn to_string(&self) -> String {
        match self {
            LogLevel::Debug => "DEBUG".to_string(),
            LogLevel::Info => "INFO".to_string(),
            LogLevel::Warn => "WARN".to_string(),
            LogLevel::Error => "ERROR".to_string(),
        }
    }
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 日志级别
    pub level: LogLevel,
    /// 插件名称
    pub plugin_name: String,
    /// 日志消息
    pub message: String,
    /// 额外的上下文信息
    pub context: HashMap<String, String>,
}

impl LogEntry {
    pub fn new(level: LogLevel, plugin_name: &str, message: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            plugin_name: plugin_name.to_string(),
            message: message.to_string(),
            context: HashMap::new(),
        }
    }

    /// 添加上下文信息
    #[allow(dead_code)]
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }

    /// 格式化为字符串
    pub fn format(&self) -> String {
        let context_str = if self.context.is_empty() {
            String::new()
        } else {
            let ctx: Vec<String> = self.context.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!(" [{}]", ctx.join(", "))
        };

        format!(
            "{} [{}] [{}] {}{}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            self.level.to_string(),
            self.plugin_name,
            self.message,
            context_str
        )
    }
}

/// 日志输出目标
#[async_trait]
#[allow(dead_code)]
pub trait LogOutput: Send + Sync {
    async fn write_log(&self, entry: &LogEntry);
    async fn flush(&self);
}

/// 控制台日志输出
#[allow(dead_code)]
pub struct ConsoleOutput;

#[async_trait]
impl LogOutput for ConsoleOutput {
    async fn write_log(&self, entry: &LogEntry) {
        println!("{}", entry.format());
    }

    async fn flush(&self) {
        // 控制台输出不需要刷新
    }
}

/// 文件日志输出
#[allow(dead_code)]
pub struct FileOutput {
    file_path: PathBuf,
    buffer: Arc<RwLock<Vec<LogEntry>>>,
    max_buffer_size: usize,
}

impl FileOutput {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            buffer: Arc::new(RwLock::new(Vec::new())),
            max_buffer_size: 100,
        }
    }

    /// 设置缓冲区大小
    #[allow(dead_code)]
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.max_buffer_size = size;
        self
    }

    /// 刷新缓冲区到文件
    async fn flush_buffer(&self) {
        let mut buffer = self.buffer.write().await;
        if buffer.is_empty() {
            return;
        }

        // 确保目录存在
        if let Some(parent) = self.file_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        // 写入文件
        let content: String = buffer.iter()
            .map(|entry| format!("{}\n", entry.format()))
            .collect();

        if let Err(e) = tokio::fs::write(&self.file_path, content).await {
            eprintln!("写入日志文件失败: {}", e);
        }

        buffer.clear();
    }
}

#[async_trait]
impl LogOutput for FileOutput {
    async fn write_log(&self, entry: &LogEntry) {
        let mut buffer = self.buffer.write().await;
        buffer.push(entry.clone());

        // 如果缓冲区满了，自动刷新
        if buffer.len() >= self.max_buffer_size {
            drop(buffer); // 释放锁
            self.flush_buffer().await;
        }
    }

    async fn flush(&self) {
        self.flush_buffer().await;
    }
}

/// 默认插件日志记录器
#[allow(dead_code)]
pub struct DefaultPluginLogger {
    outputs: Vec<Arc<dyn LogOutput>>,
    min_level: LogLevel,
}

impl DefaultPluginLogger {
    pub fn new() -> Self {
        Self {
            outputs: vec![Arc::new(ConsoleOutput)],
            min_level: LogLevel::Info,
        }
    }

    /// 添加日志输出目标
    pub fn add_output(mut self, output: Arc<dyn LogOutput>) -> Self {
        self.outputs.push(output);
        self
    }

    /// 设置最小日志级别
    pub fn with_min_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    /// 检查日志级别是否应该输出
    fn should_log(&self, level: &LogLevel) -> bool {
        match (&self.min_level, level) {
            (LogLevel::Debug, _) => true,
            (LogLevel::Info, LogLevel::Debug) => false,
            (LogLevel::Info, _) => true,
            (LogLevel::Warn, LogLevel::Debug | LogLevel::Info) => false,
            (LogLevel::Warn, _) => true,
            (LogLevel::Error, LogLevel::Error) => true,
            (LogLevel::Error, _) => false,
        }
    }

    /// 记录日志
    async fn log(&self, level: LogLevel, plugin_name: &str, message: &str) {
        if !self.should_log(&level) {
            return;
        }

        let entry = LogEntry::new(level, plugin_name, message);

        for output in &self.outputs {
            output.write_log(&entry).await;
        }
    }
}

#[async_trait]
impl PluginLogger for DefaultPluginLogger {
    async fn debug(&self, plugin_name: &str, message: &str) {
        self.log(LogLevel::Debug, plugin_name, message).await;
    }

    async fn info(&self, plugin_name: &str, message: &str) {
        self.log(LogLevel::Info, plugin_name, message).await;
    }

    async fn warn(&self, plugin_name: &str, message: &str) {
        self.log(LogLevel::Warn, plugin_name, message).await;
    }

    async fn error(&self, plugin_name: &str, message: &str) {
        self.log(LogLevel::Error, plugin_name, message).await;
    }
}

/// 插件日志管理器
#[allow(dead_code)]
pub struct PluginLogManager {
    /// 每个插件的日志记录器
    plugin_loggers: HashMap<String, Arc<dyn PluginLogger + Send + Sync>>,
    /// 全局日志记录器
    global_logger: Arc<dyn PluginLogger + Send + Sync>,
    /// 日志历史
    log_history: Arc<RwLock<Vec<LogEntry>>>,
    /// 最大历史记录数
    max_history: usize,
}

impl PluginLogManager {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let global_logger = Arc::new(DefaultPluginLogger::new());
        
        Self {
            plugin_loggers: HashMap::new(),
            global_logger,
            log_history: Arc::new(RwLock::new(Vec::new())),
            max_history: 1000,
        }
    }

    /// 设置最大历史记录数
    #[allow(dead_code)]
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// 为插件创建专用日志记录器
    #[allow(dead_code)]
    pub fn create_plugin_logger(&mut self, plugin_name: &str) -> Arc<dyn PluginLogger + Send + Sync> {
        // 创建插件专用的文件输出
        let log_file = PathBuf::from("logs")
            .join("plugins")
            .join(format!("{}.log", plugin_name));
        
        let file_output = Arc::new(FileOutput::new(log_file));
        
        let logger = Arc::new(
            DefaultPluginLogger::new()
                .add_output(file_output)
                .with_min_level(LogLevel::Debug)
        );

        self.plugin_loggers.insert(plugin_name.to_string(), logger.clone());
        logger
    }

    /// 获取插件的日志记录器
    #[allow(dead_code)]
    pub fn get_plugin_logger(&self, plugin_name: &str) -> Arc<dyn PluginLogger + Send + Sync> {
        self.plugin_loggers.get(plugin_name)
            .cloned()
            .unwrap_or_else(|| self.global_logger.clone())
    }

    /// 记录日志到历史
    #[allow(dead_code)]
    pub async fn record_log(&self, entry: LogEntry) {
        let mut history = self.log_history.write().await;
        history.push(entry);

        // 保持历史记录在限制范围内
        if history.len() > self.max_history {
            history.remove(0);
        }
    }

    /// 获取日志历史
    #[allow(dead_code)]
    pub async fn get_log_history(&self, plugin_name: Option<&str>, level: Option<LogLevel>) -> Vec<LogEntry> {
        let history = self.log_history.read().await;
        
        history.iter()
            .filter(|entry| {
                if let Some(name) = plugin_name {
                    if entry.plugin_name != name {
                        return false;
                    }
                }
                
                if let Some(ref filter_level) = level {
                    if &entry.level != filter_level {
                        return false;
                    }
                }
                
                true
            })
            .cloned()
            .collect()
    }

    /// 清除日志历史
    #[allow(dead_code)]
    pub async fn clear_history(&self, plugin_name: Option<&str>) {
        let mut history = self.log_history.write().await;
        
        if let Some(name) = plugin_name {
            history.retain(|entry| entry.plugin_name != name);
        } else {
            history.clear();
        }
    }

    /// 刷新所有日志输出
    #[allow(dead_code)]
    pub async fn flush_all(&self) {
        // 这里需要访问所有输出并刷新
        // 由于架构限制，暂时留空
        // 在实际实现中，可能需要重新设计以支持全局刷新
    }

    /// 获取插件日志统计
    #[allow(dead_code)]
    pub async fn get_plugin_log_stats(&self, plugin_name: &str) -> PluginLogStats {
        let history = self.log_history.read().await;
        
        let plugin_logs: Vec<_> = history.iter()
            .filter(|entry| entry.plugin_name == plugin_name)
            .collect();

        let mut stats = PluginLogStats::default();
        stats.total_logs = plugin_logs.len();

        for entry in plugin_logs {
            match entry.level {
                LogLevel::Debug => stats.debug_count += 1,
                LogLevel::Info => stats.info_count += 1,
                LogLevel::Warn => stats.warn_count += 1,
                LogLevel::Error => stats.error_count += 1,
            }
        }

        stats
    }
}

/// 插件日志统计
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PluginLogStats {
    pub total_logs: usize,
    pub debug_count: usize,
    pub info_count: usize,
    pub warn_count: usize,
    pub error_count: usize,
}

/// 日志过滤器
#[allow(dead_code)]
pub struct LogFilter {
    pub plugin_name: Option<String>,
    pub level: Option<LogLevel>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub message_contains: Option<String>,
}

impl LogFilter {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            plugin_name: None,
            level: None,
            start_time: None,
            end_time: None,
            message_contains: None,
        }
    }

    /// 检查日志条目是否匹配过滤器
    #[allow(dead_code)]
    pub fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(ref name) = self.plugin_name {
            if entry.plugin_name != *name {
                return false;
            }
        }

        if let Some(ref level) = self.level {
            if entry.level != *level {
                return false;
            }
        }

        if let Some(start) = self.start_time {
            if entry.timestamp < start {
                return false;
            }
        }

        if let Some(end) = self.end_time {
            if entry.timestamp > end {
                return false;
            }
        }

        if let Some(ref contains) = self.message_contains {
            if !entry.message.contains(contains) {
                return false;
            }
        }

        true
    }
}
