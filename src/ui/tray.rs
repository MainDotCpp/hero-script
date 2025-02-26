use std::sync::{Arc, Mutex};
use tray_item::{TrayItem, IconSource};
use crate::heroes::HeroRegistry;

pub struct TrayMenu {
    tray: TrayItem,
}

impl TrayMenu {
    pub fn new(
        hero_registry: Arc<Mutex<HeroRegistry>>,
        active_hero: Arc<Mutex<String>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut tray = TrayItem::new("LOL宏", IconSource::Resource("")).unwrap();
        
        // 退出选项
        tray.add_menu_item("退出", move || {
            std::process::exit(0);
        })?;
        
        // 由于tray-item库没有直接提供分隔符功能，我们使用一个空的菜单项作为视觉分隔
        tray.add_menu_item("---------------", || {})?;
        
        // 添加英雄切换菜单
        {
            let registry = hero_registry.lock().unwrap();
            let hero_names = registry.get_hero_names();
            
            for hero_name in hero_names {
                let hero_name_clone = hero_name.clone();
                let active_hero_clone = active_hero.clone();
                
                tray.add_menu_item(&hero_name, move || {
                    let mut active = active_hero_clone.lock().unwrap();
                    *active = hero_name_clone.clone();
                    println!("切换到英雄: {}", hero_name_clone);
                })?;
            }
        }
        
        Ok(Self {
            tray,
        })
    }
} 