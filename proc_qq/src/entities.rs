use crate::DeviceSource::JsonFile;

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
}
