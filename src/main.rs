mod app;
mod keyboard;
mod config;
mod macro_engine;
mod heroes;
mod ui;
mod error;

use log::{info, error};
use error::AppResult;

fn main() {
    // 初始化日志
    env_logger::init();
    
    info!("LOL宏程序启动中...");
    
    // 启动应用
    match app::App::new() {
        Ok(mut app) => {
            if let Err(e) = app.run() {
                error!("应用运行错误: {}", e);
            }
        }
        Err(e) => {
            error!("应用初始化失败: {}", e);
        }
    }
} 