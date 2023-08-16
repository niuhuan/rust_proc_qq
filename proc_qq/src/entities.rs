use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use core::future::Future;
use ricq_core::msg::elem::{FlashImage, FriendImage, GroupImage};
use std::fmt::{Debug, Formatter};
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

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

#[async_trait]
pub trait ShowSliderTrait {
    async fn show_slider(&self, verify_url: Option<String>) -> Result<String>;
}

#[deprecated(since = "0.1.36", note = "please use `show_slider` instead")]
#[allow(non_snake_case)]
pub mod ShowSlider {
    pub use super::show_slider::*;
}

pub mod show_slider {
    use super::ShowSliderTrait;
    use anyhow::{Context, Result};
    use async_trait::async_trait;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;

    pub struct AndroidHelper;

    impl AndroidHelper {
        async fn http_get(&self, url: &str) -> Result<String> {
            Ok(reqwest::ClientBuilder::new().build().unwrap().get(url).header(
                "user-agent", "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.80 Mobile Safari/537.36",
            ).send().await?
                .text()
                .await?)
        }

        pub fn boxed() -> Box<dyn ShowSliderTrait + Sync + Send> {
            Box::new(Self)
        }

        pub fn arc_boxed() -> Arc<Box<dyn ShowSliderTrait + Sync + Send>> {
            Arc::new(Box::new(Self))
        }
    }

    #[async_trait]
    impl ShowSliderTrait for AndroidHelper {
        async fn show_slider(&self, verify_url: Option<String>) -> Result<String> {
            tracing::info!("滑动条 (原URL) : {:?}", verify_url);
            let helper_url = verify_url
                .clone()
                .with_context(|| "滑动条URL不存在")?
                .replace("ssl.captcha.qq.com", "txhelper.glitch.me");
            tracing::info!("滑动条 (改URL) : {:?}", helper_url);
            let mut txt = self
                .http_get(&helper_url)
                .await
                .with_context(|| "http请求失败")?;
            tracing::info!("您需要使用该仓库 提供的APP进行滑动 , 滑动后请等待, https://github.com/mzdluo123/TxCaptchaHelper : {}", txt);
            loop {
                sleep(Duration::from_secs(5)).await;
                let rsp = self
                    .http_get(&helper_url)
                    .await
                    .with_context(|| "http请求失败")?;
                if !rsp.eq(&txt) {
                    txt = rsp;
                    break;
                }
            }
            Ok(txt)
        }
    }

    #[cfg(all(any(target_os = "windows"), feature = "pop_window_slider"))]
    pub struct PopWindow;

    #[cfg(all(any(target_os = "windows"), feature = "pop_window_slider"))]
    impl PopWindow {
        pub fn boxed() -> Box<dyn ShowSliderTrait + Sync + Send> {
            Box::new(Self)
        }

        pub fn arc_boxed() -> Arc<Box<dyn ShowSliderTrait + Sync + Send>> {
            Arc::new(Box::new(Self))
        }
    }

    #[cfg(all(any(target_os = "windows"), feature = "pop_window_slider"))]
    #[async_trait]
    impl ShowSliderTrait for PopWindow {
        async fn show_slider(&self, verify_url: Option<String>) -> Result<String> {
            crate::captcha_window::ticket(verify_url.as_ref().with_context(|| "滑动条URL不存在")?)
                .with_context(|| "ticket未获取到")
        }
    }
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

pub enum ImageElement {
    GroupImage(GroupImage),
    FriendImage(FriendImage),
    FlashImage(FlashImage),
}

macro_rules! image_element_get {
    ($name:ident, $ty:ty) => {
        impl ImageElement {
            pub fn $name(&self) -> $ty {
                match self {
                    ImageElement::GroupImage(image) => image.$name,
                    ImageElement::FriendImage(image) => image.$name,
                    ImageElement::FlashImage(image) => match image {
                        FlashImage::FriendImage(image) => image.$name,
                        FlashImage::GroupImage(image) => image.$name,
                    },
                }
            }
        }
    };
}

image_element_get!(width, u32);
image_element_get!(height, u32);
image_element_get!(size, u32);

impl ImageElement {
    pub fn url(&self) -> String {
        match self {
            ImageElement::GroupImage(image) => image.url(),
            ImageElement::FriendImage(image) => image.url(),
            ImageElement::FlashImage(image) => match image {
                FlashImage::FriendImage(image) => image.url(),
                FlashImage::GroupImage(image) => image.url(),
            },
        }
    }

    pub fn md5(&self) -> Vec<u8> {
        match self {
            ImageElement::GroupImage(image) => image.md5.clone(),
            ImageElement::FriendImage(image) => image.md5.clone(),
            ImageElement::FlashImage(image) => match image {
                FlashImage::FriendImage(image) => image.md5.clone(),
                FlashImage::GroupImage(image) => image.md5.clone(),
            },
        }
    }

    pub fn is_flash(&self) -> bool {
        match self {
            ImageElement::FlashImage(_) => true,
            _ => false,
        }
    }

    pub fn case_flash(&self) -> Result<&'_ FlashImage> {
        match self {
            ImageElement::FlashImage(image) => Ok(&image),
            _ => Err(anyhow::Error::msg("mismatching")),
        }
    }

    pub fn into_flash(self) -> Result<FlashImage> {
        match self {
            ImageElement::FlashImage(image) => Ok(image),
            _ => Err(anyhow::Error::msg("mismatching")),
        }
    }

    pub fn is_group(&self) -> bool {
        match self {
            ImageElement::GroupImage(_) => true,
            ImageElement::FriendImage(_) => false,
            ImageElement::FlashImage(image) => match image {
                FlashImage::FriendImage(_) => false,
                FlashImage::GroupImage(_) => true,
            },
        }
    }

    pub fn case_group(&self) -> Result<&'_ GroupImage> {
        match self {
            ImageElement::GroupImage(image) => Ok(&image),
            ImageElement::FriendImage(_) => Err(anyhow::Error::msg("mismatching")),
            ImageElement::FlashImage(image) => match image {
                FlashImage::FriendImage(_) => Err(anyhow::Error::msg("mismatching")),
                FlashImage::GroupImage(image) => Ok(&image),
            },
        }
    }

    pub fn into_group(self) -> Result<GroupImage> {
        match self {
            ImageElement::GroupImage(image) => Ok(image),
            ImageElement::FriendImage(_) => Err(anyhow::Error::msg("mismatching")),
            ImageElement::FlashImage(image) => match image {
                FlashImage::FriendImage(_) => Err(anyhow::Error::msg("mismatching")),
                FlashImage::GroupImage(image) => Ok(image),
            },
        }
    }

    pub fn is_friend(&self) -> bool {
        match self {
            ImageElement::GroupImage(_) => false,
            ImageElement::FriendImage(_) => true,
            ImageElement::FlashImage(image) => match image {
                FlashImage::FriendImage(_) => true,
                FlashImage::GroupImage(_) => false,
            },
        }
    }

    pub fn case_friend(&self) -> Result<&'_ FriendImage> {
        match self {
            ImageElement::GroupImage(_) => Err(anyhow::Error::msg("mismatching")),
            ImageElement::FriendImage(image) => Ok(&image),
            ImageElement::FlashImage(image) => match image {
                FlashImage::FriendImage(image) => Ok(&image),
                FlashImage::GroupImage(_) => Err(anyhow::Error::msg("mismatching")),
            },
        }
    }

    pub fn into_friend(self) -> Result<FriendImage> {
        match self {
            ImageElement::GroupImage(_) => Err(anyhow::Error::msg("mismatching")),
            ImageElement::FriendImage(image) => Ok(image),
            ImageElement::FlashImage(image) => match image {
                FlashImage::FriendImage(image) => Ok(image),
                FlashImage::GroupImage(_) => Err(anyhow::Error::msg("mismatching")),
            },
        }
    }
}
