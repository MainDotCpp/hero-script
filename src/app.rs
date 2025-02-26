use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use log::{info, error, debug};

use crate::keyboard::{KeyboardListener, KeyboardSimulator};
use crate::config::{ConfigManager, hero::HeroConfig};
use crate::macro_engine::MacroEngine;
use crate::heroes::HeroRegistry;
use crate::ui::TrayMenu;

pub struct App {
    runtime: Runtime,
    keyboard_listener: Arc<KeyboardListener>,
    keyboard_simulator: Arc<KeyboardSimulator>,
    config_manager: Arc<Mutex<ConfigManager>>,
    macro_engine: Arc<MacroEngine>,
    hero_registry: Arc<Mutex<HeroRegistry>>,
    tray_menu: Option<TrayMenu>,
    active_hero: Arc<Mutex<String>>,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // 创建异步运行时
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        
        // 初始化配置管理器
        let config_manager = Arc::new(Mutex::new(ConfigManager::new()?));
        
        // 初始化键盘监听和模拟器
        let keyboard_simulator = Arc::new(KeyboardSimulator::new());
        let active_hero = Arc::new(Mutex::new(String::from("default")));
        
        // 初始化英雄注册表
        let hero_registry = Arc::new(Mutex::new(HeroRegistry::new()));
        
        // 初始化宏引擎
        let macro_engine = Arc::new(MacroEngine::new(
            keyboard_simulator.clone(),
            hero_registry.clone(),
            active_hero.clone(),
        ));
        
        // 初始化键盘监听器
        let keyboard_listener = Arc::new(KeyboardListener::new(macro_engine.clone())?);
        
        // 创建系统托盘
        let tray_menu = Some(TrayMenu::new(hero_registry.clone(), active_hero.clone())?);
        
        Ok(Self {
            runtime,
            keyboard_listener,
            keyboard_simulator,
            config_manager,
            macro_engine,
            hero_registry,
            tray_menu,
            active_hero,
        })
    }
    
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("加载英雄配置...");
        self.load_heroes()?;
        
        info!("启动键盘监听器...");
        self.keyboard_listener.start()?;
        
        info!("应用程序已启动，按Ctrl+Shift+F12切换英雄");
        
        // 进入主事件循环
        self.runtime.block_on(async {
            // 等待程序结束信号
            tokio::signal::ctrl_c().await.unwrap();
            info!("收到退出信号，正在关闭程序...");
        });
        
        // 清理资源
        self.keyboard_listener.stop()?;
        
        info!("程序已退出");
        Ok(())
    }
    
    fn load_heroes(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 从配置文件加载英雄配置
        let configs = self.config_manager.lock().unwrap().load_hero_configs()?;
        
        // 注册英雄到注册表
        let mut registry = self.hero_registry.lock().unwrap();
        
        // 注册预定义英雄
        registry.register_hero("yasuo", crate::heroes::yasuo::create_config());
        registry.register_hero("riven", crate::heroes::riven::create_config());
        
        // 注册自定义英雄
        for (name, config) in configs {
            registry.register_hero(&name, config);
        }
        
        info!("已加载 {} 个英雄配置", registry.get_hero_names().len());
        
        Ok(())
    }
} 