use crate::DeviceSource::JsonFile;
use std::future::Future;
use std::pin::Pin;

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
