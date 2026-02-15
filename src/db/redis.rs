use redis::Client;

/// Crea un cliente Redis.
pub fn create_client(redis_url: &str) -> Client {
    Client::open(redis_url).expect("Failed to create Redis client")
}

/// Helper: guardar un valor con TTL (en segundos).
pub async fn set_with_ttl(
    client: &Client,
    key: &str,
    value: &str,
    ttl_secs: u64,
) -> redis::RedisResult<()> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    redis::cmd("SET")
        .arg(key)
        .arg(value)
        .arg("EX")
        .arg(ttl_secs)
        .query_async(&mut conn)
        .await
}

/// Helper: leer un valor.
pub async fn get(client: &Client, key: &str) -> redis::RedisResult<Option<String>> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    redis::cmd("GET").arg(key).query_async(&mut conn).await
}

/// Helper: incrementar un counter (para rate limiting).
pub async fn incr(client: &Client, key: &str, ttl_secs: u64) -> redis::RedisResult<i64> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    let count: i64 = redis::cmd("INCR").arg(key).query_async(&mut conn).await?;
    if count == 1 {
        redis::cmd("EXPIRE")
            .arg(key)
            .arg(ttl_secs)
            .query_async::<_, ()>(&mut conn)
            .await?;
    }
    Ok(count)
}

/// Helper: borrar una clave.
pub async fn del(client: &Client, key: &str) -> redis::RedisResult<()> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    redis::cmd("DEL").arg(key).query_async(&mut conn).await
}