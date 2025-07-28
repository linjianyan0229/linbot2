use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;
use libloading::{Library, Symbol};
use serde::{Serialize, Deserialize};

use crate::plugins::{Plugin, PluginInfo, PluginResult, PluginError};

/// 插件加载器
pub struct PluginLoader {
    /// 已加载的动态库
    libraries: HashMap<String, Library>,
}

impl PluginLoader {
    pub fn new() -> Self {
        Self {
            libraries: HashMap::new(),
        }
    }

    /// 加载插件
    pub async fn load_plugin(&mut self, plugin_path: &Path) -> PluginResult<Arc<dyn Plugin + Send + Sync>> {
        if plugin_path.is_file() {
            // 加载动态库插件
            self.load_dynamic_plugin(plugin_path).await
        } else if plugin_path.is_dir() {
            // 加载脚本插件或其他类型
            self.load_script_plugin(plugin_path).await
        } else {
            Err(PluginError::LoadError(format!("无效的插件路径: {}", plugin_path.display())))
        }
    }

    /// 加载动态库插件
    async fn load_dynamic_plugin(&mut self, lib_path: &Path) -> PluginResult<Arc<dyn Plugin + Send + Sync>> {
        unsafe {
            // 加载动态库
            let lib = Library::new(lib_path)
                .map_err(|e| PluginError::LoadError(format!("加载动态库失败: {}", e)))?;

            // 获取插件创建函数
            let create_plugin: Symbol<unsafe extern "C" fn() -> *mut dyn Plugin> = lib
                .get(b"create_plugin")
                .map_err(|e| PluginError::LoadError(format!("找不到create_plugin函数: {}", e)))?;

            // 创建插件实例
            let plugin_ptr = create_plugin();
            if plugin_ptr.is_null() {
                return Err(PluginError::LoadError("插件创建函数返回空指针".to_string()));
            }

            let plugin = Box::from_raw(plugin_ptr);

            // 保存动态库引用
            let lib_name = lib_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            self.libraries.insert(lib_name, lib);

            // 将Box转换为Arc
            // 由于trait object的限制，我们需要手动转换
            #[allow(unused_unsafe)]
            let arc_plugin: Arc<dyn Plugin + Send + Sync> = {
                // 将Box转换为原始指针，然后重新包装为Arc
                let raw_ptr = Box::into_raw(plugin);
                unsafe { Arc::from_raw(raw_ptr) }
            };
            Ok(arc_plugin)
        }
    }

    /// 加载脚本插件
    async fn load_script_plugin(&mut self, plugin_dir: &Path) -> PluginResult<Arc<dyn Plugin + Send + Sync>> {
        // 检查插件配置文件
        let config_file = plugin_dir.join("plugin.toml");
        if !config_file.exists() {
            return Err(PluginError::LoadError("找不到plugin.toml配置文件".to_string()));
        }

        // 读取插件配置
        let config_content = tokio::fs::read_to_string(&config_file).await?;
        let plugin_config: ScriptPluginConfig = toml::from_str(&config_content)
            .map_err(|e| PluginError::ConfigError(format!("解析插件配置失败: {}", e)))?;

        match plugin_config.plugin_type.as_str() {
            "python" => self.load_python_plugin(plugin_dir, &plugin_config).await,
            "javascript" => self.load_javascript_plugin(plugin_dir, &plugin_config).await,
            "lua" => self.load_lua_plugin(plugin_dir, &plugin_config).await,
            _ => Err(PluginError::LoadError(format!("不支持的插件类型: {}", plugin_config.plugin_type)))
        }
    }

    /// 加载Python插件
    async fn load_python_plugin(&mut self, _plugin_dir: &Path, _config: &ScriptPluginConfig) -> PluginResult<Arc<dyn Plugin + Send + Sync>> {
        // TODO: 实现Python插件加载
        Err(PluginError::LoadError("Python插件支持尚未实现".to_string()))
    }

    /// 加载JavaScript插件
    async fn load_javascript_plugin(&mut self, _plugin_dir: &Path, _config: &ScriptPluginConfig) -> PluginResult<Arc<dyn Plugin + Send + Sync>> {
        // TODO: 实现JavaScript插件加载
        Err(PluginError::LoadError("JavaScript插件支持尚未实现".to_string()))
    }

    /// 加载Lua插件
    async fn load_lua_plugin(&mut self, _plugin_dir: &Path, _config: &ScriptPluginConfig) -> PluginResult<Arc<dyn Plugin + Send + Sync>> {
        // TODO: 实现Lua插件加载
        Err(PluginError::LoadError("Lua插件支持尚未实现".to_string()))
    }

    /// 卸载插件
    #[allow(dead_code)]
    pub fn unload_plugin(&mut self, plugin_name: &str) -> PluginResult<()> {
        if let Some(_lib) = self.libraries.remove(plugin_name) {
            // 动态库会在drop时自动卸载
            Ok(())
        } else {
            Err(PluginError::PluginNotFound(plugin_name.to_string()))
        }
    }

    /// 获取已加载的插件列表
    #[allow(dead_code)]
    pub fn get_loaded_plugins(&self) -> Vec<String> {
        self.libraries.keys().cloned().collect()
    }
}

/// 脚本插件配置
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct ScriptPluginConfig {
    /// 插件信息
    pub info: PluginInfo,
    /// 插件类型 (python, javascript, lua等)
    pub plugin_type: String,
    /// 入口文件
    pub entry_point: String,
    /// 依赖列表
    pub dependencies: Vec<String>,
    /// 环境变量
    pub environment: HashMap<String, String>,
}

/// 插件验证器
#[allow(dead_code)]
pub struct PluginValidator;

impl PluginValidator {
    /// 验证插件文件
    #[allow(dead_code)]
    pub fn validate_plugin_file(plugin_path: &Path) -> PluginResult<()> {
        if !plugin_path.exists() {
            return Err(PluginError::LoadError("插件文件不存在".to_string()));
        }

        if plugin_path.is_file() {
            // 验证动态库文件
            Self::validate_dynamic_library(plugin_path)
        } else if plugin_path.is_dir() {
            // 验证脚本插件目录
            Self::validate_script_plugin_dir(plugin_path)
        } else {
            Err(PluginError::LoadError("无效的插件路径".to_string()))
        }
    }

    /// 验证动态库
    #[allow(dead_code)]
    fn validate_dynamic_library(lib_path: &Path) -> PluginResult<()> {
        // 检查文件扩展名
        let extension = lib_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let valid_extensions = if cfg!(target_os = "windows") {
            vec!["dll"]
        } else if cfg!(target_os = "macos") {
            vec!["dylib"]
        } else {
            vec!["so"]
        };

        if !valid_extensions.contains(&extension) {
            return Err(PluginError::LoadError(format!(
                "无效的动态库扩展名: {}, 期望: {:?}", 
                extension, 
                valid_extensions
            )));
        }

        // TODO: 添加更多验证，如数字签名验证等

        Ok(())
    }

    /// 验证脚本插件目录
    #[allow(dead_code)]
    fn validate_script_plugin_dir(plugin_dir: &Path) -> PluginResult<()> {
        // 检查必需的配置文件
        let config_file = plugin_dir.join("plugin.toml");
        if !config_file.exists() {
            return Err(PluginError::LoadError("缺少plugin.toml配置文件".to_string()));
        }

        // TODO: 验证配置文件内容
        // TODO: 检查入口文件是否存在
        // TODO: 验证依赖关系

        Ok(())
    }

    /// 验证插件信息
    #[allow(dead_code)]
    pub fn validate_plugin_info(info: &PluginInfo) -> PluginResult<()> {
        if info.name.is_empty() {
            return Err(PluginError::ConfigError("插件名称不能为空".to_string()));
        }

        if info.version.is_empty() {
            return Err(PluginError::ConfigError("插件版本不能为空".to_string()));
        }

        // 验证版本格式
        if !Self::is_valid_version(&info.version) {
            return Err(PluginError::ConfigError(format!("无效的版本格式: {}", info.version)));
        }

        // 验证API版本兼容性
        if !Self::is_compatible_api_version(&info.api_version) {
            return Err(PluginError::ConfigError(format!("不兼容的API版本: {}", info.api_version)));
        }

        Ok(())
    }

    /// 检查版本格式是否有效
    #[allow(dead_code)]
    fn is_valid_version(version: &str) -> bool {
        // 简单的语义化版本检查
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }

        parts.iter().all(|part| part.parse::<u32>().is_ok())
    }

    /// 检查API版本是否兼容
    #[allow(dead_code)]
    fn is_compatible_api_version(api_version: &str) -> bool {
        // 当前支持的API版本
        const SUPPORTED_API_VERSIONS: &[&str] = &["1.0.0"];
        
        SUPPORTED_API_VERSIONS.contains(&api_version)
    }
}

/// 插件依赖解析器
#[allow(dead_code)]
pub struct DependencyResolver {
    /// 已注册的插件
    registered_plugins: HashMap<String, PluginInfo>,
}

impl DependencyResolver {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            registered_plugins: HashMap::new(),
        }
    }

    /// 注册插件
    #[allow(dead_code)]
    pub fn register_plugin(&mut self, info: PluginInfo) {
        self.registered_plugins.insert(info.name.clone(), info);
    }

    /// 解析依赖关系
    #[allow(dead_code)]
    pub fn resolve_dependencies(&self, plugin_info: &PluginInfo) -> PluginResult<Vec<String>> {
        let mut resolved = Vec::new();
        let mut visited = std::collections::HashSet::new();

        self.resolve_dependencies_recursive(plugin_info, &mut resolved, &mut visited)?;

        Ok(resolved)
    }

    /// 递归解析依赖
    #[allow(dead_code)]
    fn resolve_dependencies_recursive(
        &self,
        plugin_info: &PluginInfo,
        resolved: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
    ) -> PluginResult<()> {
        if visited.contains(&plugin_info.name) {
            return Err(PluginError::LoadError(format!("检测到循环依赖: {}", plugin_info.name)));
        }

        visited.insert(plugin_info.name.clone());

        for dep_name in &plugin_info.dependencies {
            if let Some(dep_info) = self.registered_plugins.get(dep_name) {
                self.resolve_dependencies_recursive(dep_info, resolved, visited)?;
                if !resolved.contains(dep_name) {
                    resolved.push(dep_name.clone());
                }
            } else {
                return Err(PluginError::LoadError(format!("找不到依赖插件: {}", dep_name)));
            }
        }

        visited.remove(&plugin_info.name);
        Ok(())
    }

    /// 检查依赖是否满足
    #[allow(dead_code)]
    pub fn check_dependencies(&self, plugin_info: &PluginInfo) -> PluginResult<()> {
        for dep_name in &plugin_info.dependencies {
            if !self.registered_plugins.contains_key(dep_name) {
                return Err(PluginError::LoadError(format!("缺少依赖插件: {}", dep_name)));
            }
        }
        Ok(())
    }

    /// 获取插件的依赖树
    #[allow(dead_code)]
    pub fn get_dependency_tree(&self, plugin_name: &str) -> PluginResult<DependencyTree> {
        if let Some(plugin_info) = self.registered_plugins.get(plugin_name) {
            let mut tree = DependencyTree {
                name: plugin_name.to_string(),
                dependencies: Vec::new(),
            };

            for dep_name in &plugin_info.dependencies {
                let dep_tree = self.get_dependency_tree(dep_name)?;
                tree.dependencies.push(dep_tree);
            }

            Ok(tree)
        } else {
            Err(PluginError::PluginNotFound(plugin_name.to_string()))
        }
    }
}

/// 依赖树结构
#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyTree {
    pub name: String,
    pub dependencies: Vec<DependencyTree>,
}

impl DependencyTree {
    /// 获取所有依赖的扁平列表
    #[allow(dead_code)]
    pub fn flatten(&self) -> Vec<String> {
        let mut result = Vec::new();
        self.flatten_recursive(&mut result);
        result.sort();
        result.dedup();
        result
    }

    #[allow(dead_code)]
    fn flatten_recursive(&self, result: &mut Vec<String>) {
        for dep in &self.dependencies {
            result.push(dep.name.clone());
            dep.flatten_recursive(result);
        }
    }
}
