use crate::handler::EventSender;
use crate::DeviceSource::{JsonFile, JsonString};
use crate::{Authentication, ClientHandler, DeviceSource, EventResultHandler, Module};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use core::future::Future;
use futures::future::BoxFuture;
use futures::FutureExt;
use rand::prelude::IteratorRandom;
use ricq::ext::common::after_login;
use ricq_core::binary::{BinaryReader, BinaryWriter};
use ricq_core::command::wtlogin::{
    LoginDeviceLocked, LoginNeedCaptcha, LoginResponse, LoginSuccess, LoginUnknownStatus,
    QRCodeConfirmed, QRCodeImageFetch, QRCodeState,
};
use ricq_core::protocol::device::Device;
use ricq_core::protocol::version::{Version, ANDROID_PHONE};
use ricq_core::{RQError, RQResult, Token};
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::time::sleep;

#[cfg(feature = "connect_handler")]
use crate::features::connect_handler::ConnectionHandler;
#[cfg(feature = "connect_handler")]
use std::ops::Deref;

pub struct Client {
    pub rq_client: Arc<ricq::Client>,
    pub authentication: Authentication,
    pub session_store: Arc<Option<Box<dyn SessionStore + Sync + Send>>>,
    pub(crate) modules: Arc<Vec<Module>>,
    pub(crate) result_handlers: Arc<Vec<EventResultHandler>>,
    pub show_qr: ShowQR,
    pub show_slider: ShowSlider,
    pub shutting: bool,
    pub device_lock_verification: DeviceLockVerification,
    #[cfg(feature = "connect_handler")]
    pub connection_handler: Arc<Option<Box<dyn ConnectionHandler + Sync + Send>>>,
    pub reconnect_duration: Duration,
}

impl Client {
    pub async fn write_token_to_store(&self) -> Result<()> {
        if let Some(session_store) = self.session_store.as_deref() {
            session_store
                .save_session(token_to_bytes(&self.rq_client.gen_token().await).to_vec())
                .await
        } else {
            Ok(())
        }
    }
}

pub async fn run_client(c: Arc<Client>) -> Result<()> {
    // todo // max try count
    loop {
        match run_client_once(c.clone()).await {
            Ok(_) => {}
            Err(err) => tracing::warn!("连接已断开 {:?}", err),
        };
        match c.authentication {
            Authentication::QRCode => {
                tracing::info!("QR验证模式下不进行重新登录");
                return Err(anyhow!("exit disconnect"));
            }
            _ => {}
        }
        tracing::info!("五秒钟之后重试");
        sleep(c.reconnect_duration).await;
    }
}

pub async fn run_client_once(client: Arc<Client>) -> Result<()> {
    // connect to server
    let addresses = client.rq_client.get_address_list().await;
    let address = addresses
        .into_iter()
        .choose_stable(&mut rand::thread_rng())
        .unwrap();
    let rq_client = client.rq_client.clone();
    #[cfg(feature = "connect_handler")]
    let handle = if let Some(handler) = client.connection_handler.deref() {
        let stream = handler
            .connect(address)
            .await
            .with_context(|| "failed connect to server")?;
        tokio::spawn(async move { rq_client.start(Pin::new(stream)).await })
    } else {
        let stream = TcpStream::connect(address)
            .await
            .with_context(|| "failed connect to server")?;
        tokio::spawn(async move { rq_client.start(stream).await })
    };
    #[cfg(not(feature = "connect_handler"))]
    let handle = {
        let stream = TcpStream::connect(address)
            .await
            .with_context(|| "failed connect to server")?;
        tokio::spawn(async move { rq_client.start(stream).await })
    };
    tokio::task::yield_now().await;
    // token login if allow and file exists
    if !token_login(&client).await {
        // authentication if token login failed or not set
        // The error of login failure is fatal
        login_authentication(&client).await?;
    }
    // Reference RICQ docs, this function must be called after login is completed, maybe it's to register the device.
    after_login(&client.rq_client.clone()).await;
    // save session, IO errors are fatal.
    client.write_token_to_store().await?;
    let event_sender = crate::handler::EventSender {
        modules: client.modules.clone(),
        result_handlers: client.result_handlers.clone(),
    };
    loop_events(handle, event_sender).await
}

async fn loop_events(handle: JoinHandle<()>, event_sender: EventSender) -> Result<()> {
    let _ = event_sender.send_connected_and_online().await;
    let result = handle.await;
    let _ = event_sender.send_disconnected_and_offline().await;
    result.with_context(|| "handle await error")?;
    Ok(())
}

async fn token_login(client: &Client) -> bool {
    if let Some(session_file) = client.session_store.as_deref() {
        let session_data = match session_file.load_session().await {
            Ok(data) => data,
            Err(err) => {
                tracing::info!("{:?}", err);
                return false;
            }
        };
        if let Some(session_data) = session_data {
            let result = client
                .rq_client
                .token_login(bytes_to_token(session_data))
                .await;
            match result {
                Ok(_) => true,
                Err(err) => match err {
                    RQError::TokenLoginFailed => {
                        // token error (KickedOffline)
                        let _ = session_file.remove_session().await;
                        false
                    }
                    _ => false,
                },
            }
        } else {
            false
        }
    } else {
        false
    }
}

async fn login_authentication(client: &Client) -> Result<()> {
    authenticate(&client.authentication, client).await
}

fn authenticate<'a>(
    authentication: &'a Authentication,
    client: &'a Client,
) -> BoxFuture<'a, Result<()>> {
    async move {
        let rq_client = client.rq_client.clone();
        match authentication {
            Authentication::QRCode => qr_login(client, client.show_qr.clone()).await,
            Authentication::UinPassword(uin, password) => {
                let first = rq_client.password_login(uin.clone(), password).await;
                loop_login(client, first).await
            }
            Authentication::UinPasswordMd5(uin, password) => {
                let first = rq_client.password_md5_login(uin.clone(), password).await;
                loop_login(client, first).await
            }
            Authentication::CustomUinPassword(cup) => {
                let uin = (cup.input_uin)().await?;
                let password = (cup.input_password)().await?;
                let first = rq_client.password_login(uin, &password).await;
                loop_login(client, first).await
            }
            Authentication::CustomUinPasswordMd5(cup) => {
                let uin = (cup.input_uin)().await?;
                let password = (cup.input_password_md5)().await?;
                let first = rq_client.password_md5_login(uin, &password).await;
                loop_login(client, first).await
            }
            Authentication::CallBack(wrapper) => {
                let callback_authentication = (wrapper.clone().callback)(rq_client);
                match callback_authentication {
                    Authentication::CallBack(_) => {
                        Err(anyhow::Error::msg("登录失败: 嵌套的回调函数"))
                    }
                    Authentication::Abandon => Err(anyhow::Error::msg("放弃登录")),
                    _ => authenticate(&authentication, client).await,
                }
            }
            Authentication::Abandon => Err(anyhow::Error::msg("放弃登录")),
        }
    }
    .boxed()
}

async fn qr_login(client: &Client, show_qr: ShowQR) -> Result<()> {
    let rq_client = client.rq_client.clone();
    let mut image_sig = Bytes::new();
    let mut resp = rq_client
        .fetch_qrcode()
        .await
        .with_context(|| "二维码加载失败")?;
    loop {
        match resp {
            QRCodeState::ImageFetch(QRCodeImageFetch {
                ref image_data,
                ref sig,
            }) => {
                image_sig = sig.clone();
                // todo 桌面环境直接打开, 服务器使用文字渲染
                match show_qr {
                    ShowQR::OpenBySystem => {
                        tokio::fs::write("qrcode.png", &image_data)
                            .await
                            .with_context(|| "failed to write file")?;
                        tracing::info!("二维码被写入文件: qrcode.png");
                        #[cfg(any(
                            target_os = "windows",
                            target_os = "linux",
                            target_os = "macos"
                        ))]
                        match opener::open("qrcode.png") {
                            Ok(_) => tracing::info!("已打开二维码图片, 请扫码"),
                            Err(_) => tracing::warn!("未能打开二维码图片, 请手动扫码"),
                        }
                        #[cfg(not(any(
                            target_os = "windows",
                            target_os = "linux",
                            target_os = "macos"
                        )))]
                        tracing::info!("当前环境不支持打开图片, 请手动扫码");
                    }
                    #[cfg(feature = "console_qr")]
                    ShowQR::PrintToConsole => {
                        if let Err(err) = print_qr_to_console(image_data) {
                            return Err(anyhow!("二维码打印到控制台时出现误 : {}", err));
                        }
                        tracing::info!("请扫码");
                    }
                    ShowQR::Custom(ref func) => {
                        tracing::info!("使用自定义二维码打印");
                        func(image_data.clone()).await?;
                    }
                    ShowQR::SaveToFile => {
                        tokio::fs::write("qrcode.png", &image_data)
                            .await
                            .with_context(|| "failed to write file")?;
                        tracing::info!("二维码被写入文件: qrcode.png, 请扫码");
                    }
                }
            }
            QRCodeState::WaitingForScan => {
                // tracing::info!("二维码待扫描")
            }
            QRCodeState::WaitingForConfirm => {
                // tracing::info!("二维码待确认")
            }
            QRCodeState::Timeout => {
                tracing::info!("二维码已超时，重新获取");
                resp = rq_client
                    .fetch_qrcode()
                    .await
                    .with_context(|| "二维码加载失败")?;
                continue;
            }
            QRCodeState::Confirmed(QRCodeConfirmed {
                ref tmp_pwd,
                ref tmp_no_pic_sig,
                ref tgt_qr,
                ..
            }) => {
                tracing::info!("二维码已确认");
                let first = rq_client
                    .qrcode_login(tmp_pwd, tmp_no_pic_sig, tgt_qr)
                    .await;
                return loop_login(client, first).await;
            }
            QRCodeState::Canceled => {
                return Err(anyhow::Error::msg("二维码已取消"));
            }
        }
        sleep(Duration::from_secs(5)).await;
        resp = rq_client
            .query_qrcode_result(&image_sig)
            .await
            .with_context(|| "二维码状态加载失败")?;
    }
}

async fn loop_login(client: &Client, first: RQResult<LoginResponse>) -> Result<()> {
    let rq_client = client.rq_client.clone();
    let mut resp = first.unwrap();
    loop {
        match resp {
            LoginResponse::Success(LoginSuccess {
                ref account_info, ..
            }) => {
                tracing::info!("登录成功: {:?}", account_info);
                return Ok(());
            }
            LoginResponse::DeviceLocked(LoginDeviceLocked {
                ref sms_phone,
                ref verify_url,
                ref message,
                ..
            }) => {
                tracing::info!("设备锁 : {:?}", message);
                tracing::info!("密保手机 : {:?}", sms_phone);
                tracing::info!("验证地址 : {:?}", verify_url);
                match &client.device_lock_verification {
                    &DeviceLockVerification::Url => {
                        qr2term::print_qr(
                            verify_url
                                .clone()
                                .with_context(|| "未能取得设备锁验证地址")?
                                .as_str(),
                        )?;
                        tracing::info!("验证地址 : {:?}", verify_url);
                        tracing::info!("手机扫码或者打开url，处理完成后重启程序");
                        std::process::exit(0);
                    }
                    &DeviceLockVerification::Sms => resp = rq_client.request_sms().await?,
                }
            }
            LoginResponse::NeedCaptcha(LoginNeedCaptcha {
                ref verify_url,
                // 图片应该没了
                image_captcha: ref _image_captcha,
                ..
            }) => match client.show_slider {
                ShowSlider::AndroidHelper => {
                    tracing::info!("滑动条 (原URL) : {:?}", verify_url);
                    let helper_url = verify_url
                        .clone()
                        .unwrap()
                        .replace("ssl.captcha.qq.com", "txhelper.glitch.me");
                    tracing::info!("滑动条 (改URL) : {:?}", helper_url);
                    let mut txt = http_get(&helper_url)
                        .await
                        .with_context(|| "http请求失败")?;
                    tracing::info!("您需要使用该仓库 提供的APP进行滑动 , 滑动后请等待, https://github.com/mzdluo123/TxCaptchaHelper : {}", txt);
                    loop {
                        sleep(Duration::from_secs(5)).await;
                        let rsp = http_get(&helper_url)
                            .await
                            .with_context(|| "http请求失败")?;
                        if !rsp.eq(&txt) {
                            txt = rsp;
                            break;
                        }
                    }
                    tracing::info!("获取到ticket : {}", txt);
                    resp = rq_client.submit_ticket(&txt).await.expect("发送ticket失败");
                }
                #[cfg(all(any(target_os = "windows"), feature = "pop_window_slider"))]
                ShowSlider::PopWindow => {
                    if let Some(ticket) =
                        crate::captcha_window::ticket(verify_url.as_ref().unwrap())
                    {
                        resp = rq_client
                            .submit_ticket(&ticket)
                            .await
                            .expect("failed to submit ticket");
                    } else {
                        panic!("not slide");
                    }
                }
            },
            LoginResponse::DeviceLockLogin { .. } => {
                resp = rq_client
                    .device_lock_login()
                    .await
                    .with_context(|| "设备锁登录失败")?;
            }
            LoginResponse::AccountFrozen => {
                return Err(anyhow::Error::msg("账户被冻结"));
            }
            LoginResponse::TooManySMSRequest => {
                return Err(anyhow::Error::msg("短信请求过于频繁"));
            }
            LoginResponse::UnknownStatus(LoginUnknownStatus {
                ref status,
                ref tlv_map,
                message,
                ..
            }) => {
                return Err(anyhow::Error::msg(format!(
                    "不能解析的登录响应: {:?}, {:?}, {:?}",
                    status, tlv_map, message,
                )));
            }
        }
    }
}

async fn http_get(url: &str) -> Result<String> {
    Ok(reqwest::ClientBuilder::new().build().unwrap().get(url).header(
        "user-agent", "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.80 Mobile Safari/537.36",
    ).send().await?
        .text()
        .await?)
}

pub fn token_to_bytes(t: &Token) -> Bytes {
    let mut token = BytesMut::with_capacity(1024);
    token.put_i64(t.uin);
    token.write_bytes_short(&t.d2);
    token.write_bytes_short(&t.d2key);
    token.write_bytes_short(&t.tgt);
    token.write_bytes_short(&t.srm_token);
    token.write_bytes_short(&t.t133);
    token.write_bytes_short(&t.encrypted_a1);
    token.write_bytes_short(&t.wt_session_ticket_key);
    token.write_bytes_short(&t.out_packet_session_id);
    token.write_bytes_short(&t.tgtgt_key);
    token.freeze()
}

pub fn bytes_to_token(token: Vec<u8>) -> Token {
    let mut t = Bytes::from(token);
    Token {
        uin: t.get_i64(),
        d2: t.read_bytes_short().to_vec(),
        d2key: t.read_bytes_short().to_vec(),
        tgt: t.read_bytes_short().to_vec(),
        srm_token: t.read_bytes_short().to_vec(),
        t133: t.read_bytes_short().to_vec(),
        encrypted_a1: t.read_bytes_short().to_vec(),
        wt_session_ticket_key: t.read_bytes_short().to_vec(),
        out_packet_session_id: t.read_bytes_short().to_vec(),
        tgtgt_key: t.read_bytes_short().to_vec(),
    }
}

pub struct ClientBuilder {
    device_source: DeviceSource,
    version: &'static Version,
    authentication: Option<Authentication>,
    session_store: Arc<Option<Box<dyn SessionStore + Sync + Send>>>,
    modules_vec: Arc<Vec<Module>>,
    result_handlers_vec: Arc<Vec<EventResultHandler>>,
    show_qr: Option<ShowQR>,
    show_slider: Option<ShowSlider>,
    device_lock_verification: Option<DeviceLockVerification>,
    #[cfg(feature = "connect_handler")]
    connect_handler_arc: Arc<Option<Box<dyn ConnectionHandler + Sync + Send>>>,
    reconnect_duration: Duration,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            device_source: DeviceSource::default(),
            version: &ANDROID_PHONE,
            authentication: None,
            session_store: Arc::new(None),
            modules_vec: Arc::new(vec![]),
            result_handlers_vec: Arc::new(vec![]),
            show_qr: None,
            show_slider: None,
            device_lock_verification: None,
            #[cfg(feature = "connect_handler")]
            connect_handler_arc: None.into(),
            reconnect_duration: Duration::from_millis(100),
        }
    }

    pub fn modules<S: Into<Arc<Vec<Module>>>>(mut self, h: S) -> Self {
        self.modules_vec = h.into();
        self
    }

    pub fn result_handlers<E: Into<Arc<Vec<EventResultHandler>>>>(mut self, e: E) -> Self {
        self.result_handlers_vec = e.into();
        self
    }

    /// 设置显示二维码的方式
    pub fn show_rq<E: Into<Option<ShowQR>>>(mut self, show_qr: E) -> Self {
        self.show_qr = show_qr.into();
        self
    }

    /// 设置显示滑动条的方式
    pub fn show_slider<E: Into<Option<ShowSlider>>>(mut self, show_slider: E) -> Self {
        self.show_slider = show_slider.into();
        self
    }

    /// 设置显示滑动条的方式（如果是windows可以直接在桌面滑动）
    #[cfg(all(any(target_os = "windows"), feature = "pop_window_slider"))]
    pub fn show_slider_pop_menu_if_possible(self) -> Self {
        self.show_slider(ShowSlider::PopWindow)
    }

    /// 设置显示滑动条的方式（如果是windows可以直接在桌面滑动）
    #[cfg(not(all(any(target_os = "windows"), feature = "pop_window_slider")))]
    pub fn show_slider_pop_menu_if_possible(self) -> Self {
        self
    }

    /// 设置解锁设备所的方式
    pub fn device_lock_verification<E: Into<Option<DeviceLockVerification>>>(
        mut self,
        device_lock_verification: E,
    ) -> Self {
        self.device_lock_verification = device_lock_verification.into();
        self
    }

    pub async fn build(&self) -> Result<Client, anyhow::Error> {
        Ok(Client {
            rq_client: Arc::new(ricq::Client::new(
                match &self.device_source {
                    JsonFile(file_name) => {
                        if Path::new(file_name).exists() {
                            parse_device_json(
                                &tokio::fs::read_to_string(file_name)
                                    .await
                                    .with_context(|| format!("读取文件失败 : {}", file_name))?,
                            )?
                        } else {
                            let device = Device::random();
                            tokio::fs::write(file_name, serde_json::to_string(&device).unwrap())
                                .await
                                .with_context(|| format!("写入文件失败 : {}", file_name))?;
                            device
                        }
                    }
                    JsonString(json_string) => parse_device_json(json_string)?,
                },
                self.version.clone(),
                ClientHandler {
                    modules: self.modules_vec.clone(),
                    result_handlers: self.result_handlers_vec.clone(),
                },
            )),
            authentication: self
                .authentication
                .clone()
                .with_context(|| "您必须设置验证方式 (调用authentication)")?,
            session_store: self.session_store.clone(),
            modules: self.modules_vec.clone(),
            result_handlers: self.result_handlers_vec.clone(),
            show_qr: if self.show_qr.is_some() {
                self.show_qr.clone().unwrap()
            } else {
                #[cfg(feature = "console_qr")]
                let show_qr = ShowQR::PrintToConsole;
                #[cfg(not(feature = "console_qr"))]
                let show_qr = ShowQR::SaveToFile;
                show_qr
            },
            show_slider: if self.show_slider.is_some() {
                self.show_slider.clone().unwrap()
            } else {
                ShowSlider::AndroidHelper
            },
            device_lock_verification: if self.device_lock_verification.is_some() {
                self.device_lock_verification.clone().unwrap()
            } else {
                DeviceLockVerification::Url
            },
            shutting: false,
            #[cfg(feature = "connect_handler")]
            connection_handler: self.connect_handler_arc.clone(),
            reconnect_duration: self.reconnect_duration,
        })
    }

    pub fn device(mut self, device_source: DeviceSource) -> Self {
        self.device_source = device_source;
        self
    }

    pub fn version(mut self, version: &'static Version) -> Self {
        self.version = version;
        self
    }

    pub fn session_store(mut self, session_store: Box<dyn SessionStore + Sync + Send>) -> Self {
        self.session_store = Arc::new(Some(session_store));
        self
    }

    pub fn authentication(mut self, authentication: Authentication) -> Self {
        self.authentication = Some(authentication);
        self
    }

    #[cfg(feature = "connect_handler")]
    pub fn connect_handler(
        mut self,
        connect_handler: Box<dyn ConnectionHandler + Sync + Send>,
    ) -> Self {
        self.connect_handler_arc = Arc::new(Some(connect_handler));
        self
    }

    pub fn reconnect_duration(mut self, reconnect_duration: Duration) -> Self {
        self.reconnect_duration = reconnect_duration;
        self
    }
}

fn parse_device_json(json: &str) -> Result<Device, anyhow::Error> {
    Ok(serde_json::from_str(json).with_context(|| format!("DeviceJson解析失败"))?)
}

#[derive(Clone, Debug)]
pub enum ShowQR {
    OpenBySystem,
    #[cfg(feature = "console_qr")]
    PrintToConsole,
    Custom(Pin<Box<fn(Bytes) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>>>),
    SaveToFile,
}

#[cfg(feature = "console_qr")]
fn print_qr_to_console(buff: &Bytes) -> Result<()> {
    let img = image::load_from_memory(buff)?.into_luma8();
    let mut img = rqrr::PreparedImage::prepare(img);
    let grids = img.detect_grids();
    let (_, content) = grids.get(0).with_context(|| "未能识别出二维码")?.decode()?;
    qr2term::print_qr(content.as_str())?;
    Ok(())
}

#[derive(Clone, Debug)]
pub enum ShowSlider {
    AndroidHelper,

    #[cfg(all(any(target_os = "windows"), feature = "pop_window_slider"))]
    PopWindow,
}

#[derive(Clone, Debug)]
pub enum DeviceLockVerification {
    Url,
    Sms,
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
