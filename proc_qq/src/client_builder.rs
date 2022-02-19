use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use rq_engine::protocol::device::Device;
use rq_engine::protocol::version::{Version, ANDROID_PHONE};

use crate::DeviceSource::{JsonFile, JsonString};
use crate::{Authentication, Client, ClientHandler, DeviceSource, Module};

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
                                    .with_context(|| {
                                        format!("failed to read file : {}", file_name)
                                    })?,
                            )?
                        } else {
                            let device = Device::random();
                            tokio::fs::write(file_name, serde_json::to_string(&device).unwrap())
                                .await
                                .with_context(|| format!("failed to write file : {}", file_name))?;
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
                .with_context(|| "must be set authentication")?,
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
    Ok(serde_json::from_str(json).with_context(|| format!("failed to deserialize device json"))?)
}
