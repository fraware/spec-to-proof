use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use rand::Rng;

#[derive(Debug)]
pub struct RateLimiter {
    max_requests: u32,
    window_duration: Duration,
    requests: Mutex<VecDeque<Instant>>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_duration: Duration) -> Self {
        Self {
            max_requests,
            window_duration,
            requests: Mutex::new(VecDeque::new()),
        }
    }

    pub async fn acquire(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();

        // Remove expired requests
        while let Some(&timestamp) = requests.front() {
            if now.duration_since(timestamp) > self.window_duration {
                requests.pop_front();
            } else {
                break;
            }
        }

        // Check if we're at the limit
        if requests.len() >= self.max_requests as usize {
            let oldest_request = requests.front().unwrap();
            let wait_time = self.window_duration - now.duration_since(*oldest_request);
            
            // Add jitter to prevent thundering herd
            let jitter = rand::thread_rng().gen_range(0..100);
            let total_wait = wait_time + Duration::from_millis(jitter);
            
            tracing::warn!(
                "Rate limit exceeded. Waiting {:?} ms (including {}ms jitter)",
                total_wait.as_millis(),
                jitter
            );
            
            tokio::time::sleep(total_wait).await;
        }

        requests.push_back(now);
        Ok(())
    }

    pub async fn get_current_usage(&self) -> (u32, u32) {
        let requests = self.requests.lock().await;
        let now = Instant::now();
        
        // Count active requests in window
        let active_count = requests
            .iter()
            .filter(|&&timestamp| now.duration_since(timestamp) <= self.window_duration)
            .count() as u32;

        (active_count, self.max_requests)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(5, Duration::from_secs(1));
        
        // Should allow 5 requests immediately
        for _ in 0..5 {
            assert!(limiter.acquire().await.is_ok());
        }
        
        // 6th request should be rate limited
        let start = Instant::now();
        assert!(limiter.acquire().await.is_ok());
        let elapsed = start.elapsed();
        
        // Should have waited at least 1 second
        assert!(elapsed >= Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_rate_limiter_window() {
        let limiter = RateLimiter::new(3, Duration::from_millis(100));
        
        // Make 3 requests
        for _ in 0..3 {
            assert!(limiter.acquire().await.is_ok());
        }
        
        // Wait for window to expire
        sleep(Duration::from_millis(150)).await;
        
        // Should allow more requests
        assert!(limiter.acquire().await.is_ok());
    }
} 