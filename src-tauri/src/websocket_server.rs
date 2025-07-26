use crate::onebot::{OneBotEvent, OneBotConfig, OneBotApiResponse, ConnectionStatus};
use futures_util::{SinkExt, StreamExt};
use serde_json;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

/// WebSocket 连接信息
#[derive(Debug)]
#[allow(dead_code)]
pub struct Connection {
    pub id: String,
    pub addr: SocketAddr,
    pub sender: mpsc::UnboundedSender<Message>,
}

/// OneBot 反向 WebSocket 服务器
pub struct OneBotServer {
    config: OneBotConfig,
    connections: Arc<RwLock<HashMap<String, Connection>>>,
    status: Arc<Mutex<ConnectionStatus>>,
    event_callback: Arc<Mutex<Option<fn(OneBotEvent)>>>,
    shutdown_sender: Arc<Mutex<Option<mpsc::UnboundedSender<()>>>>,
}

impl OneBotServer {
    /// 创建新的 OneBot 服务器实例
    pub fn new(config: OneBotConfig) -> Self {
        Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            event_callback: Arc::new(Mutex::new(None)),
            shutdown_sender: Arc::new(Mutex::new(None)),
        }
    }

    /// 设置事件回调函数
    pub async fn set_event_callback(&self, callback: fn(OneBotEvent)) {
        let mut cb = self.event_callback.lock().await;
        *cb = Some(callback);
    }

    /// 启动服务器
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        
        // 创建shutdown通道
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        {
            let mut sender = self.shutdown_sender.lock().await;
            *sender = Some(shutdown_tx);
        }
        
        {
            let mut status = self.status.lock().await;
            *status = ConnectionStatus::Connecting;
        }

        println!("OneBot 反向 WebSocket 服务器启动于: {}", addr);

        {
            let mut status = self.status.lock().await;
            *status = ConnectionStatus::Connected;
        }

        loop {
            tokio::select! {
                // 检查shutdown信号
                _ = shutdown_rx.recv() => {
                    println!("收到shutdown信号，停止服务器");
                    break;
                }
                // 接受新连接
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            let connections = Arc::clone(&self.connections);
                            let event_callback = Arc::clone(&self.event_callback);
                            let access_token = self.config.access_token.clone();

                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_connection(stream, addr, connections, event_callback, access_token).await {
                                    eprintln!("处理连接时出错: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("接受连接失败: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        // 关闭所有连接
        {
            let connections = self.connections.read().await;
            for conn in connections.values() {
                let _ = conn.sender.send(Message::Close(None));
            }
        }

        // 设置状态为已断开
        {
            let mut status = self.status.lock().await;
            *status = ConnectionStatus::Disconnected;
        }

        println!("OneBot 服务器已停止");
        Ok(())
    }

    /// 处理 WebSocket 连接
    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        connections: Arc<RwLock<HashMap<String, Connection>>>,
        event_callback: Arc<Mutex<Option<fn(OneBotEvent)>>>,
        access_token: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ws_stream = accept_async(stream).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        let connection_id = Uuid::new_v4().to_string();
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        
        // 保存连接信息
        {
            let mut conns = connections.write().await;
            conns.insert(connection_id.clone(), Connection {
                id: connection_id.clone(),
                addr,
                sender: tx,
            });
        }

        println!("新的 OneBot 连接: {} ({})", connection_id, addr);

        // 处理发送消息的任务
        let sender_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = ws_sender.send(message).await {
                    eprintln!("发送消息失败: {}", e);
                    break;
                }
            }
        });

        // 处理接收消息的任务
        let receiver_task = {
            let connection_id = connection_id.clone();
            let connections = Arc::clone(&connections);
            
            tokio::spawn(async move {
                while let Some(msg) = ws_receiver.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                                                         // 验证访问令牌（如果配置了）
                             if let Some(ref _token) = access_token {
                                 // 这里可以添加更复杂的验证逻辑
                                 // 简单示例：检查消息中是否包含正确的token
                             }
                            
                            // 解析 OneBot 事件
                            if let Ok(event) = serde_json::from_str::<OneBotEvent>(&text) {
                                println!("收到 OneBot 事件: {:?}", event);
                                
                                // 调用事件回调
                                if let Some(callback) = *event_callback.lock().await {
                                    callback(event);
                                }
                            } else {
                                println!("无法解析的消息: {}", text);
                            }
                        }
                        Ok(Message::Close(_)) => {
                            println!("连接 {} 已关闭", connection_id);
                            break;
                        }
                        Err(e) => {
                            eprintln!("WebSocket 错误: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
                
                // 清理连接
                let mut conns = connections.write().await;
                conns.remove(&connection_id);
                println!("连接 {} 已移除", connection_id);
            })
        };

        // 等待任务完成
        tokio::select! {
            _ = sender_task => {},
            _ = receiver_task => {},
        }

        Ok(())
    }

    /// 停止服务器
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let sender = self.shutdown_sender.lock().await;
        if let Some(ref tx) = *sender {
            tx.send(())?;
            println!("已发送shutdown信号");
        }
        Ok(())
    }

    /// 获取连接状态
    pub async fn get_status(&self) -> ConnectionStatus {
        self.status.lock().await.clone()
    }

    /// 获取活跃连接数量
    pub async fn get_connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// 发送 API 响应到指定连接
    #[allow(dead_code)]
    pub async fn send_api_response(
        &self,
        connection_id: &str,
        response: OneBotApiResponse,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connections = self.connections.read().await;
        if let Some(conn) = connections.get(connection_id) {
            let message = serde_json::to_string(&response)?;
            conn.sender.send(Message::Text(message))?;
        }
        Ok(())
    }

    /// 广播 API 响应到所有连接
    #[allow(dead_code)]
    pub async fn broadcast_api_response(
        &self,
        response: OneBotApiResponse,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connections = self.connections.read().await;
        let message = serde_json::to_string(&response)?;
        
        for conn in connections.values() {
            let _ = conn.sender.send(Message::Text(message.clone()));
        }
        Ok(())
    }
} 