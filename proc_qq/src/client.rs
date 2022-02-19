use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use bytes::Bytes;
use rq_engine::command::wtlogin::{
    LoginDeviceLocked, LoginNeedCaptcha, LoginResponse, LoginSuccess, LoginUnknownStatus,
    QRCodeConfirmed, QRCodeImageFetch, QRCodeState,
};
use rq_engine::RQResult;
use rs_qq::ext::common::after_login;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::Authentication;

/// 客户端, 对RS-QQ的封装
pub struct Client {
    pub rq_client: Arc<rs_qq::Client>,
    pub authentication: Authentication,
    pub priority_session: Option<String>,
}

impl Client {
    /// 启动客户端
    pub fn start(self) -> JoinHandle<Result<()>> {
        tokio::spawn(run_client(self))
    }
}

/// 启动客户端
pub async fn run_client(client: Client) -> Result<()> {
    loop {
        // 连接到服务器, 并启动客户端, 并获取到handle
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
        // 使用session恢复登录
        if !token_login(&client).await {
            // 不成功的话使用验证登录
            // 验证登录不成功认为是致命错误
            login_authentication(&client).await?;
        }
        // rs-qq 文档中说明, 登录完成后必须注册设备
        after_login(&client.rq_client.clone()).await;
        // 如果需要的话保存session, 如果出错认为是致命错误
        if let Some(session_file) = &client.priority_session {
            tokio::fs::write(session_file, client.rq_client.gen_token().await)
                .await
                .with_context(|| "写入session出错")?;
        }
        // 等待结束
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
                .with_context(|| format!("fs io error : {}", session_file))
            {
                Ok(data) => data,
                Err(err) => {
                    tracing::info!("{:?}", err);
                    return false;
                }
            };
            client
                .rq_client
                .token_login(Bytes::from(session_data))
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
        .with_context(|| "failed to fetch qrcode")?;
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
                // 桌面环境直接打开, 服务器使用文字渲染
                tracing::info!("二维码: qrcode.png");
            }
            QRCodeState::WaitingForScan => {
                tracing::info!("二维码待扫描")
            }
            QRCodeState::WaitingForConfirm => {
                tracing::info!("二维码待确认")
            }
            QRCodeState::Timeout => {
                tracing::info!("二维码已超时，重新获取");
                resp = rq_client
                    .fetch_qrcode()
                    .await
                    .with_context(|| "failed to fetch qrcode")?;
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
                panic!("二维码已取消")
            }
        }
        sleep(Duration::from_secs(5)).await;
        resp = rq_client
            .query_qrcode_result(&image_sig)
            .await
            .expect("failed to query qrcode result");
    }
}

async fn loop_login(client: Arc<rs_qq::Client>, first: RQResult<LoginResponse>) -> Result<()> {
    let mut resp = first.unwrap();
    loop {
        match resp {
            LoginResponse::Success(LoginSuccess {
                ref account_info, ..
            }) => {
                tracing::info!("login success: {:?}", account_info);
                return Ok(());
            }
            LoginResponse::DeviceLocked(LoginDeviceLocked {
                ref sms_phone,
                ref verify_url,
                ref message,
                ..
            }) => {
                tracing::info!("device locked: {:?}", message);
                tracing::info!("sms_phone: {:?}", sms_phone);
                tracing::info!("verify_url: {:?}", verify_url);
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
                tracing::info!("verify_url: {:?}", verify_url);
                let helper_url = verify_url
                    .clone()
                    .unwrap()
                    .replace("ssl.captcha.qq.com", "txhelper.glitch.me");
                tracing::info!("helper_url: {:?}", helper_url);
                let mut txt = http_get(&helper_url)
                    .await
                    .with_context(|| "http请求失败")?;
                tracing::info!("helper: 滑动后请等待: {}", txt);
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
                tracing::info!("helper: {}", txt);
                resp = client
                    .submit_ticket(&txt)
                    .await
                    .expect("failed to submit ticket");
            }
            LoginResponse::DeviceLockLogin { .. } => {
                resp = client
                    .device_lock_login()
                    .await
                    .expect("failed to login with device lock");
            }
            LoginResponse::AccountFrozen => {
                panic!("account frozen");
            }
            LoginResponse::TooManySMSRequest => {
                panic!("too many sms request");
            }
            LoginResponse::UnknownStatus(LoginUnknownStatus {
                ref status,
                ref tlv_map,
            }) => {
                panic!("unknown login status: {:?}, {:?}", status, tlv_map);
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
