/// 此模块用于重新导出引入, 以便macros使用
pub use client::*;
pub use entities::*;
pub use handler::*;
pub use proc_qq_codegen::*;
pub use traits::*;

#[cfg(any(target_os = "windows"))]
mod captcha_window;
mod client;
mod entities;
mod handler;
pub mod re_exports;
mod traits;
