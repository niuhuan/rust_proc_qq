use crate::DeviceSource::{JsonFile, JsonString};
use crate::{Authentication, ClientHandler, DeviceSource, Module};
use anyhow::{Context, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use rq_engine::binary::{BinaryReader, BinaryWriter};
use rq_engine::command::wtlogin::{
    LoginDeviceLocked, LoginNeedCaptcha, LoginResponse, LoginSuccess, LoginUnknownStatus,
    QRCodeConfirmed, QRCodeImageFetch, QRCodeState,
};
use rq_engine::protocol::device::Device;
use rq_engine::protocol::version::{Version, ANDROID_PHONE};
use rq_engine::{RQResult, Token};
use rs_qq::ext::common::after_login;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::time::sleep;

pub struct Client {
    pub rq_client: Arc<rs_qq::Client>,
    pub authentication: Authentication,
    pub priority_session: Option<String>,
}

impl Client {
    pub fn start(self) -> JoinHandle<Result<()>> {
        tokio::spawn(run_client(self))
    }
}

pub async fn run_client(client: Client) -> Result<()> {
    // todo // KickedOffline on token login
    loop {
        // connect to server
        let stream = match TcpStream::connect(client.rq_client.get_address())
            .await
            .with_context(|| "连接到服务器失败")
        {
            Ok(stream) => stream,
            Err(err) => {
                tracing::info!("{:?}", err);
                tracing::info!("五秒钟之后重试");
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        let rq_client = client.rq_client.clone();
        let handle = tokio::spawn(async move { rq_client.start(stream).await });
        tokio::task::yield_now().await;
        // token login if allow and file exists
        if !token_login(&client).await {
            // authentication if token login failed or not set
            // The error of login failure is fatal
            login_authentication(&client).await?;
        }
        // Reference rs-qq docs, this function must be called after login is completed, maybe it's to register the device.
        after_login(&client.rq_client.clone()).await;
        // save session, IO errors are fatal.
        if let Some(session_file) = &client.priority_session {
            tokio::fs::write(
                session_file,
                token_to_bytes(&client.rq_client.gen_token().await),
            )
            .await
            .with_context(|| "写入session出错")?;
        }
        // hold handle
        match handle.await {
            Ok(_) => {}
            Err(err) => tracing::info!("{:?}", err),
        };
        tracing::info!("连接已断开, 五秒钟之后重试");
        sleep(Duration::from_secs(5)).await;
    }
}

async fn token_login(client: &Client) -> bool {
    if let Some(session_file) = &client.priority_session {
        if Path::new(session_file).exists() {
            let session_data = match tokio::fs::read(session_file)
                .await
                .with_context(|| format!("文件读取失败 : {}", session_file))
            {
                Ok(data) => data,
                Err(err) => {
                    tracing::info!("{:?}", err);
                    return false;
                }
            };
            client
                .rq_client
                .token_login(bytes_to_token(session_data))
                .await
                .is_ok()
        } else {
            false
        }
    } else {
        false
    }
}

async fn login_authentication(client: &Client) -> Result<()> {
    let rq_client = client.rq_client.clone();
    match &client.authentication {
        Authentication::QRCode => qr_login(rq_client).await,
        Authentication::UinPassword(uin, password) => {
            let first = rq_client.password_login(uin.clone(), password).await;
            loop_login(rq_client, first).await
        }
        Authentication::UinPasswordMd5(uin, password) => {
            let first = rq_client.password_md5_login(uin.clone(), password).await;
            loop_login(rq_client, first).await
        }
    }
}

async fn qr_login(rq_client: Arc<rs_qq::Client>) -> Result<()> {
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
                tokio::fs::write("qrcode.png", &image_data)
                    .await
                    .with_context(|| "failed to write file")?;
                image_sig = sig.clone();
                // todo 桌面环境直接打开, 服务器使用文字渲染
                tracing::info!("二维码: qrcode.png");
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
                return loop_login(rq_client, first).await;
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

async fn loop_login(client: Arc<rs_qq::Client>, first: RQResult<LoginResponse>) -> Result<()> {
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
                tracing::info!("手机打开url，处理完成后重启程序");
                std::process::exit(0);
                //也可以走短信验证
                // resp = client.request_sms().await.expect("failed to request sms");
            }
            LoginResponse::NeedCaptcha(LoginNeedCaptcha {
                ref verify_url,
                // 图片应该没了
                image_captcha: ref _image_captcha,
                ..
            }) => {
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
                resp = client.submit_ticket(&txt).await.expect("发送ticket失败");
            }
            LoginResponse::DeviceLockLogin { .. } => {
                resp = client
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

#[derive(Debug)]
pub struct ClientBuilder {
    device_source: DeviceSource,
    version: &'static Version,
    authentication: Option<Authentication>,
    priority_session: Option<String>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            device_source: DeviceSource::default(),
            version: ANDROID_PHONE,
            authentication: None,
            priority_session: None,
        }
    }

    pub async fn build(&self, h: Vec<Module>) -> Result<Client, anyhow::Error> {
        Ok(Client {
            rq_client: Arc::new(rs_qq::Client::new(
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
                self.version,
                ClientHandler { modules: h },
            )),
            authentication: self
                .authentication
                .clone()
                .with_context(|| "您必须设置验证方式 (调用authentication)")?,
            priority_session: self.priority_session.clone(),
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

    pub fn priority_session<S: Into<String>>(mut self, session_file: S) -> Self {
        self.priority_session = Some(session_file.into());
        self
    }

    pub fn authentication(mut self, authentication: Authentication) -> Self {
        self.authentication = Some(authentication);
        self
    }
}

fn parse_device_json(json: &str) -> Result<Device, anyhow::Error> {
    Ok(serde_json::from_str(json).with_context(|| format!("DeviceJson解析失败"))?)
}
