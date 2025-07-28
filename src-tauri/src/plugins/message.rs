use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use regex::Regex;
use lazy_static::lazy_static;

use crate::plugins::{PluginResult, PluginError};

/// CQ码类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CQCodeType {
    /// 纯文本
    Text,
    /// 艾特某人
    At,
    /// 图片
    Image,
    /// 语音
    Record,
    /// 视频
    Video,
    /// 表情
    Face,
    /// 音乐
    Music,
    /// 回复
    Reply,
    /// 转发
    Forward,
    /// 位置
    Location,
    /// 分享
    Share,
    /// 联系人
    Contact,
    /// 红包
    RedBag,
    /// 戳一戳
    Poke,
    /// 礼物
    Gift,
    /// 自定义
    Custom(String),
}

impl From<&str> for CQCodeType {
    fn from(s: &str) -> Self {
        match s {
            "text" => CQCodeType::Text,
            "at" => CQCodeType::At,
            "image" => CQCodeType::Image,
            "record" => CQCodeType::Record,
            "video" => CQCodeType::Video,
            "face" => CQCodeType::Face,
            "music" => CQCodeType::Music,
            "reply" => CQCodeType::Reply,
            "forward" => CQCodeType::Forward,
            "location" => CQCodeType::Location,
            "share" => CQCodeType::Share,
            "contact" => CQCodeType::Contact,
            "redbag" => CQCodeType::RedBag,
            "poke" => CQCodeType::Poke,
            "gift" => CQCodeType::Gift,
            _ => CQCodeType::Custom(s.to_string()),
        }
    }
}

impl ToString for CQCodeType {
    fn to_string(&self) -> String {
        match self {
            CQCodeType::Text => "text".to_string(),
            CQCodeType::At => "at".to_string(),
            CQCodeType::Image => "image".to_string(),
            CQCodeType::Record => "record".to_string(),
            CQCodeType::Video => "video".to_string(),
            CQCodeType::Face => "face".to_string(),
            CQCodeType::Music => "music".to_string(),
            CQCodeType::Reply => "reply".to_string(),
            CQCodeType::Forward => "forward".to_string(),
            CQCodeType::Location => "location".to_string(),
            CQCodeType::Share => "share".to_string(),
            CQCodeType::Contact => "contact".to_string(),
            CQCodeType::RedBag => "redbag".to_string(),
            CQCodeType::Poke => "poke".to_string(),
            CQCodeType::Gift => "gift".to_string(),
            CQCodeType::Custom(s) => s.clone(),
        }
    }
}

/// CQ码结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CQCode {
    /// CQ码类型
    pub code_type: CQCodeType,
    /// 参数
    pub params: HashMap<String, String>,
    /// 原始文本（如果是纯文本）
    pub text: Option<String>,
}

impl CQCode {
    /// 创建纯文本CQ码
    pub fn text(content: &str) -> Self {
        Self {
            code_type: CQCodeType::Text,
            params: HashMap::new(),
            text: Some(content.to_string()),
        }
    }

    /// 创建艾特CQ码
    pub fn at(user_id: i64) -> Self {
        let mut params = HashMap::new();
        params.insert("qq".to_string(), user_id.to_string());
        
        Self {
            code_type: CQCodeType::At,
            params,
            text: None,
        }
    }

    /// 创建艾特全体成员CQ码
    pub fn at_all() -> Self {
        let mut params = HashMap::new();
        params.insert("qq".to_string(), "all".to_string());
        
        Self {
            code_type: CQCodeType::At,
            params,
            text: None,
        }
    }

    /// 创建图片CQ码
    pub fn image(file: &str) -> Self {
        let mut params = HashMap::new();
        params.insert("file".to_string(), file.to_string());
        
        Self {
            code_type: CQCodeType::Image,
            params,
            text: None,
        }
    }

    /// 创建语音CQ码
    pub fn record(file: &str) -> Self {
        let mut params = HashMap::new();
        params.insert("file".to_string(), file.to_string());
        
        Self {
            code_type: CQCodeType::Record,
            params,
            text: None,
        }
    }

    /// 创建表情CQ码
    pub fn face(id: i32) -> Self {
        let mut params = HashMap::new();
        params.insert("id".to_string(), id.to_string());
        
        Self {
            code_type: CQCodeType::Face,
            params,
            text: None,
        }
    }

    /// 创建回复CQ码
    pub fn reply(message_id: i64) -> Self {
        let mut params = HashMap::new();
        params.insert("id".to_string(), message_id.to_string());
        
        Self {
            code_type: CQCodeType::Reply,
            params,
            text: None,
        }
    }

    /// 获取参数值
    pub fn get_param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    /// 设置参数
    #[allow(dead_code)]
    pub fn set_param(&mut self, key: &str, value: &str) {
        self.params.insert(key.to_string(), value.to_string());
    }

    /// 转换为CQ码字符串
    pub fn to_cq_string(&self) -> String {
        match &self.code_type {
            CQCodeType::Text => {
                self.text.as_ref().unwrap_or(&String::new()).clone()
            }
            _ => {
                let type_str = self.code_type.to_string();
                if self.params.is_empty() {
                    format!("[CQ:{}]", type_str)
                } else {
                    let params_str: Vec<String> = self.params.iter()
                        .map(|(k, v)| format!("{}={}", k, escape_cq_param(v)))
                        .collect();
                    format!("[CQ:{},{}]", type_str, params_str.join(","))
                }
            }
        }
    }
}

/// 转义CQ码参数
fn escape_cq_param(param: &str) -> String {
    param
        .replace("&", "&amp;")
        .replace("[", "&#91;")
        .replace("]", "&#93;")
        .replace(",", "&#44;")
}

/// 反转义CQ码参数
fn unescape_cq_param(param: &str) -> String {
    param
        .replace("&#44;", ",")
        .replace("&#93;", "]")
        .replace("&#91;", "[")
        .replace("&amp;", "&")
}

/// 消息解析器
pub struct MessageParser;

lazy_static! {
    static ref CQ_CODE_REGEX: Regex = Regex::new(r"\[CQ:([^,\]]+)(?:,([^\]]*))?\]").unwrap();
}

impl MessageParser {
    /// 解析CQ码字符串
    pub fn parse_cq_codes(message: &str) -> PluginResult<Vec<CQCode>> {
        let mut codes = Vec::new();
        let mut last_end = 0;

        for cap in CQ_CODE_REGEX.captures_iter(message) {
            let full_match = cap.get(0).unwrap();
            let start = full_match.start();
            let end = full_match.end();

            // 添加前面的纯文本
            if start > last_end {
                let text = &message[last_end..start];
                if !text.is_empty() {
                    codes.push(CQCode::text(text));
                }
            }

            // 解析CQ码
            let code_type = cap.get(1).unwrap().as_str();
            let params_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let mut params = HashMap::new();
            if !params_str.is_empty() {
                for param_pair in params_str.split(',') {
                    if let Some(eq_pos) = param_pair.find('=') {
                        let key = &param_pair[..eq_pos];
                        let value = &param_pair[eq_pos + 1..];
                        params.insert(key.to_string(), unescape_cq_param(value));
                    }
                }
            }

            codes.push(CQCode {
                code_type: CQCodeType::from(code_type),
                params,
                text: None,
            });

            last_end = end;
        }

        // 添加最后的纯文本
        if last_end < message.len() {
            let text = &message[last_end..];
            if !text.is_empty() {
                codes.push(CQCode::text(text));
            }
        }

        // 如果没有找到任何CQ码，整个消息就是纯文本
        if codes.is_empty() && !message.is_empty() {
            codes.push(CQCode::text(message));
        }

        Ok(codes)
    }

    /// 解析OneBot事件
    pub fn parse_onebot_event(event: &crate::onebot::OneBotEvent) -> PluginResult<ParsedMessage> {
        match event {
            crate::onebot::OneBotEvent::Message {
                message_id,
                message_type,
                sub_type,
                user_id,
                group_id,
                raw_message,
                message,
                sender,
                time,
                ..
            } => {
                // 将message转换为字符串
                let message_str = if let Some(msg_str) = message.as_str() {
                    msg_str
                } else {
                    &message.to_string()
                };

                let cq_codes = Self::parse_cq_codes(message_str)?;

                Ok(ParsedMessage {
                    message_id: *message_id,
                    message_type: message_type.clone(),
                    sub_type: sub_type.clone(),
                    user_id: *user_id,
                    group_id: *group_id,
                    raw_message: raw_message.clone(),
                    message: message_str.to_string(),
                    cq_codes,
                    sender: serde_json::to_value(sender).unwrap_or_default(),
                    time: *time,
                })
            }
            _ => Err(PluginError::MessageParseError("不支持的事件类型".to_string()))
        }
    }

    /// 提取纯文本内容
    pub fn extract_plain_text(cq_codes: &[CQCode]) -> String {
        cq_codes.iter()
            .filter_map(|code| {
                if code.code_type == CQCodeType::Text {
                    code.text.as_ref()
                } else {
                    None
                }
            })
            .cloned()
            .collect::<Vec<String>>()
            .join("")
    }

    /// 检查是否包含特定类型的CQ码
    #[allow(dead_code)]
    pub fn contains_cq_type(cq_codes: &[CQCode], code_type: &CQCodeType) -> bool {
        cq_codes.iter().any(|code| &code.code_type == code_type)
    }

    /// 获取特定类型的CQ码
    #[allow(dead_code)]
    pub fn get_cq_codes_by_type<'a>(cq_codes: &'a [CQCode], code_type: &CQCodeType) -> Vec<&'a CQCode> {
        cq_codes.iter()
            .filter(|code| &code.code_type == code_type)
            .collect()
    }
}

/// 解析后的消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedMessage {
    pub message_id: i64,
    pub message_type: String,
    pub sub_type: String,
    pub user_id: i64,
    pub group_id: Option<i64>,
    pub raw_message: String,
    pub message: String,
    pub cq_codes: Vec<CQCode>,
    pub sender: serde_json::Value,
    pub time: i64,
}

impl ParsedMessage {
    /// 获取纯文本内容
    pub fn get_plain_text(&self) -> String {
        MessageParser::extract_plain_text(&self.cq_codes)
    }

    /// 检查是否为群消息
    pub fn is_group_message(&self) -> bool {
        self.message_type == "group"
    }

    /// 检查是否为私聊消息
    pub fn is_private_message(&self) -> bool {
        self.message_type == "private"
    }

    /// 获取发送者昵称
    #[allow(dead_code)]
    pub fn get_sender_nickname(&self) -> Option<String> {
        self.sender.get("nickname")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// 检查是否艾特了机器人
    #[allow(dead_code)]
    pub fn is_at_bot(&self, bot_id: i64) -> bool {
        self.cq_codes.iter().any(|code| {
            if code.code_type == CQCodeType::At {
                if let Some(qq) = code.get_param("qq") {
                    return qq == &bot_id.to_string();
                }
            }
            false
        })
    }

    /// 检查是否艾特了全体成员
    #[allow(dead_code)]
    pub fn is_at_all(&self) -> bool {
        self.cq_codes.iter().any(|code| {
            if code.code_type == CQCodeType::At {
                if let Some(qq) = code.get_param("qq") {
                    return qq == "all";
                }
            }
            false
        })
    }
}

/// 消息构建器
#[allow(dead_code)]
pub struct MessageBuilder {
    codes: Vec<CQCode>,
}

impl MessageBuilder {
    /// 创建新的消息构建器
    pub fn new() -> Self {
        Self {
            codes: Vec::new(),
        }
    }

    /// 添加文本
    #[allow(dead_code)]
    pub fn text(mut self, content: &str) -> Self {
        self.codes.push(CQCode::text(content));
        self
    }

    /// 添加艾特
    #[allow(dead_code)]
    pub fn at(mut self, user_id: i64) -> Self {
        self.codes.push(CQCode::at(user_id));
        self
    }

    /// 添加艾特全体成员
    #[allow(dead_code)]
    pub fn at_all(mut self) -> Self {
        self.codes.push(CQCode::at_all());
        self
    }

    /// 添加图片
    #[allow(dead_code)]
    pub fn image(mut self, file: &str) -> Self {
        self.codes.push(CQCode::image(file));
        self
    }

    /// 添加语音
    #[allow(dead_code)]
    pub fn record(mut self, file: &str) -> Self {
        self.codes.push(CQCode::record(file));
        self
    }

    /// 添加表情
    #[allow(dead_code)]
    pub fn face(mut self, id: i32) -> Self {
        self.codes.push(CQCode::face(id));
        self
    }

    /// 添加回复
    #[allow(dead_code)]
    pub fn reply(mut self, message_id: i64) -> Self {
        self.codes.push(CQCode::reply(message_id));
        self
    }

    /// 添加自定义CQ码
    pub fn custom_cq(mut self, code: CQCode) -> Self {
        self.codes.push(code);
        self
    }

    /// 添加换行
    #[allow(dead_code)]
    pub fn line_break(mut self) -> Self {
        self.codes.push(CQCode::text("\n"));
        self
    }

    /// 添加空格
    #[allow(dead_code)]
    pub fn space(mut self) -> Self {
        self.codes.push(CQCode::text(" "));
        self
    }

    /// 构建消息字符串
    #[allow(dead_code)]
    pub fn build(self) -> String {
        self.codes.iter()
            .map(|code| code.to_cq_string())
            .collect::<Vec<String>>()
            .join("")
    }

    /// 构建CQ码数组
    #[allow(dead_code)]
    pub fn build_codes(self) -> Vec<CQCode> {
        self.codes
    }

    /// 检查消息是否为空
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.codes.is_empty() ||
        self.codes.iter().all(|code| {
            match &code.code_type {
                CQCodeType::Text => code.text.as_ref().map_or(true, |t| t.trim().is_empty()),
                _ => false
            }
        })
    }

    /// 获取消息长度（纯文本字符数）
    #[allow(dead_code)]
    pub fn text_length(&self) -> usize {
        self.codes.iter()
            .filter_map(|code| {
                if code.code_type == CQCodeType::Text {
                    code.text.as_ref()
                } else {
                    None
                }
            })
            .map(|text| text.len())
            .sum()
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 消息模板系统
#[allow(dead_code)]
pub struct MessageTemplate {
    template: String,
    variables: HashMap<String, String>,
}

impl MessageTemplate {
    /// 创建新的消息模板
    #[allow(dead_code)]
    pub fn new(template: &str) -> Self {
        Self {
            template: template.to_string(),
            variables: HashMap::new(),
        }
    }

    /// 设置变量
    #[allow(dead_code)]
    pub fn set_var(mut self, key: &str, value: &str) -> Self {
        self.variables.insert(key.to_string(), value.to_string());
        self
    }

    /// 设置多个变量
    #[allow(dead_code)]
    pub fn set_vars(mut self, vars: HashMap<String, String>) -> Self {
        self.variables.extend(vars);
        self
    }

    /// 渲染模板
    #[allow(dead_code)]
    pub fn render(&self) -> PluginResult<String> {
        let mut result = self.template.clone();

        for (key, value) in &self.variables {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }

        // 检查是否还有未替换的占位符
        if result.contains('{') && result.contains('}') {
            let remaining_placeholders: Vec<&str> = result
                .split('{')
                .skip(1)
                .filter_map(|s| s.split('}').next())
                .collect();

            if !remaining_placeholders.is_empty() {
                return Err(PluginError::MessageParseError(
                    format!("模板中存在未定义的变量: {:?}", remaining_placeholders)
                ));
            }
        }

        Ok(result)
    }

    /// 渲染为消息构建器
    #[allow(dead_code)]
    pub fn render_to_builder(&self) -> PluginResult<MessageBuilder> {
        let rendered = self.render()?;
        let codes = MessageParser::parse_cq_codes(&rendered)?;

        let mut builder = MessageBuilder::new();
        for code in codes {
            builder = builder.custom_cq(code);
        }

        Ok(builder)
    }
}

/// 消息格式验证器
#[allow(dead_code)]
pub struct MessageValidator;

impl MessageValidator {
    /// 验证消息长度
    #[allow(dead_code)]
    pub fn validate_length(message: &str, max_length: usize) -> PluginResult<()> {
        if message.len() > max_length {
            return Err(PluginError::MessageParseError(
                format!("消息长度超过限制: {} > {}", message.len(), max_length)
            ));
        }
        Ok(())
    }

    /// 验证CQ码格式
    #[allow(dead_code)]
    pub fn validate_cq_codes(cq_codes: &[CQCode]) -> PluginResult<()> {
        for code in cq_codes {
            match &code.code_type {
                CQCodeType::At => {
                    if !code.params.contains_key("qq") {
                        return Err(PluginError::MessageParseError(
                            "艾特CQ码缺少qq参数".to_string()
                        ));
                    }
                }
                CQCodeType::Image => {
                    if !code.params.contains_key("file") {
                        return Err(PluginError::MessageParseError(
                            "图片CQ码缺少file参数".to_string()
                        ));
                    }
                }
                CQCodeType::Record => {
                    if !code.params.contains_key("file") {
                        return Err(PluginError::MessageParseError(
                            "语音CQ码缺少file参数".to_string()
                        ));
                    }
                }
                CQCodeType::Face => {
                    if !code.params.contains_key("id") {
                        return Err(PluginError::MessageParseError(
                            "表情CQ码缺少id参数".to_string()
                        ));
                    }
                }
                CQCodeType::Reply => {
                    if !code.params.contains_key("id") {
                        return Err(PluginError::MessageParseError(
                            "回复CQ码缺少id参数".to_string()
                        ));
                    }
                }
                _ => {} // 其他类型暂不验证
            }
        }
        Ok(())
    }

    /// 验证消息是否包含敏感内容
    #[allow(dead_code)]
    pub fn validate_content(message: &str, forbidden_words: &[String]) -> PluginResult<()> {
        let message_lower = message.to_lowercase();
        for word in forbidden_words {
            if message_lower.contains(&word.to_lowercase()) {
                return Err(PluginError::MessageParseError(
                    format!("消息包含敏感词: {}", word)
                ));
            }
        }
        Ok(())
    }
}
