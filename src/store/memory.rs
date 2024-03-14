use super::Store;
use crate::score::Score;
use std::error::Error;

/// In-memory store for scores.
///
/// Utilizes a concurrent hash map for storing and retrieving scores quickly and efficiently.
#[derive(Debug, Clone)]
pub struct Memory {
    /// The inner data structure for storing scores.
    /// Maps a `String` (representing a URL) to a `Score`.
    pub inner: dashmap::DashMap<String, Score>,
}

impl Default for Memory {
    /// Creates a new in-memory store instance.
    ///
    /// ## Returns
    /// A new `Memory` instance with an initialized `DashMap`.
    fn default() -> Self {
        Self { inner: dashmap::DashMap::new() }
    }
}

impl Memory {
    /// Creates a new in-memory store instance.
    ///
    /// ## Returns
    /// A new `Memory` instance with an initialized `DashMap`.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl Store for Memory {
    /// Sets a score for a specific key.
    ///
    /// ## Arguments
    /// * `key`: String - The key under which to store the score.
    /// * `value`: Score - The score to store.
    ///
    /// ## Returns
    /// A result indicating success or an error.
    async fn set(&self, key: String, value: Score) -> Result<(), Box<dyn Error>> {
        self.inner.insert(key, value);
        Ok(())
    }
    /// Retrieves the score associated with a specific key.
    ///
    /// ## Arguments
    /// * `key`: &str - The key for which to retrieve the score.
    ///
    /// ## Returns
    /// An option containing the score if it exists, or None otherwise.
    async fn get(&self, key: &str) -> Result<Option<Score>, Box<dyn Error>> {
        Ok(self.inner.get(key).map(|v| v.value().clone()))
    }
    /// Identifies the key associated with the best score (highest value).
    ///
    /// ## Returns
    /// An option containing the key of the best score if it exists, or None otherwise.
    async fn best_url(&self) -> Result<Option<String>, Box<dyn Error>> {
        Ok(self
            .inner
            .iter()
            .max_by(|a, b| a.value().score.partial_cmp(&b.value().score).expect("failed to compare scores"))
            .map(|v| v.key().clone()))
    }
}
