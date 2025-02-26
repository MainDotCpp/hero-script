use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO错误: {0}")]
    Io(#[from] io::Error),
    
    #[error("配置错误: {0}")]
    Config(String),
    
    #[error("键盘监听器错误: {0}")]
    KeyboardListener(String),
    
    #[error("系统错误: {0}")]
    System(String),
    
    #[error("JSON错误: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("未知错误: {0}")]
    Unknown(String),
}

pub type AppResult<T> = Result<T, AppError>; 