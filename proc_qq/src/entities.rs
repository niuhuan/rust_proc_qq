use crate::DeviceSource::JsonFile;
use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use core::future::Future;
use std::fmt::{Debug, Formatter};
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum DeviceSource {
    JsonFile(String),
    JsonString(String),
}

impl DeviceSource {
    pub fn default() -> Self {
        JsonFile("device.json".to_owned())
    }
}

#[derive(Clone)]
pub enum Authentication {
    QRCode,
    UinPassword(i64, String),
    UinPasswordMd5(i64, [u8; 16]),
    CustomUinPassword(Arc<Box<dyn CustomUinPassword + Sync + Send>>),
    CustomUinPasswordMd5(Arc<Box<dyn CustomUinPasswordMd5 + Sync + Send>>),
    CallBack(CallBackWrapper),
    Abandon,
}

#[derive(Clone)]
pub struct CallBackWrapper {
    pub callback: Pin<Box<fn(Arc<ricq::Client>) -> Authentication>>,
}

unsafe impl Send for CallBackWrapper {}
unsafe impl Sync for CallBackWrapper {}

impl Debug for CallBackWrapper {
    fn fmt(&self, mut f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(&mut f, "回调函数决定返回决定是放弃登录, 还是扫码, 还是密码")
    }
}

impl CallBackWrapper {
    pub fn new(callback: fn(Arc<ricq::Client>) -> Authentication) -> Self {
        CallBackWrapper {
            callback: Pin::new(Box::new(callback)),
        }
    }
}

#[async_trait]
pub trait CustomUinPassword {
    async fn input_uin(&self) -> Result<i64>;
    async fn input_password(&self) -> Result<String>;
}

#[async_trait]
pub trait CustomUinPasswordMd5 {
    async fn input_uin(&self) -> Result<i64>;
    async fn input_password_md5(&self) -> Result<[u8; 16]>;
}

#[derive(Clone, Debug)]
pub enum ShowQR {
    OpenBySystem,
    #[cfg(feature = "console_qr")]
    PrintToConsole,
    Custom(Pin<Box<fn(Bytes) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>>>),
    SaveToFile,
}

#[derive(Clone, Debug)]
pub enum ShowSlider {
    AndroidHelper,

    #[cfg(all(any(target_os = "windows"), feature = "pop_window_slider"))]
    PopWindow,
}

#[derive(Clone)]
pub enum DeviceLockVerification {
    Url,
    Sms(Arc<Box<dyn Supplier<String> + Sync + Send>>),
}

#[async_trait]
pub trait Supplier<T> {
    async fn get(&self) -> Result<T>;
}

#[async_trait]
pub trait SessionStore {
    async fn save_session(&self, data: Vec<u8>) -> Result<()>;
    async fn load_session(&self) -> Result<Option<Vec<u8>>>;
    async fn remove_session(&self) -> Result<()>;
}

pub struct FileSessionStore {
    pub path: String,
}

impl FileSessionStore {
    pub fn boxed(path: impl Into<String>) -> Box<dyn SessionStore + Send + Sync> {
        return Box::new(Self { path: path.into() });
    }
}

#[async_trait]
impl SessionStore for FileSessionStore {
    async fn save_session(&self, data: Vec<u8>) -> Result<()> {
        tokio::fs::write(self.path.as_str(), data).await?;
        Ok(())
    }
    async fn load_session(&self) -> Result<Option<Vec<u8>>> {
        if Path::new(self.path.as_str()).exists() {
            Ok(Some(tokio::fs::read(self.path.as_str()).await?))
        } else {
            Ok(None)
        }
    }
    async fn remove_session(&self) -> Result<()> {
        let _ = tokio::fs::remove_file(self.path.as_str()).await;
        Ok(())
    }
}
