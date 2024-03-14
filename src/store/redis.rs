use super::Store; // Import the KVStore trait from the parent module
use crate::score::Score; // Import the Score struct from the crate root
use deadpool_redis::Pool; // Deadpool pool for managing Redis connections
use redis::AsyncCommands; // Import Redis async commands
use std::error::Error;

#[derive(serde::Deserialize, Debug)]
pub struct Config {
    pub connection: String,
}

/// Represents a store system using Redis.
///
/// Provides an asynchronous interface to interact with Redis,
/// including operations for storing and retrieving scores efficiently.
#[derive(Clone)]
pub struct Redis {
    // Pool of Redis connections for async communication
    inner: Pool,
    // Name of the sorted set used in Redis
    sorted_set_name: String,
    // Prefix for keys to avoid collisions
    key_prefix: String,
}

impl Default for Redis {
    /// Creates a new Redis store instance with default settings.
    ///
    /// ## Returns
    /// A new `Redis` instance with default settings.
    fn default() -> Self {
        Self::from_url("redis://localhost:6379")
    }
}

impl Redis {
    /// Constructs a new Redis store instance.
    ///
    /// ## Arguments
    /// * `url`: &str - Redis server URL.
    /// * `sorted_set_name`: &str - Name of the sorted set for storing scores.
    /// * `key_prefix`: &str - Prefix for key names maintain a unique namespace.
    ///
    /// ## Returns
    /// A `Result` containing the new Redis instance or an error if the connection fails.
    pub fn new<U, S, K>(url: U, sorted_set_name: S, key_prefix: K) -> Self
    where
        U: Into<String>,
        S: Into<String>,
        K: Into<String>,
    {
        let inner = deadpool_redis::Config::from_url(url).create_pool(None).expect("failed to create pool");

        Self { inner, sorted_set_name: sorted_set_name.into(), key_prefix: key_prefix.into() }
    }

    /// Constructs a Redis store instance from a URL with default prefix `isup:` and sorted set name `isup:scores`.
    ///
    /// ## Arguments
    /// * `url`: Option<&str> - Optional Redis server URL. Defaults to localhost.
    ///
    /// ## Returns
    /// A `Result` containing the new Redis instance or an error if the connection fails.
    pub fn from_url<I: Into<String>>(url: I) -> Self {
        Self::new(url, "isup:scores", "isup:")
    }
}

#[async_trait::async_trait]
impl Store for Redis {
    /// Sets a score for a given key.
    ///
    /// ## Arguments
    /// * `key` - String: The key under which to store the score.
    /// * `value` - Score: The score to be stored.
    ///
    /// ## Returns
    /// A `Result` indicating success or an error.
    ///
    /// Utilizes Redis pipeline to efficiently set data and update the sorted set.
    async fn set(&self, key: String, value: Score) -> Result<(), Box<dyn Error>> {
        // Retrieve a connection from the pool.
        let mut connection = self.inner.get().await?;
        let prefixed_key = format!("{}{}", self.key_prefix, key);
        // Create a new Redis pipeline. Pipelines allow for multiple commands
        // to be sent to the server without waiting for individual replies,
        // thus improving performance.
        let mut pipe = redis::pipe();
        // Serialize the `Score` object to a JSON string.
        let json = serde_yaml::to_string(&value)?;
        // Add a command to the pipeline to set the key-value pair in Redis.
        // The `ignore` method is used since we're not interested in the command's result.
        pipe.set(&prefixed_key, json).ignore();
        // Add a command to the pipeline to add the score to a sorted set.
        // The sorted set is used for efficiently retrieving the top scores.
        // Again, `ignore` is used as the result of this operation is not needed immediately.
        pipe.zadd(&self.sorted_set_name, &key, value.score).ignore();
        // Execute the pipeline. This sends all commands in the pipeline to Redis in one go.
        // `query_async` is used for asynchronous execution.
        Ok(pipe.query_async(&mut connection).await?)
    }

    // Retrieves a score for a given key.
    ///
    /// ## Arguments
    /// * `key` - String: The key for which to retrieve the score.
    ///
    /// ## Returns
    /// A `Result` containing the score or None if not found.
    ///
    /// Retrieves the score from Redis, handling serialization and key prefixing.
    async fn get(&self, key: &str) -> Result<Option<Score>, Box<dyn Error>> {
        let mut connection = self.inner.get().await?;
        let prefixed_key = format!("{}{}", self.key_prefix, key);

        Ok(match connection.get::<_, String>(prefixed_key).await {
            Ok(r) => serde_yaml::from_str(&r).ok(),
            Err(_) => None,
        })
    }

    /// Retrieves the key with the highest score.
    ///
    /// ## Returns
    /// A `Result` containing the key with the highest score or None if the store is empty.
    ///
    /// Uses a Redis sorted set to efficiently find the highest score.
    async fn best_url(&self) -> Result<Option<String>, Box<dyn Error>> {
        let mut connection = self.inner.get().await?;
        let best: Vec<String> = connection.zrevrange(&self.sorted_set_name, 0, 0).await?;
        Ok(best.first().cloned())
    }
}
