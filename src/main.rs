mod app;
mod keyboard;
mod config;
mod macro_engine;
mod heroes;
mod error;

use log::{info, error, debug, LevelFilter};
use env_logger::Builder;

fn main() {
    // 设置更详细的日志级别
    let mut builder = Builder::new();
    builder
        .filter_level(LevelFilter::Debug)  // 设置为Debug级别，显示更多信息
        .init();
    
    info!("LOL宏程序启动中...");
    debug!("调试模式已启用");
    
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