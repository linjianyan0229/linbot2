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
        user_id: i64,
        message: String,
        raw_message: String,
        font: i32,
        sender: Sender,
        #[serde(skip_serializing_if = "Option::is_none")]
        group_id: Option<i64>,
    },
    #[serde(rename = "notice")]
    Notice {
        time: i64,
        self_id: i64,
        notice_type: String,
        sub_type: String,
        user_id: i64,
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
    },
    #[serde(rename = "meta_event")]
    MetaEvent {
        time: i64,
        self_id: i64,
        meta_event_type: String,
        sub_type: String,
        #[serde(flatten)]
        extra: HashMap<String, serde_json::Value>,
    },
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