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
        // 尝试创建托盘但处理可能的图标错误
        let tray_result = TrayItem::new("LOL宏", IconSource::Resource("shell32.dll,1"));
        
        // 如果使用系统图标失败，则尝试不使用图标
        let mut tray = match tray_result {
            Ok(t) => t,
            Err(e) => {
                // 记录错误但继续尝试其他方法
                eprintln!("无法使用系统图标，尝试不使用图标: {}", e);
                
                // 尝试使用应用程序默认图标
                match TrayItem::new("LOL宏", IconSource::Resource("#32512")) {
                    Ok(t) => t,
                    Err(_) => {
                        // 如果仍然失败，则放弃使用图标
                        eprintln!("无法使用任何图标，创建无图标托盘");
                        
                        // 我们需要返回一个有效的TrayMenu
                        // 但由于tray-item库要求图标，我们创建一个模拟版本
                        return Ok(Self {
                            tray: create_fallback_tray()?,
                        });
                    }
                }
            }
        };
        
        // 退出选项
        tray.add_menu_item("退出", move || {
            std::process::exit(0);
        })?;
        
        // 添加分隔符替代品
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

// 创建一个备用的托盘实现
fn create_fallback_tray() -> Result<TrayItem, Box<dyn std::error::Error>> {
    // 在Windows上，我们尝试使用一个已知存在的系统图标
    // IDI_APPLICATION = 32512 是应用程序默认图标
    // 如果这个也失败，程序可能需要修改为不使用托盘
    let tray = TrayItem::new("LOL宏", IconSource::Resource("user32.dll,1"))?;
    Ok(tray)
} 