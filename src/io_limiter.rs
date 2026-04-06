use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoPriority {
    High,
    Medium,
    Low,
}

pub struct IoRateLimiter {
    bytes_per_second: AtomicU64,
    high_priority_quota: AtomicU64,
    medium_priority_quota: AtomicU64,
    low_priority_quota: AtomicU64,
    current_usage: AtomicU64,
    last_refill: std::sync::Mutex<Instant>,
    refill_interval: Duration,
}

impl IoRateLimiter {
    pub fn new(bytes_per_second: u64) -> Self {
        Self {
            bytes_per_second: AtomicU64::new(bytes_per_second),
            high_priority_quota: AtomicU64::new(bytes_per_second / 2),
            medium_priority_quota: AtomicU64::new(bytes_per_second / 3),
            low_priority_quota: AtomicU64::new(bytes_per_second / 6),
            current_usage: AtomicU64::new(0),
            last_refill: std::sync::Mutex::new(Instant::now()),
            refill_interval: Duration::from_millis(100),
        }
    }

    pub fn request(&self, bytes: u64, priority: IoPriority) -> bool {
        self.refill_if_needed();

        let quota = match priority {
            IoPriority::High => &self.high_priority_quota,
            IoPriority::Medium => &self.medium_priority_quota,
            IoPriority::Low => &self.low_priority_quota,
        };

        loop {
            let current = quota.load(Ordering::Relaxed);
            if current < bytes {
                return false;
            }

            if quota
                .compare_exchange_weak(
                    current,
                    current - bytes,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                self.current_usage.fetch_add(bytes, Ordering::Relaxed);
                return true;
            }
        }
    }

    pub fn request_with_timeout(
        &self,
        bytes: u64,
        priority: IoPriority,
        timeout: Duration,
    ) -> bool {
        let start = Instant::now();

        while start.elapsed() < timeout {
            if self.request(bytes, priority) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        false
    }

    fn refill_if_needed(&self) {
        let mut last_refill = self.last_refill.lock().unwrap();
        let now = Instant::now();

        if now.duration_since(*last_refill) >= self.refill_interval {
            let total_bytes = self.bytes_per_second.load(Ordering::Relaxed);
            let refill_amount = (total_bytes as f64 * self.refill_interval.as_secs_f64()) as u64;

            self.high_priority_quota
                .fetch_add(refill_amount / 2, Ordering::Relaxed);
            self.medium_priority_quota
                .fetch_add(refill_amount / 3, Ordering::Relaxed);
            self.low_priority_quota
                .fetch_add(refill_amount / 6, Ordering::Relaxed);

            self.current_usage.store(0, Ordering::Relaxed);
            *last_refill = now;
        }
    }

    pub fn set_rate(&self, bytes_per_second: u64) {
        self.bytes_per_second
            .store(bytes_per_second, Ordering::Relaxed);
    }

    pub fn get_current_usage(&self) -> u64 {
        self.current_usage.load(Ordering::Relaxed)
    }

    pub fn get_available_quota(&self, priority: IoPriority) -> u64 {
        self.refill_if_needed();

        match priority {
            IoPriority::High => self.high_priority_quota.load(Ordering::Relaxed),
            IoPriority::Medium => self.medium_priority_quota.load(Ordering::Relaxed),
            IoPriority::Low => self.low_priority_quota.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_basic() {
        let limiter = IoRateLimiter::new(1000);

        assert!(limiter.request(100, IoPriority::High));
        assert!(limiter.request(100, IoPriority::Medium));
        assert!(limiter.request(100, IoPriority::Low));
    }

    #[test]
    fn test_rate_limiter_quota() {
        let limiter = IoRateLimiter::new(1000);

        let available = limiter.get_available_quota(IoPriority::High);
        assert!(available > 0);
    }

    #[test]
    fn test_rate_limiter_timeout() {
        let limiter = IoRateLimiter::new(100);

        let result =
            limiter.request_with_timeout(1000, IoPriority::High, Duration::from_millis(50));
        assert!(!result);
    }
}
