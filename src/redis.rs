use fred::prelude::*;

pub async fn connect_redis(host: String, port: u16) -> Result<RedisClient, RedisError> {
    let client = RedisClient::new(RedisConfig {
        fail_fast: false,
        pipeline: false,
        blocking: Default::default(),
        username: None,
        password: None,
        server: ServerConfig::new_centralized(host, port),
        tls: None
    });

    client.connect(Some(ReconnectPolicy::default()));
    client.wait_for_connect().await?;
    client.flushall(false).await?;

    if client.is_connected() {
        Ok(client)
    } else {
        Err(RedisError::from(()))
    }
}