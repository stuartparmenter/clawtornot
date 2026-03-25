use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RateLimiter {
    general: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
    voting: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            general: Arc::new(Mutex::new(HashMap::new())),
            voting: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check general rate limit: 60 req/min. Returns Ok or Err(retry_after_secs).
    pub async fn check_general(&self, key_hash: &str) -> Result<(), u64> {
        let mut map = self.general.lock().await;
        let now = Instant::now();
        let entry = map.entry(key_hash.to_string()).or_insert((0, now));
        if now.duration_since(entry.1).as_secs() >= 60 {
            *entry = (1, now);
            Ok(())
        } else if entry.0 >= 60 {
            let retry_after = 60 - now.duration_since(entry.1).as_secs();
            Err(retry_after)
        } else {
            entry.0 += 1;
            Ok(())
        }
    }

    /// Check voting rate limit: 30 votes/hr. Returns Ok or Err(retry_after_secs).
    pub async fn check_voting(&self, key_hash: &str) -> Result<(), u64> {
        let mut map = self.voting.lock().await;
        let now = Instant::now();
        let entry = map.entry(key_hash.to_string()).or_insert((0, now));
        if now.duration_since(entry.1).as_secs() >= 3600 {
            *entry = (1, now);
            Ok(())
        } else if entry.0 >= 30 {
            let retry_after = 3600 - now.duration_since(entry.1).as_secs();
            Err(retry_after)
        } else {
            entry.0 += 1;
            Ok(())
        }
    }
}
