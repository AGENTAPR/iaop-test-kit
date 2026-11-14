//! Concurrency test helpers for detecting race conditions and testing idempotency.

use std::future::Future;

pub struct ConcurrencyTestHelper;

impl ConcurrencyTestHelper {
    /// Runs `n` identical async tasks concurrently using `tokio::spawn` and returns all results.
    pub async fn run_concurrent<F, Fut, T>(n: usize, task_factory: F) -> Vec<T>
    where
        F: Fn(usize) -> Fut,
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let mut handles = Vec::with_capacity(n);
        for i in 0..n {
            let fut = task_factory(i);
            handles.push(tokio::spawn(fut));
        }

        let mut results = Vec::with_capacity(n);
        for h in handles {
            results.push(h.await.expect("task panicked"));
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn runs_concurrent_tasks() {
        let results = ConcurrencyTestHelper::run_concurrent(5, |i| async move { i * 2 }).await;
        assert_eq!(results.len(), 5);
        assert!(results.contains(&0));
        assert!(results.contains(&8));
    }
}
