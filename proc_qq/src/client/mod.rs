use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
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

pub use entities::*;

mod entities;

pub struct Client {
    pub rq_client: Arc<rs_qq::Client>,
    pub authentication: Authentication,
    pub priority_session: Option<String>,
}

impl Client {
    pub fn start(self) -> JoinHandle<Result<(), Box<dyn Error + Sync + Send>>> {
        tokio::spawn(run_client(self))
    }
}

pub async fn run_client(client: Client) -> Result<(), Box<dyn Error + Sync + Send>> {
    let stream = TcpStream::connect(client.rq_client.get_address())
        .await
        .expect("failed to connect");
    let rq_client = client.rq_client.clone();
    let handle = tokio::spawn(async move { rq_client.start(stream).await });
    tokio::task::yield_now().await;
    let token_login = if let Some(session_file) = &client.priority_session {
        if Path::new(session_file).exists() {
            let session_data = tokio::fs::read(session_file)
                .await
                .with_context(|| format!("fs io error : {}", session_file))?;
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
    };
    if !token_login {
        login_authentication(&client).await
    }
    after_login(&client.rq_client.clone()).await;
    if let Some(session_file) = &client.priority_session {
        tokio::fs::write(session_file, client.rq_client.gen_token().await)
            .await
            .unwrap();
    }
    handle.await?;
    Ok(())
}

async fn login_authentication(client: &Client) {
    let rq_client = client.rq_client.clone();
    match &client.authentication {
        Authentication::QRCode => qr_login(rq_client).await,
        Authentication::UinPassword(uin, password) => {
            let first = rq_client.password_login(uin.clone(), password).await;
            loop_login(rq_client, first).await;
        }
        Authentication::UinPasswordMd5(uin, password) => {
            let first = rq_client.password_md5_login(uin.clone(), password).await;
            loop_login(rq_client, first).await;
        }
    };
}

async fn qr_login(rq_client: Arc<rs_qq::Client>) {
    let mut image_sig = Bytes::new();
    let mut resp = rq_client
        .fetch_qrcode()
        .await
        .expect("failed to fetch qrcode");
    loop {
        match resp {
            QRCodeState::ImageFetch(QRCodeImageFetch {
                ref image_data,
                ref sig,
            }) => {
                tokio::fs::write("qrcode.png", &image_data)
                    .await
                    .expect("failed to write file");
                image_sig = sig.clone();
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
                if let QRCodeState::ImageFetch(QRCodeImageFetch {
                    ref image_data,
                    ref sig,
                }) = rq_client
                    .fetch_qrcode()
                    .await
                    .expect("failed to fetch qrcode")
                {
                    tokio::fs::write("qrcode.png", &image_data)
                        .await
                        .expect("failed to write file");
                    image_sig = sig.clone();
                    tracing::info!("二维码: qrcode.png");
                }
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
                loop_login(rq_client, first).await;
                return;
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

async fn loop_login(client: Arc<rs_qq::Client>, first: RQResult<LoginResponse>) {
    let mut resp = first.unwrap();
    loop {
        match resp {
            LoginResponse::Success(LoginSuccess {
                ref account_info, ..
            }) => {
                tracing::info!("login success: {:?}", account_info);
                return;
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
                let mut txt = reqwest::ClientBuilder::new().build().unwrap().get(&helper_url).header(
                    "user-agent", "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.80 Mobile Safari/537.36",
                ).send().await
                    .unwrap()
                    .text()
                    .await
                    .unwrap();
                tracing::info!("helper: {}", txt);
                loop {
                    sleep(Duration::from_secs(5)).await;
                    let rsp = reqwest::ClientBuilder::new().build().unwrap().get(&helper_url).header(
                        "user-agent", "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.80 Mobile Safari/537.36",
                    ).send().await
                        .unwrap()
                        .text()
                        .await
                        .unwrap();
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
