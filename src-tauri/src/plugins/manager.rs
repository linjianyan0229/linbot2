use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::plugins::{
    PluginInstance, PluginStatus, PluginResult, PluginError,
    PluginContext, PluginMetadata
};
use crate::plugins::message::ParsedMessage;
use crate::plugins::command::CommandMatch;
use crate::plugins::loader::PluginLoader;
use crate::plugins::config::PluginConfig;

/// 插件管理器
pub struct PluginManager {
    /// 已加载的插件实例
    plugins: HashMap<Uuid, PluginInstance>,
    /// 插件名称到ID的映射
    name_to_id: HashMap<String, Uuid>,
    /// 插件加载器
    loader: PluginLoader,
    /// 插件目录
    plugins_dir: PathBuf,
    /// 是否已初始化
    initialized: bool,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            name_to_id: HashMap::new(),
            loader: PluginLoader::new(),
            plugins_dir: PathBuf::from("plugins"),
            initialized: false,
        }
    }

    /// 初始化插件管理器
    pub async fn initialize(&mut self) -> PluginResult<()> {
        if self.initialized {
            return Ok(());
        }

        // 确保插件目录存在
        if !self.plugins_dir.exists() {
            std::fs::create_dir_all(&self.plugins_dir)?;
        }

        // 扫描并加载插件
        self.scan_and_load_plugins().await?;

        self.initialized = true;
        Ok(())
    }

    /// 扫描并加载插件
    async fn scan_and_load_plugins(&mut self) -> PluginResult<()> {
        let plugin_files = self.scan_plugin_files()?;
        
        for file_path in plugin_files {
            if let Err(e) = self.load_plugin_from_file(&file_path).await {
                eprintln!("加载插件失败 {}: {}", file_path.display(), e);
            }
        }

        Ok(())
    }

    /// 扫描插件文件
    fn scan_plugin_files(&self) -> PluginResult<Vec<PathBuf>> {
        let mut plugin_files = Vec::new();

        if !self.plugins_dir.exists() {
            return Ok(plugin_files);
        }

        for entry in std::fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                // 检查文件扩展名
                if let Some(ext) = path.extension() {
                    if ext == "dll" || ext == "so" || ext == "dylib" {
                        plugin_files.push(path);
                    }
                }
            } else if path.is_dir() {
                // 检查目录中的插件配置文件
                let config_file = path.join("plugin.toml");
                if config_file.exists() {
                    plugin_files.push(path);
                }
            }
        }

        Ok(plugin_files)
    }

    /// 从文件加载插件
    async fn load_plugin_from_file(&mut self, file_path: &Path) -> PluginResult<Uuid> {
        // 加载插件
        let plugin = self.loader.load_plugin(file_path).await?;
        let info = plugin.get_info();

        // 检查插件是否已存在
        if self.name_to_id.contains_key(&info.name) {
            return Err(PluginError::PluginAlreadyExists(info.name));
        }

        // 加载插件配置
        let config = PluginConfig::load_for_plugin(&info.name).await
            .unwrap_or_else(|_| PluginConfig::default());

        // 创建插件实例
        let mut instance = PluginInstance::new(info.clone(), config);
        instance.plugin = Some(plugin);
        instance.status = PluginStatus::Loaded;

        let plugin_id = instance.id;

        // 注册插件
        self.plugins.insert(plugin_id, instance);
        self.name_to_id.insert(info.name, plugin_id);

        Ok(plugin_id)
    }

    /// 启用插件
    pub async fn enable_plugin(&mut self, plugin_id: &Uuid) -> PluginResult<()> {
        // 先获取插件信息和配置
        let (plugin_name, plugin_config, plugin_arc) = {
            let instance = self.plugins.get(plugin_id)
                .ok_or_else(|| PluginError::PluginNotFound(plugin_id.to_string()))?;

            if instance.status == PluginStatus::Running {
                return Ok(());
            }

            (
                instance.info.name.clone(),
                instance.config.clone(),
                instance.plugin.clone()
            )
        };

        if let Some(plugin) = plugin_arc {
            // 创建插件上下文
            let context = self.create_plugin_context(&plugin_name, &plugin_config).await?;

            // 调用插件生命周期方法
            plugin.on_init(&context).await?;
            plugin.on_start(&context).await?;

            // 更新插件状态
            let instance = self.plugins.get_mut(plugin_id).unwrap();
            instance.status = PluginStatus::Running;
            instance.stats.start_time = Some(chrono::Utc::now());
        }

        Ok(())
    }

    /// 禁用插件
    pub async fn disable_plugin(&mut self, plugin_id: &Uuid) -> PluginResult<()> {
        // 先获取插件信息和配置
        let (plugin_name, plugin_config, plugin_arc) = {
            let instance = self.plugins.get(plugin_id)
                .ok_or_else(|| PluginError::PluginNotFound(plugin_id.to_string()))?;

            if instance.status != PluginStatus::Running {
                return Ok(());
            }

            (
                instance.info.name.clone(),
                instance.config.clone(),
                instance.plugin.clone()
            )
        };

        if let Some(plugin) = plugin_arc {
            // 创建插件上下文
            let context = self.create_plugin_context(&plugin_name, &plugin_config).await?;

            // 调用插件生命周期方法
            plugin.on_stop(&context).await?;

            // 更新插件状态
            let instance = self.plugins.get_mut(plugin_id).unwrap();
            instance.status = PluginStatus::Paused;
        }

        Ok(())
    }

    /// 卸载插件
    pub async fn unload_plugin(&mut self, plugin_id: &Uuid) -> PluginResult<()> {
        // 先禁用插件
        if let Some(instance) = self.plugins.get(plugin_id) {
            if instance.status == PluginStatus::Running {
                self.disable_plugin(plugin_id).await?;
            }
        }

        // 获取插件信息进行卸载
        let (plugin_name, plugin_config, plugin_arc) = {
            let instance = self.plugins.get(plugin_id)
                .ok_or_else(|| PluginError::PluginNotFound(plugin_id.to_string()))?;

            (
                instance.info.name.clone(),
                instance.config.clone(),
                instance.plugin.clone()
            )
        };

        if let Some(plugin) = plugin_arc {
            // 创建插件上下文
            let context = self.create_plugin_context(&plugin_name, &plugin_config).await?;

            // 调用插件卸载方法
            plugin.on_unload(&context).await?;
        }

        // 从映射中移除
        self.name_to_id.remove(&plugin_name);
        self.plugins.remove(plugin_id);

        Ok(())
    }

    /// 重新加载插件
    #[allow(dead_code)]
    pub async fn reload_plugin(&mut self, plugin_id: &Uuid) -> PluginResult<()> {
        let _instance = self.plugins.get(plugin_id)
            .ok_or_else(|| PluginError::PluginNotFound(plugin_id.to_string()))?;

        // 卸载现有插件
        self.unload_plugin(plugin_id).await?;

        // 重新加载插件
        // 这里需要保存原始文件路径信息
        // 暂时返回错误，需要在实际实现中完善
        Err(PluginError::Other("重新加载功能暂未完全实现".to_string()))
    }

    /// 处理消息
    #[allow(dead_code)]
    pub async fn handle_message(&self, message: &ParsedMessage) -> PluginResult<()> {
        // 按优先级排序插件
        let mut sorted_plugins: Vec<_> = self.plugins.values()
            .filter(|instance| instance.can_process_messages())
            .collect();

        sorted_plugins.sort_by_key(|instance| {
            instance.plugin.as_ref()
                .map(|p| p.get_priority())
                .unwrap_or(999)
        });

        for instance in sorted_plugins {
            if let Some(plugin) = &instance.plugin {
                if plugin.should_handle_message(message).await {
                    let context = self.create_plugin_context(&instance.info.name, &instance.config).await?;

                    match plugin.handle_message(&context, message).await {
                        Ok(_handled) => {
                            // 如果插件处理了消息，可以选择是否继续传递给其他插件
                            // 这里继续传递，可以根据需要修改
                        }
                        Err(e) => {
                            eprintln!("插件 {} 处理消息时出错: {}", instance.info.name, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 处理命令
    #[allow(dead_code)]
    pub async fn handle_command(&self, command: &CommandMatch, message: &ParsedMessage) -> PluginResult<()> {
        // 找到匹配的插件
        for instance in self.plugins.values() {
            if instance.can_process_messages() {
                if let Some(plugin) = &instance.plugin {
                    if plugin.should_handle_command(command).await {
                        let context = self.create_plugin_context(&instance.info.name, &instance.config).await?;
                        plugin.handle_command(&context, command, message).await?;
                        break; // 只让第一个匹配的插件处理
                    }
                }
            }
        }

        Ok(())
    }

    /// 创建插件上下文
    async fn create_plugin_context(&self, plugin_name: &str, config: &PluginConfig) -> PluginResult<PluginContext> {
        use crate::plugins::api::OneBotApi;
        use crate::plugins::logger::DefaultPluginLogger;
        use std::sync::Arc;

        // 创建数据目录
        let data_dir = self.plugins_dir.join(plugin_name).join("data");
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }

        // 创建API实例（这里需要传入实际的OneBot连接信息）
        let api = Arc::new(OneBotApi::new("http://localhost:3000".to_string()));

        // 创建日志记录器
        let logger = Arc::new(DefaultPluginLogger::new());

        Ok(PluginContext::new(
            api,
            config.settings.clone(),
            data_dir,
            logger,
        ))
    }

    /// 获取所有插件信息
    pub fn get_all_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.values()
            .map(|instance| PluginMetadata {
                file_path: PathBuf::from(""), // 需要保存文件路径
                info: instance.info.clone(),
                loaded: instance.plugin.is_some(),
                enabled: instance.is_running(),
                load_time: instance.stats.start_time,
                last_error: None, // 需要实现错误跟踪
            })
            .collect()
    }

    /// 根据名称获取插件
    #[allow(dead_code)]
    pub fn get_plugin_by_name(&self, name: &str) -> Option<&PluginInstance> {
        self.name_to_id.get(name)
            .and_then(|id| self.plugins.get(id))
    }

    /// 获取插件统计信息
    pub fn get_plugin_stats(&self, plugin_id: &Uuid) -> Option<&crate::plugins::PluginStats> {
        self.plugins.get(plugin_id)
            .map(|instance| &instance.stats)
    }
}
