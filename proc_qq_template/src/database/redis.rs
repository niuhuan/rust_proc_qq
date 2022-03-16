use crate::config::Redis;
use anyhow::Context;
use once_cell::sync::OnceCell;
use redis::{Client, Commands};

static CLIENT: OnceCell<Client> = OnceCell::new();

pub(crate) async fn init_redis(redis: &Redis) -> anyhow::Result<()> {
    let client = Client::open(format!("redis://{}:{}/", redis.host, redis.port))?;
    let mut con = client.get_connection().with_context(|| "redis连接失败")?;
    con.get("test_key")?;
    CLIENT.set(client).unwrap();
    tracing::info!("redis连接成功");
    Ok(())
}

#[allow(dead_code)]
pub(crate) async fn get_string(key: &str) -> anyhow::Result<Option<String>> {
    Ok(CLIENT.get().unwrap().get_connection()?.get(key)?)
}

#[allow(dead_code)]
pub(crate) async fn set_string(
    key: &str,
    value: &str,
    expire_seconds: usize,
) -> anyhow::Result<()> {
    CLIENT
        .get()
        .unwrap()
        .get_connection()?
        .set_ex(key, value, expire_seconds)?;
    Ok(())
}
