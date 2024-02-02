use crate::config::Redis;
use anyhow::Context;
use once_cell::sync::OnceCell;
use redis::{AsyncCommands, Client, FromRedisValue, RedisResult};
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
pub(crate) async fn redis_get<T>(key: &str) -> RedisResult<Option<T>>
where
    T: FromRedisValue,
{
    CLIENT
        .get()
        .unwrap()
        .get_async_connection()
        .await?
        .get(key)
        .await
}

#[allow(dead_code)]
pub(crate) async fn redis_set<T>(key: &str, value: T, expire_seconds: usize) -> RedisResult<()>
where
    T: redis::ToRedisArgs + Send + Sync,
{
    CLIENT
        .get()
        .unwrap()
        .get_async_connection()
        .await?
        .set_ex(key, value, expire_seconds as u64)
        .await
}

#[allow(dead_code)]
pub(crate) async fn redis_delete(key: &str) -> RedisResult<()> {
    CLIENT
        .get()
        .unwrap()
        .get_async_connection()
        .await?
        .del(key)
        .await
}
