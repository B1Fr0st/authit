use redis::AsyncCommands;
use tracing::info;

#[derive(Clone)]
pub struct TokenBlacklist {
    redis_client: redis::Client,
}

impl TokenBlacklist {
    pub fn new(redis_client: redis::Client) -> Self {
        Self { redis_client }
    }

    /// Blacklist a specific token until its expiration
    pub async fn blacklist_token(&self, token: &str, expires_in_seconds: i64) -> Result<(), redis::RedisError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("blacklist:token:{}", token);

        // Set with TTL matching token expiration
        let _: () = conn.set_ex(&key, "1", expires_in_seconds as u64).await?;
        info!("Blacklisted token (expires in {}s)", expires_in_seconds);

        Ok(())
    }

    /// Check if a specific token is blacklisted
    pub async fn is_token_blacklisted(&self, token: &str) -> Result<bool, redis::RedisError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("blacklist:token:{}", token);

        conn.exists(&key).await
    }

    /// Blacklist all tokens for a user issued before a specific timestamp
    /// This is used when a user's role changes - we invalidate old tokens but allow new ones
    /// TTL should match the maximum token lifetime (e.g., 24 hours)
    pub async fn blacklist_user_before_timestamp(&self, user_id: &str, timestamp: i64, ttl_seconds: i64) -> Result<(), redis::RedisError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("blacklist:user:{}", user_id);

        // Store the timestamp until longest possible token expiration
        let _: () = conn.set_ex(&key, timestamp.to_string(), ttl_seconds as u64).await?;
        info!("Blacklisted tokens for user {} issued before timestamp {} (expires in {}s)", user_id, timestamp, ttl_seconds);

        Ok(())
    }

    /// Check if a user's token is blacklisted based on when it was issued
    /// Returns true if the token was issued before the stored blacklist timestamp
    pub async fn is_user_token_blacklisted(&self, user_id: &str, token_issued_at: i64) -> Result<bool, redis::RedisError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("blacklist:user:{}", user_id);

        // Get the blacklist timestamp (tokens issued before this are invalid)
        let blacklist_timestamp: Option<String> = conn.get(&key).await?;

        match blacklist_timestamp {
            Some(timestamp_str) => {
                // Parse the timestamp and compare
                if let Ok(blacklist_ts) = timestamp_str.parse::<i64>() {
                    Ok(token_issued_at < blacklist_ts)
                } else {
                    Ok(false)
                }
            }
            None => Ok(false), // No blacklist entry for this user
        }
    }

    /// Remove user from blacklist (if needed for debugging/admin override)
    pub async fn unblacklist_user(&self, user_id: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("blacklist:user:{}", user_id);

        let _: () = conn.del(&key).await?;
        info!("Removed blacklist for user {}", user_id);

        Ok(())
    }
}
