use crate::score::Score;
use std::error::Error;

// Feature-gated Redis module. Included only if the "redis" feature is enabled.
#[cfg(feature = "redis")]
mod redis;

// Feature-gated use statement. Makes `Redis` available only if the "redis" feature is enabled.
#[cfg(feature = "redis")]
pub use redis::Redis;

mod memory;
pub use memory::Memory;

/// Configuration options for different storage types.
///
/// The configuration is defined as an enum to represent various storage types.
/// Feature gates are used to conditionally compile code for specific storage,
/// like Redis, based on the compilation features provided.
#[derive(serde::Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Config {
    // The Redis configuration is only included if the "redis" feature is enabled.
    #[cfg(feature = "redis")]
    Redis(redis::Config),

    // Memory storage configuration.
    Memory,
}

impl Default for Config {
    /// Provides a default configuration, which is the in-memory storage.
    fn default() -> Self {
        Config::Memory
    }
}

/// Constructs a storage instance from the provided configuration.
///
/// Based on the provided `Config`, this function initializes the appropriate
/// storage backend.
///
/// # Arguments
/// * `config` - Storage configuration.
///
/// # Returns
/// A boxed storage instance implementing the `Store` trait.
pub fn from_config(config: Config) -> Box<dyn Store + Sync + Send + 'static> {
    match config {
        // Initialize Redis storage if the "redis" feature is enabled and selected.
        #[cfg(feature = "redis")]
        Config::Redis(config) => Box::new(Redis::from_url(config.connection)),

        // Initialize in-memory storage by default.
        Config::Memory => Box::new(Memory::new()),
    }
}

/// Trait defining the key-value store functionality.
/// This trait abstracts the store layer, allowing various implementations such as memory-based or database-backed stores.
#[async_trait::async_trait]
pub trait Store {
    /// Sets a score for a given key.
    ///
    /// ## Arguments
    /// * `key`: String - The key to associate with the score.
    /// * `value`: Score - The score to store.
    ///
    /// ## Returns
    /// A result indicating success or an error.
    async fn set(&self, key: String, value: Score) -> Result<(), Box<dyn Error>>;
    /// Retrieves the score associated with a given key.
    ///
    /// ## Arguments
    /// * `key`: &str - The key whose score is to be retrieved.
    ///
    /// ## Returns
    /// An optional score if found, or None otherwise.
    async fn get(&self, key: &str) -> Result<Option<Score>, Box<dyn Error>>;
    /// Retrieves the key associated with the highest score.
    ///
    /// ## Returns
    /// An optional string representing the key of the highest score, or None if the store is empty.
    async fn best_url(&self) -> Result<Option<String>, Box<dyn Error>>;
}
