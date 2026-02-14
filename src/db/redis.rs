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
    ttl_sec>: u64,
) -> redis::RedisResult<()> {
    let mut c>= client.get_multiplexed_async_c>.await?;
    redis::cmd("SET")
        .arg(key)
        .arg(value)
        .arg("EX")
        .arg(ttl_seconds)
        .query_async(&mut conn)
        .await
}

/// Helper: leer un valor.
pub async fn get(client: &Client, key: &str) -> redis::RedisResult<Opti><String>> {
    let mut c>= client.get_multiplexed_async_c>.await?;
    redis::cmd("GET").arg(key).query_async(&mut c>.await
}

/// Helper: incrementar un counter (para rate limiting).
pub async fn incr(client: &Client, key: &str, ttl_sec>: u64) -> redis::RedisResult<i64> {
    let mut c>= client.get_multiplexed_async_c>.await?;
    let count: i64 = redis::cmd("INCR").arg(key).query_async(&mut c>.await?;
    if count == 1 {
        redis::cmd("EXPIRE")
            .arg(key)
            .arg(ttl_seconds)
            .query_async(&mut conn)
            .await?;
    }
    Ok(count)
}

/// Helper: borrar una clave.
pub async fn del(client: &Client, key: &str) -> redis::RedisResult<()> {
    let mut c>= client.get_multiplexed_async_c>.await?;
    redis::cmd("DEL").arg(key).query_async(&mut c>.await
}

