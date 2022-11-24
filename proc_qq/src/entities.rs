use crate::DeviceSource::JsonFile;
use std::fmt::{Debug, Formatter};
use std::future::Future;
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

#[derive(Debug, Clone)]
pub enum Authentication {
    QRCode,
    UinPassword(i64, String),
    UinPasswordMd5(i64, [u8; 16]),
    CustomUinPassword(CustomUinPassword),
    CustomUinPasswordMd5(CustomUinPasswordMd5),
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

#[derive(Debug, Clone)]
pub struct CustomUinPassword {
    pub input_uin: Pin<Box<fn() -> Pin<Box<dyn Future<Output = anyhow::Result<i64>> + Send>>>>,
    pub input_password:
        Pin<Box<fn() -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>>>>,
}

#[derive(Debug, Clone)]
pub struct CustomUinPasswordMd5 {
    pub input_uin: Pin<Box<fn() -> Pin<Box<dyn Future<Output = anyhow::Result<i64>> + Send>>>>,
    pub input_password_md5:
        Pin<Box<fn() -> Pin<Box<dyn Future<Output = anyhow::Result<[u8; 16]>> + Send>>>>,
}
