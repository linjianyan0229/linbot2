use std::time::Duration;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use tokio::time::timeout;

use crate::plugins::{PluginResult, PluginError};

/// OneBot API响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct OneBotResponse<T> {
    pub status: String,
    pub retcode: i32,
    pub data: Option<T>,
    pub message: Option<String>,
}

/// 发送消息响应
#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub message_id: i64,
}

/// 用户信息
#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: i64,
    pub nickname: String,
    pub sex: String,
    pub age: i32,
}

/// 群信息
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupInfo {
    pub group_id: i64,
    pub group_name: String,
    pub member_count: i32,
    pub max_member_count: i32,
}

/// 群成员信息
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupMemberInfo {
    pub group_id: i64,
    pub user_id: i64,
    pub nickname: String,
    pub card: String,
    pub sex: String,
    pub age: i32,
    pub area: String,
    pub join_time: i64,
    pub last_sent_time: i64,
    pub level: String,
    pub role: String,
    pub unfriendly: bool,
    pub title: String,
    pub title_expire_time: i64,
    pub card_changeable: bool,
}

/// OneBot API客户端
#[allow(dead_code)]
pub struct OneBotApi {
    #[allow(dead_code)]
    client: Client,
    #[allow(dead_code)]
    base_url: String,
    #[allow(dead_code)]
    timeout: Duration,
    #[allow(dead_code)]
    retry_count: u32,
}

impl OneBotApi {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            timeout: Duration::from_secs(30),
            retry_count: 3,
        }
    }

    /// 设置超时时间
    #[allow(dead_code)]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 设置重试次数
    #[allow(dead_code)]
    pub fn with_retry_count(mut self, retry_count: u32) -> Self {
        self.retry_count = retry_count;
        self
    }

    /// 发送API请求
    async fn send_request<T, R>(&self, endpoint: &str, params: &T) -> PluginResult<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.base_url, endpoint);
        
        for attempt in 0..=self.retry_count {
            let request = self.client
                .post(&url)
                .json(params)
                .timeout(self.timeout);

            match timeout(self.timeout, request.send()).await {
                Ok(Ok(response)) => {
                    if response.status().is_success() {
                        let onebot_response: OneBotResponse<R> = response.json().await
                            .map_err(|e| PluginError::ApiError(format!("解析响应失败: {}", e)))?;

                        if onebot_response.status == "ok" {
                            return onebot_response.data
                                .ok_or_else(|| PluginError::ApiError("响应数据为空".to_string()));
                        } else {
                            return Err(PluginError::ApiError(
                                onebot_response.message.unwrap_or_else(|| 
                                    format!("API调用失败，错误码: {}", onebot_response.retcode)
                                )
                            ));
                        }
                    } else {
                        if attempt == self.retry_count {
                            return Err(PluginError::ApiError(
                                format!("HTTP错误: {}", response.status())
                            ));
                        }
                    }
                }
                Ok(Err(e)) => {
                    if attempt == self.retry_count {
                        return Err(PluginError::ApiError(format!("请求失败: {}", e)));
                    }
                }
                Err(_) => {
                    if attempt == self.retry_count {
                        return Err(PluginError::ApiError("请求超时".to_string()));
                    }
                }
            }

            // 重试前等待
            if attempt < self.retry_count {
                tokio::time::sleep(Duration::from_millis(1000 * (attempt + 1) as u64)).await;
            }
        }

        Err(PluginError::ApiError("所有重试都失败了".to_string()))
    }

    /// 发送私聊消息
    #[allow(dead_code)]
    pub async fn send_private_msg(&self, user_id: i64, message: &str) -> PluginResult<i64> {
        let params = serde_json::json!({
            "user_id": user_id,
            "message": message
        });

        let response: SendMessageResponse = self.send_request("send_private_msg", &params).await?;
        Ok(response.message_id)
    }

    /// 发送群聊消息
    #[allow(dead_code)]
    pub async fn send_group_msg(&self, group_id: i64, message: &str) -> PluginResult<i64> {
        let params = serde_json::json!({
            "group_id": group_id,
            "message": message
        });

        let response: SendMessageResponse = self.send_request("send_group_msg", &params).await?;
        Ok(response.message_id)
    }

    /// 发送消息（自动判断类型）
    #[allow(dead_code)]
    pub async fn send_msg(&self, message_type: &str, target_id: i64, message: &str) -> PluginResult<i64> {
        match message_type {
            "private" => self.send_private_msg(target_id, message).await,
            "group" => self.send_group_msg(target_id, message).await,
            _ => Err(PluginError::ApiError(format!("不支持的消息类型: {}", message_type)))
        }
    }

    /// 撤回消息
    #[allow(dead_code)]
    pub async fn delete_msg(&self, message_id: i64) -> PluginResult<()> {
        let params = serde_json::json!({
            "message_id": message_id
        });

        let _: serde_json::Value = self.send_request("delete_msg", &params).await?;
        Ok(())
    }

    /// 获取消息
    #[allow(dead_code)]
    pub async fn get_msg(&self, message_id: i64) -> PluginResult<serde_json::Value> {
        let params = serde_json::json!({
            "message_id": message_id
        });

        self.send_request("get_msg", &params).await
    }

    /// 获取转发消息
    #[allow(dead_code)]
    pub async fn get_forward_msg(&self, id: &str) -> PluginResult<serde_json::Value> {
        let params = serde_json::json!({
            "id": id
        });

        self.send_request("get_forward_msg", &params).await
    }

    /// 发送点赞
    #[allow(dead_code)]
    pub async fn send_like(&self, user_id: i64, times: i32) -> PluginResult<()> {
        let params = serde_json::json!({
            "user_id": user_id,
            "times": times
        });

        let _: serde_json::Value = self.send_request("send_like", &params).await?;
        Ok(())
    }

    /// 群组踢人
    #[allow(dead_code)]
    pub async fn set_group_kick(&self, group_id: i64, user_id: i64, reject_add_request: bool) -> PluginResult<()> {
        let params = serde_json::json!({
            "group_id": group_id,
            "user_id": user_id,
            "reject_add_request": reject_add_request
        });

        let _: serde_json::Value = self.send_request("set_group_kick", &params).await?;
        Ok(())
    }

    /// 群组禁言
    #[allow(dead_code)]
    pub async fn set_group_ban(&self, group_id: i64, user_id: i64, duration: i64) -> PluginResult<()> {
        let params = serde_json::json!({
            "group_id": group_id,
            "user_id": user_id,
            "duration": duration
        });

        let _: serde_json::Value = self.send_request("set_group_ban", &params).await?;
        Ok(())
    }

    /// 群组匿名用户禁言
    #[allow(dead_code)]
    pub async fn set_group_anonymous_ban(&self, group_id: i64, anonymous: serde_json::Value, duration: i64) -> PluginResult<()> {
        let params = serde_json::json!({
            "group_id": group_id,
            "anonymous": anonymous,
            "duration": duration
        });

        let _: serde_json::Value = self.send_request("set_group_anonymous_ban", &params).await?;
        Ok(())
    }

    /// 群组全员禁言
    #[allow(dead_code)]
    pub async fn set_group_whole_ban(&self, group_id: i64, enable: bool) -> PluginResult<()> {
        let params = serde_json::json!({
            "group_id": group_id,
            "enable": enable
        });

        let _: serde_json::Value = self.send_request("set_group_whole_ban", &params).await?;
        Ok(())
    }

    /// 群组设置管理员
    #[allow(dead_code)]
    pub async fn set_group_admin(&self, group_id: i64, user_id: i64, enable: bool) -> PluginResult<()> {
        let params = serde_json::json!({
            "group_id": group_id,
            "user_id": user_id,
            "enable": enable
        });

        let _: serde_json::Value = self.send_request("set_group_admin", &params).await?;
        Ok(())
    }

    /// 群组匿名
    #[allow(dead_code)]
    pub async fn set_group_anonymous(&self, group_id: i64, enable: bool) -> PluginResult<()> {
        let params = serde_json::json!({
            "group_id": group_id,
            "enable": enable
        });

        let _: serde_json::Value = self.send_request("set_group_anonymous", &params).await?;
        Ok(())
    }

    /// 设置群名片（群备注）
    #[allow(dead_code)]
    pub async fn set_group_card(&self, group_id: i64, user_id: i64, card: &str) -> PluginResult<()> {
        let params = serde_json::json!({
            "group_id": group_id,
            "user_id": user_id,
            "card": card
        });

        let _: serde_json::Value = self.send_request("set_group_card", &params).await?;
        Ok(())
    }

    /// 设置群名
    #[allow(dead_code)]
    pub async fn set_group_name(&self, group_id: i64, group_name: &str) -> PluginResult<()> {
        let params = serde_json::json!({
            "group_id": group_id,
            "group_name": group_name
        });

        let _: serde_json::Value = self.send_request("set_group_name", &params).await?;
        Ok(())
    }

    /// 退出群组
    #[allow(dead_code)]
    pub async fn set_group_leave(&self, group_id: i64, is_dismiss: bool) -> PluginResult<()> {
        let params = serde_json::json!({
            "group_id": group_id,
            "is_dismiss": is_dismiss
        });

        let _: serde_json::Value = self.send_request("set_group_leave", &params).await?;
        Ok(())
    }

    /// 设置群组专属头衔
    #[allow(dead_code)]
    pub async fn set_group_special_title(&self, group_id: i64, user_id: i64, special_title: &str, duration: i64) -> PluginResult<()> {
        let params = serde_json::json!({
            "group_id": group_id,
            "user_id": user_id,
            "special_title": special_title,
            "duration": duration
        });

        let _: serde_json::Value = self.send_request("set_group_special_title", &params).await?;
        Ok(())
    }

    /// 处理加好友请求
    #[allow(dead_code)]
    pub async fn set_friend_add_request(&self, flag: &str, approve: bool, remark: &str) -> PluginResult<()> {
        let params = serde_json::json!({
            "flag": flag,
            "approve": approve,
            "remark": remark
        });

        let _: serde_json::Value = self.send_request("set_friend_add_request", &params).await?;
        Ok(())
    }

    /// 处理加群请求／邀请
    #[allow(dead_code)]
    pub async fn set_group_add_request(&self, flag: &str, sub_type: &str, approve: bool, reason: &str) -> PluginResult<()> {
        let params = serde_json::json!({
            "flag": flag,
            "sub_type": sub_type,
            "approve": approve,
            "reason": reason
        });

        let _: serde_json::Value = self.send_request("set_group_add_request", &params).await?;
        Ok(())
    }

    /// 获取登录号信息
    #[allow(dead_code)]
    pub async fn get_login_info(&self) -> PluginResult<UserInfo> {
        let params = serde_json::json!({});
        self.send_request("get_login_info", &params).await
    }

    /// 获取陌生人信息
    #[allow(dead_code)]
    pub async fn get_stranger_info(&self, user_id: i64, no_cache: bool) -> PluginResult<UserInfo> {
        let params = serde_json::json!({
            "user_id": user_id,
            "no_cache": no_cache
        });

        self.send_request("get_stranger_info", &params).await
    }

    /// 获取好友列表
    #[allow(dead_code)]
    pub async fn get_friend_list(&self) -> PluginResult<Vec<UserInfo>> {
        let params = serde_json::json!({});
        self.send_request("get_friend_list", &params).await
    }

    /// 获取群信息
    #[allow(dead_code)]
    pub async fn get_group_info(&self, group_id: i64, no_cache: bool) -> PluginResult<GroupInfo> {
        let params = serde_json::json!({
            "group_id": group_id,
            "no_cache": no_cache
        });

        self.send_request("get_group_info", &params).await
    }

    /// 获取群列表
    #[allow(dead_code)]
    pub async fn get_group_list(&self) -> PluginResult<Vec<GroupInfo>> {
        let params = serde_json::json!({});
        self.send_request("get_group_list", &params).await
    }

    /// 获取群成员信息
    #[allow(dead_code)]
    pub async fn get_group_member_info(&self, group_id: i64, user_id: i64, no_cache: bool) -> PluginResult<GroupMemberInfo> {
        let params = serde_json::json!({
            "group_id": group_id,
            "user_id": user_id,
            "no_cache": no_cache
        });

        self.send_request("get_group_member_info", &params).await
    }

    /// 获取群成员列表
    #[allow(dead_code)]
    pub async fn get_group_member_list(&self, group_id: i64) -> PluginResult<Vec<GroupMemberInfo>> {
        let params = serde_json::json!({
            "group_id": group_id
        });

        self.send_request("get_group_member_list", &params).await
    }

    /// 获取群荣誉信息
    #[allow(dead_code)]
    pub async fn get_group_honor_info(&self, group_id: i64, honor_type: &str) -> PluginResult<serde_json::Value> {
        let params = serde_json::json!({
            "group_id": group_id,
            "type": honor_type
        });

        self.send_request("get_group_honor_info", &params).await
    }

    /// 获取Cookies
    #[allow(dead_code)]
    pub async fn get_cookies(&self, domain: &str) -> PluginResult<serde_json::Value> {
        let params = serde_json::json!({
            "domain": domain
        });

        self.send_request("get_cookies", &params).await
    }

    /// 获取CSRF Token
    #[allow(dead_code)]
    pub async fn get_csrf_token(&self) -> PluginResult<serde_json::Value> {
        let params = serde_json::json!({});
        self.send_request("get_csrf_token", &params).await
    }

    /// 获取QQ相关接口凭证
    #[allow(dead_code)]
    pub async fn get_credentials(&self, domain: &str) -> PluginResult<serde_json::Value> {
        let params = serde_json::json!({
            "domain": domain
        });

        self.send_request("get_credentials", &params).await
    }

    /// 获取版本信息
    #[allow(dead_code)]
    pub async fn get_version_info(&self) -> PluginResult<serde_json::Value> {
        let params = serde_json::json!({});
        self.send_request("get_version_info", &params).await
    }

    /// 重启OneBot实现
    #[allow(dead_code)]
    pub async fn set_restart(&self, delay: i64) -> PluginResult<()> {
        let params = serde_json::json!({
            "delay": delay
        });

        let _: serde_json::Value = self.send_request("set_restart", &params).await?;
        Ok(())
    }

    /// 清理缓存
    #[allow(dead_code)]
    pub async fn clean_cache(&self) -> PluginResult<()> {
        let params = serde_json::json!({});
        let _: serde_json::Value = self.send_request("clean_cache", &params).await?;
        Ok(())
    }
}

/// NapCat API兼容层
#[allow(dead_code)]
pub struct NapCatApi {
    onebot_api: OneBotApi,
}

impl NapCatApi {
    #[allow(dead_code)]
    pub fn new(base_url: String) -> Self {
        Self {
            onebot_api: OneBotApi::new(base_url),
        }
    }

    /// 获取OneBot API实例
    #[allow(dead_code)]
    pub fn onebot(&self) -> &OneBotApi {
        &self.onebot_api
    }

    /// NapCat特有的API方法可以在这里添加
    /// 例如：获取设备信息、获取在线状态等

    /// 获取在线机器人列表
    #[allow(dead_code)]
    pub async fn get_online_clients(&self, no_cache: bool) -> PluginResult<serde_json::Value> {
        let params = serde_json::json!({
            "no_cache": no_cache
        });

        self.onebot_api.send_request("get_online_clients", &params).await
    }

    /// 获取群文件系统信息
    #[allow(dead_code)]
    pub async fn get_group_file_system_info(&self, group_id: i64) -> PluginResult<serde_json::Value> {
        let params = serde_json::json!({
            "group_id": group_id
        });

        self.onebot_api.send_request("get_group_file_system_info", &params).await
    }

    /// 获取群根目录文件列表
    #[allow(dead_code)]
    pub async fn get_group_root_files(&self, group_id: i64) -> PluginResult<serde_json::Value> {
        let params = serde_json::json!({
            "group_id": group_id
        });

        self.onebot_api.send_request("get_group_root_files", &params).await
    }

    /// 获取群子目录文件列表
    #[allow(dead_code)]
    pub async fn get_group_files_by_folder(&self, group_id: i64, folder_id: &str) -> PluginResult<serde_json::Value> {
        let params = serde_json::json!({
            "group_id": group_id,
            "folder_id": folder_id
        });

        self.onebot_api.send_request("get_group_files_by_folder", &params).await
    }

    /// 获取群文件资源链接
    #[allow(dead_code)]
    pub async fn get_group_file_url(&self, group_id: i64, file_id: &str, busid: i32) -> PluginResult<serde_json::Value> {
        let params = serde_json::json!({
            "group_id": group_id,
            "file_id": file_id,
            "busid": busid
        });

        self.onebot_api.send_request("get_group_file_url", &params).await
    }
}
