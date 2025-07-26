mod onebot;
mod websocket_server;
mod config;

use std::sync::Arc;
use std::collections::{VecDeque, HashMap};
use tokio::sync::{Mutex, mpsc};
use tauri::Emitter;
use serde::{Serialize, Deserialize};

use config::{ConfigManager, ServerConfig};
use onebot::{OneBotConfig, OneBotEvent, ConnectionStatus, BotAccount, Friend, Group, OneBotApiRequest, OneBotApiResponse, BotLoginInfo, SendMessageResponse};
use websocket_server::OneBotServer;

use crate::onebot::format_event_log;
use crate::config::{AppSettings, LogEntry, LogLevel};
use once_cell::sync::Lazy;

// 全局服务器实例
static SERVER: once_cell::sync::Lazy<Arc<Mutex<Option<Arc<OneBotServer>>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

// 配置管理器
static CONFIG_MANAGER: once_cell::sync::Lazy<Arc<Mutex<Option<ConfigManager>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

// 服务器运行状态 (is_running, status_string, connection_count)
static SERVER_STATUS: once_cell::sync::Lazy<Arc<Mutex<(bool, String, u32)>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new((false, "disconnected".to_string(), 0))));

// 日志管理相关的全局状态
static LOG_BUFFER: Lazy<Arc<Mutex<VecDeque<LogEntry>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(VecDeque::new()))
});

static LOG_SUBSCRIBERS: Lazy<Arc<Mutex<Vec<mpsc::UnboundedSender<LogEntry>>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(Vec::new()))
});

// 机器人账号缓存
static BOT_ACCOUNTS: Lazy<Arc<Mutex<HashMap<i64, BotAccount>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

// API 调用缓存时间（秒）
const CACHE_DURATION: i64 = 300; // 5分钟

// API 响应等待映射
static API_RESPONSE_MAP: Lazy<Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<OneBotApiResponse>>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// 启动 OneBot 反向 WebSocket 服务器
#[tauri::command]
async fn start_onebot_server(
    host: String,
    port: u16,
    access_token: Option<String>,
) -> Result<String, String> {
    // 设置状态为连接中
    {
        let mut status = SERVER_STATUS.lock().await;
        *status = (true, "connecting".to_string(), 0);
    }

    let config = OneBotConfig {
        host: host.clone(),
        port,
        access_token: access_token.clone(),
        secret: None,
    };

    let server = OneBotServer::new(config);

    // 设置事件回调
    server.set_event_callback(handle_onebot_event).await;

    // 将服务器实例包装在 Arc 中并保存到全局变量
    let server_arc = Arc::new(server);
    {
        let mut server_guard = SERVER.lock().await;
        *server_guard = Some(Arc::clone(&server_arc));
    }

    // 在后台任务中启动服务器
    let server_for_task = Arc::clone(&server_arc);
    tokio::spawn(async move {
        match server_for_task.start().await {
            Ok(_) => {
                println!("OneBot 服务器启动成功");
                // 服务器启动成功，但连接状态由事件回调来设置
                // 如果30秒内没有收到连接事件，则设置为已启动但无连接
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                let mut status = SERVER_STATUS.lock().await;
                if status.1 == "connecting" {
                    *status = (true, "connected".to_string(), 0);
                }
            }
            Err(e) => {
                eprintln!("OneBot 服务器启动失败: {}", e);
                // 设置状态为未连接
                let mut status = SERVER_STATUS.lock().await;
                *status = (false, "disconnected".to_string(), 0);
            }
        }
    });

    // 等待一小段时间让服务器启动
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    Ok(format!("OneBot 服务器已启动在 {}:{}", host, port))
}

/// 获取 OneBot 服务器状态
#[tauri::command]
async fn get_onebot_status() -> Result<String, String> {
    let server_guard = SERVER.lock().await;
    if let Some(ref server) = *server_guard {
        let status = server.get_status().await;
        let count = server.get_connection_count().await;

        let status_str = match status {
            ConnectionStatus::Connected => "已连接",
            ConnectionStatus::Connecting => "连接中",
            ConnectionStatus::Disconnected => "未连接",
        };

        Ok(format!("状态: {} | 连接数: {}", status_str, count))
    } else {
        Ok("服务器未启动".to_string())
    }
}

/// 检查服务器是否真正在运行并返回详细状态
#[tauri::command]
async fn get_server_runtime_status() -> Result<(bool, String, u32), String> {
    let status = SERVER_STATUS.lock().await;
    let (is_running, status_str, connection_count) = status.clone();

    Ok((is_running, status_str, connection_count))
}

/// 停止 OneBot 服务器
#[tauri::command]
async fn stop_onebot_server() -> Result<String, String> {
    // 首先尝试优雅地停止服务器
    {
        let server_guard = SERVER.lock().await;
        if let Some(ref server) = *server_guard {
            if let Err(e) = server.shutdown().await {
                eprintln!("停止服务器时出错: {}", e);
            }
        }
    }

    // 设置状态为未连接
    {
        let mut status = SERVER_STATUS.lock().await;
        *status = (false, "disconnected".to_string(), 0);
    }

    // 清除服务器实例
    {
        let mut server_guard = SERVER.lock().await;
        *server_guard = None;
    }

    println!("OneBot 服务器已停止");
    Ok("OneBot 服务器已停止".to_string())
}

/// 初始化配置管理器
#[tauri::command]
async fn init_config_manager(app_handle: tauri::AppHandle) -> Result<String, String> {
    let manager = ConfigManager::new(&app_handle)
        .map_err(|e| format!("初始化配置管理器失败: {}", e))?;

    let config_path = manager.get_config_path().display().to_string();

    {
        let mut config_guard = CONFIG_MANAGER.lock().await;
        *config_guard = Some(manager);
    }

    println!("配置管理器已初始化，配置文件路径: {}", config_path);
    Ok(config_path)
}

/// 获取所有服务器配置
#[tauri::command]
async fn get_all_servers() -> Result<Vec<ServerConfig>, String> {
    let config_guard = CONFIG_MANAGER.lock().await;
    if let Some(ref manager) = *config_guard {
        Ok(manager.get_servers())
    } else {
        Err("配置管理器未初始化".to_string())
    }
}

/// 添加服务器配置
#[tauri::command]
async fn add_server_config(
    name: String,
    host: String,
    port: u16,
    access_token: Option<String>,
) -> Result<ServerConfig, String> {
    let server_id = uuid::Uuid::new_v4().to_string();
    let server = ServerConfig::new(server_id, name, host, port, access_token);

    {
        let mut config_guard = CONFIG_MANAGER.lock().await;
        if let Some(ref mut manager) = *config_guard {
            manager.add_server(server.clone())
                .map_err(|e| format!("添加服务器配置失败: {}", e))?;
        } else {
            return Err("配置管理器未初始化".to_string());
        }
    }

    println!("已添加服务器配置: {} ({}:{})", server.name, server.host, server.port);
    Ok(server)
}

/// 更新服务器配置
#[tauri::command]
async fn update_server_config(server: ServerConfig) -> Result<(), String> {
    let mut config_guard = CONFIG_MANAGER.lock().await;
    if let Some(ref mut manager) = *config_guard {
        manager.update_server(server)
            .map_err(|e| format!("更新服务器配置失败: {}", e))?;
        Ok(())
    } else {
        Err("配置管理器未初始化".to_string())
    }
}

/// 删除服务器配置
#[tauri::command]
async fn remove_server_config(server_id: String) -> Result<(), String> {
    let mut config_guard = CONFIG_MANAGER.lock().await;
    if let Some(ref mut manager) = *config_guard {
        manager.remove_server(&server_id)
            .map_err(|e| format!("删除服务器配置失败: {}", e))?;
        println!("已删除服务器配置: {}", server_id);
        Ok(())
    } else {
        Err("配置管理器未初始化".to_string())
    }
}

/// 设置服务器启用状态
#[tauri::command]
async fn set_server_enabled(server_id: String, enabled: bool) -> Result<(), String> {
    let mut config_guard = CONFIG_MANAGER.lock().await;
    if let Some(ref mut manager) = *config_guard {
        manager.set_server_enabled(&server_id, enabled)
            .map_err(|e| format!("设置服务器状态失败: {}", e))?;
        println!("服务器 {} 状态已设置为: {}", server_id, if enabled { "启用" } else { "禁用" });
        Ok(())
    } else {
        Err("配置管理器未初始化".to_string())
    }
}

/// 获取配置文件路径
#[tauri::command]
async fn get_config_path() -> Result<String, String> {
    let config_guard = CONFIG_MANAGER.lock().await;
    if let Some(ref manager) = *config_guard {
        Ok(manager.get_config_path().display().to_string())
    } else {
        Err("配置管理器未初始化".to_string())
    }
}

/// 获取应用设置
#[tauri::command]
async fn get_app_settings() -> Result<AppSettings, String> {
    let config_guard = CONFIG_MANAGER.lock().await;
    if let Some(ref manager) = *config_guard {
        Ok(manager.get_settings().clone())
    } else {
        Err("配置管理器未初始化".to_string())
    }
}

/// 更新应用设置
#[tauri::command]
async fn update_app_settings(settings: AppSettings) -> Result<(), String> {
    let mut config_guard = CONFIG_MANAGER.lock().await;
    if let Some(ref mut manager) = *config_guard {
        manager.update_settings(settings).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("配置管理器未初始化".to_string())
    }
}

/// 获取日志历史
#[tauri::command]
async fn get_log_history() -> Result<Vec<LogEntry>, String> {
    let buffer = LOG_BUFFER.lock().await;
    Ok(buffer.iter().cloned().collect())
}

/// 清空日志历史
#[tauri::command]
async fn clear_log_history() -> Result<(), String> {
    let mut buffer = LOG_BUFFER.lock().await;
    buffer.clear();
    Ok(())
}

/// 订阅实时日志
#[tauri::command]
async fn subscribe_logs(window: tauri::Window) -> Result<(), String> {
    let (tx, mut rx) = mpsc::unbounded_channel::<LogEntry>();
    
    // 添加到订阅者列表
    {
        let mut subscribers = LOG_SUBSCRIBERS.lock().await;
        subscribers.push(tx);
    }
    
    // 启动发送任务
    tokio::spawn(async move {
        while let Some(log_entry) = rx.recv().await {
            if let Err(e) = window.emit("log-entry", &log_entry) {
                eprintln!("发送日志事件失败: {}", e);
                break;
            }
        }
    });
    
    Ok(())
}

/// 添加日志条目到缓冲区和推送给订阅者
async fn add_log_entry(entry: LogEntry) {
    // 添加到缓冲区
    {
        let mut buffer = LOG_BUFFER.lock().await;
        
        // 检查是否需要移除旧日志
        let config_guard = CONFIG_MANAGER.lock().await;
        let max_entries = if let Some(ref manager) = *config_guard {
            manager.get_settings().max_log_entries as usize
        } else {
            1000
        };
        
        while buffer.len() >= max_entries {
            buffer.pop_front();
        }
        
        buffer.push_back(entry.clone());
    }
    
    // 推送给所有订阅者
    {
        let mut subscribers = LOG_SUBSCRIBERS.lock().await;
        
        // 移除已关闭的订阅者
        subscribers.retain(|tx| !tx.is_closed());
        
        // 发送给所有活跃的订阅者
        for tx in subscribers.iter() {
            if let Err(e) = tx.send(entry.clone()) {
                eprintln!("发送日志给订阅者失败: {}", e);
            }
        }
    }
}

/// OneBot 事件处理函数
fn handle_onebot_event(event: OneBotEvent) {
    // 提取 self_id 用于跟踪机器人账号
    let self_id = match &event {
        OneBotEvent::Message { self_id, .. } => Some(*self_id),
        OneBotEvent::Notice { self_id, .. } => Some(*self_id),
        OneBotEvent::Request { self_id, .. } => Some(*self_id),
        OneBotEvent::MetaEvent { self_id, .. } => Some(*self_id),
    };

    // 更新机器人账号信息
    if let Some(bot_id) = self_id {
        tokio::spawn(async move {
            let mut accounts = BOT_ACCOUNTS.lock().await;
            let _current_time = chrono::Utc::now().timestamp();

            // 确保机器人账号存在于缓存中
            let account = accounts.entry(bot_id).or_insert_with(|| BotAccount {
                self_id: bot_id,
                nickname: format!("Bot {}", bot_id),
                status: "online".to_string(),
                friends: Vec::new(),
                groups: Vec::new(),
                last_updated: 0,
            });

            // 更新状态为在线
            account.status = "online".to_string();

            // 如果昵称还是默认的，尝试获取真实昵称
            if account.nickname.starts_with("Bot ") {
                let bot_id_for_task = bot_id;
                tokio::spawn(async move {
                    if let Ok(login_info) = get_bot_login_info(bot_id_for_task).await {
                        let mut accounts = BOT_ACCOUNTS.lock().await;
                        if let Some(account) = accounts.get_mut(&bot_id_for_task) {
                            account.nickname = login_info.nickname;
                        }
                    }
                });
            }
        });
    }

    // 先处理状态更新逻辑（避免move问题）
    let update_connection_status = match &event {
        OneBotEvent::Message { .. } => true,
        OneBotEvent::MetaEvent { meta_event_type, .. } => {
            if meta_event_type == "lifecycle" {
                println!("收到生命周期事件: {}", meta_event_type);
            }
            true
        }
        _ => false,
    };

    if update_connection_status {
        tokio::spawn(async {
            let mut status = SERVER_STATUS.lock().await;
            if status.0 { // 如果服务器标记为运行中
                *status = (true, "connected".to_string(), 1);
            }
        });
    }

    // 创建日志条目
    let log_entry = match &event {
        OneBotEvent::Message { 
            user_id, 
            message_type,
            group_id,
            sender,
            .. 
        } => {
            let sender_name = if let Some(card) = &sender.card {
                if card.is_empty() { &sender.nickname } else { card }
            } else {
                &sender.nickname
            };
            
            let log_content = format_event_log(&event);
            
            LogEntry::new(
                LogLevel::Info,
                "message".to_string(),
                log_content,
                Some(serde_json::to_value(&event).unwrap_or_default()),
            ).with_message_info(
                Some(message_type.clone()),
                *group_id,
                Some(*user_id),
                Some(sender_name.to_string()),
            )
        }
        OneBotEvent::Notice { user_id, .. } => {
            let log_content = format_event_log(&event);
            LogEntry::new(
                LogLevel::Info,
                "notice".to_string(),
                log_content,
                Some(serde_json::to_value(&event).unwrap_or_default()),
            ).with_message_info(None, None, Some(*user_id), None)
        }
        OneBotEvent::Request { user_id, .. } => {
            let log_content = format_event_log(&event);
            LogEntry::new(
                LogLevel::Info,
                "request".to_string(),
                log_content,
                Some(serde_json::to_value(&event).unwrap_or_default()),
            ).with_message_info(None, None, Some(*user_id), None)
        }
        OneBotEvent::MetaEvent { meta_event_type, .. } => {
            let log_content = format_event_log(&event);
            let level = match meta_event_type.as_str() {
                "heartbeat" => LogLevel::Debug,
                _ => LogLevel::Info,
            };
            LogEntry::new(
                level,
                meta_event_type.clone(),
                log_content,
                Some(serde_json::to_value(&event).unwrap_or_default()),
            )
        }
    };

    // 异步添加日志条目
    let should_show_heartbeat = matches!(&event, OneBotEvent::MetaEvent { meta_event_type, .. } if meta_event_type == "heartbeat");
    
    tokio::spawn(async move {
        // 检查是否应该显示心跳包日志
        let should_show = if should_show_heartbeat {
            let config_guard = CONFIG_MANAGER.lock().await;
            if let Some(ref manager) = *config_guard {
                manager.get_settings().show_heartbeat_logs
            } else {
                false
            }
        } else {
            true
        };

        if should_show {
            add_log_entry(log_entry).await;
        }
    });
}

/// 向 OneBot 客户端发送 API 请求
#[allow(dead_code)]
async fn send_onebot_api_request(
    action: &str,
    params: HashMap<String, serde_json::Value>,
) -> Result<OneBotApiResponse, String> {
    let server_guard = SERVER.lock().await;
    if let Some(ref server) = *server_guard {
        // 检查是否有连接
        let connections = server.get_connections().await;
        if connections.is_empty() {
            return Err("没有活跃的 OneBot 连接".to_string());
        }

        // 生成唯一的 echo ID
        let echo = uuid::Uuid::new_v4().to_string();

        // 构建 API 请求
        let request = OneBotApiRequest {
            action: action.to_string(),
            params,
            echo: Some(echo.clone()),
        };

        // 创建响应通道
        let (tx, rx) = tokio::sync::oneshot::channel();

        // 注册响应等待
        {
            let mut response_map = API_RESPONSE_MAP.lock().await;
            response_map.insert(echo.clone(), tx);
        }

        // 发送请求
        if let Err(e) = server.send_api_request(request).await {
            // 清理响应等待
            let mut response_map = API_RESPONSE_MAP.lock().await;
            response_map.remove(&echo);
            return Err(format!("发送 API 请求失败: {}", e));
        }

        // 等待响应（设置超时）
        match tokio::time::timeout(tokio::time::Duration::from_secs(10), rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => {
                // 清理响应等待
                let mut response_map = API_RESPONSE_MAP.lock().await;
                response_map.remove(&echo);
                Err("API 响应通道关闭".to_string())
            }
            Err(_) => {
                // 超时，清理响应等待
                let mut response_map = API_RESPONSE_MAP.lock().await;
                response_map.remove(&echo);
                Err("API 请求超时".to_string())
            }
        }
    } else {
        Err("服务器未启动".to_string())
    }
}

/// 获取好友列表（带缓存）
async fn get_friend_list_cached(self_id: i64) -> Result<Vec<Friend>, String> {
    let mut accounts = BOT_ACCOUNTS.lock().await;
    let current_time = chrono::Utc::now().timestamp();

    // 检查缓存是否有效
    if let Some(account) = accounts.get(&self_id) {
        if current_time - account.last_updated < CACHE_DURATION {
            return Ok(account.friends.clone());
        }
    }

    // 调用真实的 OneBot API
    let params = HashMap::new();
    let response = send_onebot_api_request("get_friend_list", params).await?;

    // 解析响应
    if response.status == "ok" && response.retcode == 0 {
        if let Some(data) = response.data {
            let friends: Vec<Friend> = serde_json::from_value(data)
                .map_err(|e| format!("解析好友列表失败: {}", e))?;

            // 更新缓存
            let account = accounts.entry(self_id).or_insert_with(|| BotAccount {
                self_id,
                nickname: format!("Bot {}", self_id),
                status: "online".to_string(),
                friends: Vec::new(),
                groups: Vec::new(),
                last_updated: current_time,
            });
            account.friends = friends.clone();
            account.last_updated = current_time;

            return Ok(friends);
        }
    }

    Err(format!("API 调用失败: {} ({})",
        response.message.unwrap_or_default(),
        response.retcode))
}

/// 获取群聊列表（带缓存）
async fn get_group_list_cached(self_id: i64) -> Result<Vec<Group>, String> {
    let mut accounts = BOT_ACCOUNTS.lock().await;
    let current_time = chrono::Utc::now().timestamp();

    // 检查缓存是否有效
    if let Some(account) = accounts.get(&self_id) {
        if current_time - account.last_updated < CACHE_DURATION {
            return Ok(account.groups.clone());
        }
    }

    // 调用真实的 OneBot API
    let params = HashMap::new();
    let response = send_onebot_api_request("get_group_list", params).await?;

    // 解析响应
    if response.status == "ok" && response.retcode == 0 {
        if let Some(data) = response.data {
            let groups: Vec<Group> = serde_json::from_value(data)
                .map_err(|e| format!("解析群聊列表失败: {}", e))?;

            // 更新缓存
            let account = accounts.entry(self_id).or_insert_with(|| BotAccount {
                self_id,
                nickname: format!("Bot {}", self_id),
                status: "online".to_string(),
                friends: Vec::new(),
                groups: Vec::new(),
                last_updated: current_time,
            });
            account.groups = groups.clone();
            account.last_updated = current_time;

            return Ok(groups);
        }
    }

    Err(format!("API 调用失败: {} ({})",
        response.message.unwrap_or_default(),
        response.retcode))
}

/// 获取机器人登录信息
async fn get_bot_login_info(_self_id: i64) -> Result<BotLoginInfo, String> {
    let params = HashMap::new();
    let response = send_onebot_api_request("get_login_info", params).await?;

    if response.status == "ok" && response.retcode == 0 {
        if let Some(data) = response.data {
            let login_info: BotLoginInfo = serde_json::from_value(data)
                .map_err(|e| format!("解析登录信息失败: {}", e))?;
            return Ok(login_info);
        }
    }

    Err(format!("获取登录信息失败: {} ({})",
        response.message.unwrap_or_default(),
        response.retcode))
}

/// 获取所有机器人账号
#[tauri::command]
async fn get_bot_accounts() -> Result<Vec<BotAccount>, String> {
    let accounts = BOT_ACCOUNTS.lock().await;
    Ok(accounts.values().cloned().collect())
}

/// 获取指定机器人的好友列表
#[tauri::command]
async fn get_friends(self_id: i64) -> Result<Vec<Friend>, String> {
    get_friend_list_cached(self_id).await
}

/// 获取指定机器人的群聊列表
#[tauri::command]
async fn get_groups(self_id: i64) -> Result<Vec<Group>, String> {
    get_group_list_cached(self_id).await
}

/// 刷新机器人数据（清除缓存）
#[tauri::command]
async fn refresh_bot_data(self_id: Option<i64>) -> Result<(), String> {
    let mut accounts = BOT_ACCOUNTS.lock().await;

    if let Some(id) = self_id {
        // 刷新指定机器人的数据
        if let Some(account) = accounts.get_mut(&id) {
            account.last_updated = 0; // 强制过期缓存
        }
    } else {
        // 刷新所有机器人的数据
        for account in accounts.values_mut() {
            account.last_updated = 0;
        }
    }

    Ok(())
}

/// 发送私聊消息
#[tauri::command]
#[allow(non_snake_case)]
async fn send_private_message(userId: i64, message: String) -> Result<SendMessageResponse, String> {
    let mut params = HashMap::new();
    params.insert("user_id".to_string(), serde_json::Value::Number(serde_json::Number::from(userId)));
    params.insert("message".to_string(), serde_json::Value::String(message));

    let response = send_onebot_api_request("send_private_msg", params).await?;

    if response.status == "ok" && response.retcode == 0 {
        if let Some(data) = response.data {
            let send_response: SendMessageResponse = serde_json::from_value(data)
                .map_err(|e| format!("解析发送响应失败: {}", e))?;
            return Ok(send_response);
        }
    }

    Err(format!("发送私聊消息失败: {} ({})",
        response.message.unwrap_or_default(),
        response.retcode))
}

/// 发送群聊消息
#[tauri::command]
#[allow(non_snake_case)]
async fn send_group_message(groupId: i64, message: String) -> Result<SendMessageResponse, String> {
    let mut params = HashMap::new();
    params.insert("group_id".to_string(), serde_json::Value::Number(serde_json::Number::from(groupId)));
    params.insert("message".to_string(), serde_json::Value::String(message));

    let response = send_onebot_api_request("send_group_msg", params).await?;

    if response.status == "ok" && response.retcode == 0 {
        if let Some(data) = response.data {
            let send_response: SendMessageResponse = serde_json::from_value(data)
                .map_err(|e| format!("解析发送响应失败: {}", e))?;
            return Ok(send_response);
        }
    }

    Err(format!("发送群聊消息失败: {} ({})",
        response.message.unwrap_or_default(),
        response.retcode))
}

/// 获取用户头像
#[tauri::command]
async fn get_user_avatar(user_id: i64) -> Result<String, String> {
    // OneBot 标准中没有直接的头像API，通常使用QQ头像链接
    Ok(format!("https://q1.qlogo.cn/g?b=qq&nk={}&s=640", user_id))
}

/// 获取群聊头像
#[tauri::command]
async fn get_group_avatar(group_id: i64) -> Result<String, String> {
    // OneBot 标准中没有直接的群头像API，通常使用QQ群头像链接
    Ok(format!("https://p.qlogo.cn/gh/{}/{}/640/", group_id, group_id))
}

/// 获取应用版本
#[tauri::command]
async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

/// 服务器状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusInfo {
    pub is_running: bool,
    pub status: String,
    pub connection_count: u32,
    pub active_bots: Vec<i64>,
}

/// 获取详细的服务器状态信息
#[tauri::command]
async fn get_server_status_info() -> Result<ServerStatusInfo, String> {
    let status = SERVER_STATUS.lock().await;
    let accounts = BOT_ACCOUNTS.lock().await;

    let active_bots: Vec<i64> = accounts.keys().cloned().collect();

    Ok(ServerStatusInfo {
        is_running: status.0,
        status: status.1.clone(),
        connection_count: status.2,
        active_bots,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 应用启动时初始化配置管理器
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match ConfigManager::new(&app_handle) {
                    Ok(manager) => {
                        let config_path = manager.get_config_path().display().to_string();
                        println!("配置管理器已初始化，配置文件路径: {}", config_path);

                        // 保存到全局变量
                        let mut config_guard = CONFIG_MANAGER.lock().await;
                        *config_guard = Some(manager);
                    }
                    Err(e) => {
                        eprintln!("初始化配置管理器失败: {}", e);
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            start_onebot_server,
            stop_onebot_server,
            get_onebot_status,
            get_server_runtime_status,
            init_config_manager,
            get_all_servers,
            add_server_config,
            update_server_config,
            remove_server_config,
            set_server_enabled,
            get_config_path,
            get_app_settings,
            update_app_settings,
            get_log_history,
            clear_log_history,
            subscribe_logs,
            get_bot_accounts,
            get_friends,
            get_groups,
            refresh_bot_data,
            get_server_status_info,
            send_private_message,
            send_group_message,
            get_user_avatar,
            get_group_avatar,
            get_app_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
