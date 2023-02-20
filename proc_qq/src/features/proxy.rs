use crate::{Connection, ConnectionHandler};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::io;
use std::io::ErrorKind;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::AsyncResolver;
use url::Host;

pub fn proxy_by_url(url: String) -> Result<Box<dyn ConnectionHandler + Send + Sync>> {
    Ok(Box::new(ProxyConnectHandler { proxy: url }))
}

struct ProxyConnectHandler {
    pub proxy: String,
}

#[async_trait]
impl ConnectionHandler for ProxyConnectHandler {
    async fn connect(&self, address: SocketAddr) -> Result<Box<dyn Connection>> {
        if self.proxy.is_empty() {
            return Ok(Box::new(ProxyConnect::TcpStream(
                TcpStream::connect(address).await?,
            )));
        }
        let proxy = url::Url::parse(self.proxy.as_str())
            .map_err(|err| io::Error::new(ErrorKind::InvalidData, err))?;
        let scheme = proxy.scheme();
        let host = proxy.host().ok_or(io::Error::new(
            ErrorKind::NotFound,
            format!("proxy host is missing from url: {}", proxy),
        ))?;
        let port = proxy.port().ok_or(io::Error::new(
            ErrorKind::NotFound,
            format!("proxy port is missing from url: {}", proxy),
        ))?;
        let username = proxy.username();
        let password = proxy.password().unwrap_or("");
        let socks_addr = match host {
            Host::Domain(domain) => {
                let resolver =
                    AsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default())?;
                let response = resolver.lookup_ip(domain).await?;
                let socks_ip_addr = response.into_iter().next().ok_or(io::Error::new(
                    ErrorKind::NotFound,
                    format!("proxy host did not return any ip address: {}", domain),
                ))?;
                SocketAddr::new(socks_ip_addr, port)
            }
            Host::Ipv4(v4) => SocketAddr::new(IpAddr::from(v4), port),
            Host::Ipv6(v6) => SocketAddr::new(IpAddr::from(v6), port),
        };

        let stream = match scheme {
            "socks5" => {
                if username.is_empty() {
                    ProxyConnect::Socks5Stream(
                        tokio_socks::tcp::Socks5Stream::connect(socks_addr, address)
                            .await
                            .map_err(|err| io::Error::new(ErrorKind::ConnectionAborted, err))?,
                    )
                } else {
                    ProxyConnect::Socks5Stream(
                        tokio_socks::tcp::Socks5Stream::connect_with_password(
                            socks_addr, address, username, password,
                        )
                        .await
                        .map_err(|err| io::Error::new(ErrorKind::ConnectionAborted, err))?,
                    )
                }
            }
            scheme => {
                return Err(anyhow!("proxy scheme not supported: {}", scheme));
            }
        };
        Ok(Box::new(stream))
    }
}

enum ProxyConnect {
    TcpStream(TcpStream),
    Socks5Stream(Socks5Stream<TcpStream>),
}

impl Connection for ProxyConnect {}

impl AsyncRead for ProxyConnect {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        AsyncRead::poll_read(
            Pin::new(&mut match self.get_mut() {
                ProxyConnect::TcpStream(s) => s,
                ProxyConnect::Socks5Stream(s) => s,
            }),
            cx,
            buf,
        )
    }
}

impl AsyncWrite for ProxyConnect {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        AsyncWrite::poll_write(
            Pin::new(&mut match self.get_mut() {
                ProxyConnect::TcpStream(s) => s,
                ProxyConnect::Socks5Stream(s) => s,
            }),
            cx,
            buf,
        )
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        AsyncWrite::poll_flush(
            Pin::new(match &mut self.get_mut() {
                ProxyConnect::TcpStream(s) => s,
                ProxyConnect::Socks5Stream(s) => s,
            }),
            cx,
        )
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        AsyncWrite::poll_shutdown(
            Pin::new(match &mut self.get_mut() {
                ProxyConnect::TcpStream(s) => s,
                ProxyConnect::Socks5Stream(s) => s,
            }),
            cx,
        )
    }
}
