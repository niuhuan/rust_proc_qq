use crate::config::Redis;
use anyhow::Context;
use once_cell::sync::OnceCell;
use redis::{AsyncCommands, Client, RedisResult};
static CLIENT: OnceCell<Client> = OnceCell::new();

pub(crate) async fn init_redis(redis: &Redis) -> anyhow::Result<()> {
    let client = Client::open(format!("redis://{}:{}/", redis.host, redis.port))?;
    let mut con = client
        .get_async_connection()
        .await
        .with_context(|| "redis连接失败")?;
    con.get("test_key").await?;
    CLIENT.set(client).unwrap();
    tracing::info!("redis连接成功");
    Ok(())
}

#[allow(dead_code)]
pub(crate) async fn get_string(key: &str) -> RedisResult<Option<String>> {
    CLIENT
        .get()
        .unwrap()
        .get_async_connection()
        .await?
        .get(key)
        .await
}

#[allow(dead_code)]
pub(crate) async fn set_string(key: &str, value: &str, expire_seconds: usize) -> RedisResult<()> {
    CLIENT
        .get()
        .unwrap()
        .get_async_connection()
        .await?
        .set_ex(key, value, expire_seconds)
        .await
}
