use anyhow::Result;
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait Connection: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

#[async_trait]
pub trait ConnectionHandler {
    async fn connect(&self, address: SocketAddr) -> Result<Box<dyn Connection>>;
}
