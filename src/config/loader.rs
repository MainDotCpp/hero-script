use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use serde_json;
use crate::error::AppResult;
use crate::config::hero::HeroConfig;

pub struct ConfigLoader {
    config_dir: PathBuf,
}

impl ConfigLoader {
    pub fn new() -> AppResult<Self> {
        let config_dir = Path::new("config").to_path_buf();
        
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        
        // 创建英雄配置目录
        let hero_config_dir = config_dir.join("heroes");
        if !hero_config_dir.exists() {
            fs::create_dir_all(&hero_config_dir)?;
        }
        
        Ok(Self {
            config_dir,
        })
    }
    
    pub fn save_hero_config(&self, hero: &HeroConfig) -> AppResult<()> {
        let hero_dir = self.config_dir.join("heroes");
        let file_path = hero_dir.join(format!("{}.json", hero.name));
        
        let json = serde_json::to_string_pretty(hero)?;
        fs::write(file_path, json)?;
        
        Ok(())
    }
    
    pub fn load_hero_config(&self, name: &str) -> AppResult<Option<HeroConfig>> {
        let file_path = self.config_dir.join("heroes").join(format!("{}.json", name));
        
        if !file_path.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(file_path)?;
        let config = serde_json::from_str(&content)?;
        
        Ok(Some(config))
    }
} 