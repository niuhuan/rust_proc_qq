use std::path::{Path, PathBuf};

/// 取临时文件目录
#[cfg(target_os = "macos")]
pub(crate) fn template_dir() -> String {
    "/tmp".to_owned()
}

/// 取临时文件目录
#[cfg(target_os = "linux")]
pub(crate) fn template_dir() -> String {
    "/tmp".to_owned()
}

/// 取临时文件目录
#[cfg(target_os = "windows")]
pub(crate) fn template_dir() -> String {
    std::env::temp_dir().to_str().unwrap().to_owned()
}

/// 连接路径
pub(crate) fn join_paths<P: AsRef<Path>>(paths: Vec<P>) -> String {
    match paths.len() {
        0 => String::default(),
        _ => {
            let mut path: PathBuf = PathBuf::new();
            for x in paths {
                path = path.join(x);
            }
            return path.to_str().unwrap().to_string();
        }
    }
}
