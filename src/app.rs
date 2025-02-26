use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use log::{info, error};
use std::io::{self, Write};
use std::thread;
use winapi::um::winuser::{GetMessageA, TranslateMessage, DispatchMessageA, MSG};
use winapi::shared::minwindef::BOOL;

use crate::keyboard::{KeyboardListener, KeyboardSimulator};
use crate::config::ConfigManager;
use crate::macro_engine::MacroEngine;
use crate::heroes::HeroRegistry;

pub struct App {
    runtime: Runtime,
    keyboard_listener: Arc<KeyboardListener>,
    keyboard_simulator: Arc<KeyboardSimulator>,
    config_manager: Arc<Mutex<ConfigManager>>,
    macro_engine: Arc<MacroEngine>,
    hero_registry: Arc<Mutex<HeroRegistry>>,
    active_hero: Arc<Mutex<String>>,
    running: Arc<Mutex<bool>>,
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
        
        // 程序运行标志
        let running = Arc::new(Mutex::new(true));
        
        Ok(Self {
            runtime,
            keyboard_listener,
            keyboard_simulator,
            config_manager,
            macro_engine,
            hero_registry,
            active_hero,
            running,
        })
    }
    
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("启动消息循环...");
        Self::run_message_loop();
        Ok(())
    }
    
    fn command_line_interface(
        active_hero: Arc<Mutex<String>>, 
        hero_registry: Arc<Mutex<HeroRegistry>>,
        running: Arc<Mutex<bool>>,  // 这里使用的是引用计数指针，不会拥有值
    ) {
        let mut input = String::new();
        
        while let Ok(is_running) = running.lock() {
            if !*is_running {
                break;
            }
            
            // 显示当前英雄
            if let Ok(hero) = active_hero.lock() {
                print!("\n当前英雄: {} > ", hero);
                io::stdout().flush().unwrap();
            }
            
            // 读取命令
            input.clear();
            if io::stdin().read_line(&mut input).is_ok() {
                let command = input.trim();
                
                // 处理命令
                match command {
                    "quit" | "exit" | "q" => {
                        println!("正在退出...");
                        if let Ok(mut is_running) = running.lock() {
                            *is_running = false;
                        }
                        break;
                    },
                    "list" | "ls" | "l" => {
                        // 显示可用英雄列表
                        if let Ok(registry) = hero_registry.lock() {
                            let heroes = registry.get_hero_names();
                            println!("可用英雄列表:");
                            for (i, name) in heroes.iter().enumerate() {
                                println!("{}) {}", i + 1, name);
                            }
                        }
                    },
                    s if s.starts_with("switch ") || s.starts_with("s ") => {
                        let parts: Vec<&str> = s.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let hero_name = parts[1];
                            
                            // 检查英雄是否存在
                            let hero_exists = {
                                if let Ok(registry) = hero_registry.lock() {
                                    registry.get_hero(hero_name).is_some()
                                } else {
                                    false
                                }
                            };
                            
                            if hero_exists {
                                if let Ok(mut curr_hero) = active_hero.lock() {
                                    *curr_hero = hero_name.to_string();
                                    println!("已切换到英雄: {}", hero_name);
                                    println!("\n使用说明:");
                                    println!("1. 这是一个全局键盘宏，请在游戏窗口中按键，不是在这里输入");
                                    println!("2. 对于亚索(yasuo):");
                                    println!("   - 先按E，然后在150ms内按R：将触发EQR连招");
                                    println!("   - 先按E，然后在300ms内按D：将触发EQD连招");
                                    println!("3. 对于瑞文(riven):");
                                    println!("   - 请参考瑞文专用连招说明");
                                }
                            } else {
                                println!("错误: 英雄 '{}' 不存在", hero_name);
                            }
                        } else {
                            println!("用法: switch <英雄名称>");
                        }
                    },
                    "help" | "h" | "?" => {
                        println!("可用命令:");
                        println!("  list (ls, l)       - 显示所有可用英雄");
                        println!("  switch (s) <英雄>  - 切换到指定英雄");
                        println!("  help (h, ?)        - 显示帮助信息");
                        println!("  quit (q, exit)     - 退出程序");
                    },
                    "" => {
                        // 忽略空行
                    },
                    _ => {
                        println!("未知命令: {}. 输入 'help' 查看可用命令", command);
                    }
                }
            }
        }
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

    fn run_message_loop() {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageA(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
        }
    }
} 