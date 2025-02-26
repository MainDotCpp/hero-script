pub mod hero;

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json;
use self::hero::HeroConfig;

// 导出HeroConfig以便app.rs可以引用
pub use self::hero::HeroConfig;

pub struct ConfigManager {
    config_path: String,
}

impl ConfigManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // 创建配置目录
        let config_dir = "config";
        if !Path::new(config_dir).exists() {
            fs::create_dir_all(config_dir)?;
        }
        
        Ok(Self {
            config_path: config_dir.to_string(),
        })
    }
    
    pub fn load_hero_configs(&self) -> Result<HashMap<String, HeroConfig>, Box<dyn std::error::Error>> {
        let mut configs = HashMap::new();
        let heroes_path = format!("{}/heroes", self.config_path);
        
        if !Path::new(&heroes_path).exists() {
            fs::create_dir_all(&heroes_path)?;
            return Ok(configs);
        }
        
        for entry in fs::read_dir(&heroes_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().unwrap_or_default() == "json" {
                let config_str = fs::read_to_string(&path)?;
                let config: HeroConfig = serde_json::from_str(&config_str)?;
                configs.insert(config.name.clone(), config);
            }
        }
        
        Ok(configs)
    }
} 