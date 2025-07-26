use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OneBot 事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "post_type")]
pub enum OneBotEvent {
    #[serde(rename = "message")]
    Message {
        time: i64,
        self_id: i64,
        message_type: String,
        sub_type: String,
        message_id: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        message_seq: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        real_id: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        real_seq: Option<String>,
        user_id: i64,
        message: serde_json::Value, // 支持数组和字符串格式
        raw_message: String,
        font: i32,
        sender: Sender,
        #[serde(skip_serializing_if = "Option::is_none")]
        group_id: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        message_format: Option<String>,
    },
    #[serde(rename = "notice")]
    Notice {
        time: i64,
        self_id: i64,
        notice_type: String,
        sub_type: String,
        user_id: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        group_id: Option<i64>,
        #[serde(flatten)]
        extra: HashMap<String, serde_json::Value>,
    },
    #[serde(rename = "request")]
    Request {
        time: i64,
        self_id: i64,
        request_type: String,
        sub_type: String,
        user_id: i64,
        comment: String,
        flag: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        group_id: Option<i64>,
    },
    #[serde(rename = "meta_event")]
    MetaEvent {
        time: i64,
        self_id: i64,
        meta_event_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sub_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<OneBotStatus>,
        #[serde(skip_serializing_if = "Option::is_none")]
        interval: Option<i64>,
        #[serde(flatten)]
        extra: HashMap<String, serde_json::Value>,
    },
}

/// OneBot 状态信息（用于心跳包）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneBotStatus {
    pub online: bool,
    pub good: bool,
}

/// 发送者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sender {
    pub user_id: i64,
    pub nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// 消息段（用于解析消息内容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSegment {
    #[serde(rename = "type")]
    pub seg_type: String,
    pub data: HashMap<String, serde_json::Value>,
}

/// OneBot API 调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneBotApiRequest {
    pub action: String,
    pub params: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<String>,
}

/// OneBot API 调用响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneBotApiResponse {
    pub status: String,
    pub retcode: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wording: Option<String>,
}

/// WebSocket 连接状态
#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Disconnected,
    Connected,
    Connecting,
}

/// OneBot 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneBotConfig {
    pub host: String,
    pub port: u16,
    pub access_token: Option<String>,
    pub secret: Option<String>,
}

impl Default for OneBotConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            access_token: None,
            secret: None,
        }
    }
}

/// 解析消息内容为纯文本
pub fn extract_plain_text(message: &serde_json::Value) -> String {
    match message {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Array(segments) => {
            let mut text = String::new();
            for segment in segments {
                if let Some(seg) = segment.as_object() {
                    if let Some(seg_type) = seg.get("type").and_then(|v| v.as_str()) {
                        if seg_type == "text" {
                            if let Some(data) = seg.get("data").and_then(|v| v.as_object()) {
                                if let Some(content) = data.get("text").and_then(|v| v.as_str()) {
                                    text.push_str(content);
                                }
                            }
                        }
                    }
                }
            }
            text
        }
        _ => message.to_string(),
    }
}

/// 格式化事件为友好的日志信息
pub fn format_event_log(event: &OneBotEvent) -> String {
    match event {
        OneBotEvent::Message { 
            message_type, 
            group_id, 
            sender, 
            message, 
            raw_message,
            .. 
        } => {
            let plain_text = extract_plain_text(message);
            let sender_name = if let Some(card) = &sender.card {
                if card.is_empty() { &sender.nickname } else { card }
            } else {
                &sender.nickname
            };
            
            if message_type == "group" {
                if let Some(gid) = group_id {
                    // 根据已知的群ID显示群名称（可以后续扩展为查询群信息）
                    let group_name = match *gid {
                        707972378 => "修仙",
                        _ => "未知群",
                    };
                    format!("[INFO] 接收 <- 群聊 [{}({})] [{}({})] {}", 
                        group_name, gid, sender_name, sender.user_id, plain_text)
                } else {
                    format!("[INFO] 接收 <- 群聊消息 [{}({})] {}", 
                        sender_name, sender.user_id, plain_text)
                }
            } else if message_type == "private" {
                format!("[INFO] 接收 <- 私聊 [{}({})] {}", 
                    sender_name, sender.user_id, plain_text)
            } else {
                format!("[INFO] 接收 <- {} [{}({})] {}", 
                    message_type, sender_name, sender.user_id, raw_message)
            }
        }
        OneBotEvent::MetaEvent { meta_event_type, interval, status, .. } => {
            match meta_event_type.as_str() {
                "heartbeat" => {
                    let interval_info = interval.map(|i| format!("{}ms", i)).unwrap_or_default();
                    let status_info = status.as_ref()
                        .map(|s| format!("(在线:{}, 状态:{})", s.online, if s.good { "良好" } else { "异常" }))
                        .unwrap_or_default();
                    format!("[DEBUG] 心跳包 {} {}", interval_info, status_info)
                }
                "lifecycle" => {
                    format!("[INFO] 生命周期事件: 连接建立")
                }
                _ => {
                    format!("[DEBUG] 元事件: {}", meta_event_type)
                }
            }
        }
        OneBotEvent::Notice { notice_type, .. } => {
            format!("[INFO] 通知事件: {}", notice_type)
        }
        OneBotEvent::Request { request_type, .. } => {
            format!("[INFO] 请求事件: {}", request_type)
        }
    }
} 