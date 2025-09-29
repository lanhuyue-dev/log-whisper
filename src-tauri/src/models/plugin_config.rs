use serde::{Deserialize, Serialize};

/// 插件配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// 插件名称
    pub name: String,
    /// 插件描述
    pub description: String,
    /// 插件优先级
    pub priority: u32,
    /// 是否启用
    pub enabled: bool,
    /// 插件设置
    pub settings: PluginSettings,
}

/// 插件设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSettings {
    /// 是否启用自动检测
    pub auto_detect: bool,
    /// 最小置信度阈值
    pub min_confidence: f32,
    /// 最大处理行数
    pub max_lines: Option<usize>,
    /// 自定义正则表达式
    pub custom_patterns: Vec<String>,
}

impl Default for PluginSettings {
    fn default() -> Self {
        Self {
            auto_detect: true,
            min_confidence: 0.5,
            max_lines: None,
            custom_patterns: Vec::new(),
        }
    }
}

impl PluginConfig {
    /// 创建新的插件配置
    pub fn new(name: String, description: String, priority: u32) -> Self {
        Self {
            name,
            description,
            priority,
            enabled: true,
            settings: PluginSettings::default(),
        }
    }
    
    /// 设置是否启用
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    
    /// 设置插件设置
    pub fn with_settings(mut self, settings: PluginSettings) -> Self {
        self.settings = settings;
        self
    }
    
    /// 检查插件是否可用
    pub fn is_available(&self) -> bool {
        self.enabled
    }
}

/// 插件优先级枚举
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PluginPriority {
    /// 最高优先级
    Highest = 0,
    /// 高优先级
    High = 10,
    /// 中等优先级
    Medium = 50,
    /// 低优先级
    Low = 100,
    /// 最低优先级
    Lowest = 1000,
}

impl PluginPriority {
    /// 获取优先级数值
    pub fn value(&self) -> u32 {
        *self as u32
    }
}

/// 插件管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManager {
    /// 插件配置列表
    pub plugins: Vec<PluginConfig>,
    /// 默认插件
    pub default_plugin: String,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new() -> Self {
        let mut plugins = Vec::new();
        
        // 添加默认插件
        plugins.push(PluginConfig::new(
            "Auto".to_string(),
            "自动选择最佳插件".to_string(),
            PluginPriority::Highest.value(),
        ));
        
        plugins.push(PluginConfig::new(
            "MyBatis".to_string(),
            "MyBatis SQL 解析器".to_string(),
            PluginPriority::High.value(),
        ));
        
        plugins.push(PluginConfig::new(
            "JSON".to_string(),
            "JSON 修复和格式化".to_string(),
            PluginPriority::High.value(),
        ));
        
        plugins.push(PluginConfig::new(
            "Raw".to_string(),
            "原始文本显示".to_string(),
            PluginPriority::Lowest.value(),
        ));
        
        Self {
            plugins,
            default_plugin: "Auto".to_string(),
        }
    }
    
    /// 获取启用的插件
    pub fn get_enabled_plugins(&self) -> Vec<&PluginConfig> {
        self.plugins.iter().filter(|p| p.enabled).collect()
    }
    
    /// 根据名称获取插件
    pub fn get_plugin(&self, name: &str) -> Option<&PluginConfig> {
        self.plugins.iter().find(|p| p.name == name)
    }
    
    /// 根据名称获取插件（可变引用）
    pub fn get_plugin_mut(&mut self, name: &str) -> Option<&mut PluginConfig> {
        self.plugins.iter_mut().find(|p| p.name == name)
    }
    
    /// 启用插件
    pub fn enable_plugin(&mut self, name: &str) -> bool {
        if let Some(plugin) = self.get_plugin_mut(name) {
            plugin.enabled = true;
            true
        } else {
            false
        }
    }
    
    /// 禁用插件
    pub fn disable_plugin(&mut self, name: &str) -> bool {
        if let Some(plugin) = self.get_plugin_mut(name) {
            plugin.enabled = false;
            true
        } else {
            false
        }
    }
    
    /// 设置默认插件
    pub fn set_default_plugin(&mut self, name: &str) -> bool {
        if self.get_plugin(name).is_some() {
            self.default_plugin = name.to_string();
            true
        } else {
            false
        }
    }
    
    /// 获取插件列表（按优先级排序）
    pub fn get_plugins_by_priority(&self) -> Vec<&PluginConfig> {
        let mut plugins: Vec<&PluginConfig> = self.plugins.iter().collect();
        plugins.sort_by_key(|p| p.priority);
        plugins
    }
}
