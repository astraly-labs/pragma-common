use std::fmt::Debug;

use futures::future::select_all;
use tokio::task::JoinHandle;

/// Allows us to run multiples tokio tasks at the same time but
/// cancels them all if one fails.
#[derive(Debug, Default)]
pub struct TaskGroup<T: Debug> {
    handles: Vec<JoinHandle<T>>,
}

impl<T: Debug> TaskGroup<T> {
    pub const fn new() -> Self {
        Self {
            handles: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_handle(mut self, task: JoinHandle<T>) -> Self {
        self.handles.push(task);
        self
    }

    pub async fn abort_all_if_one_resolves(self) {
        let handles = self.handles;
        if handles.is_empty() {
            return;
        }
        let (_, _, remaining) = select_all(handles).await;

        for task in remaining {
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;

    /// Test that verifies `TaskGroup` correctly aborts remaining tasks when one completes.
    ///
    /// This test creates three tasks:
    /// - Task 1: Completes quickly (50ms)
    /// - Task 2: Would take longer (2s) if not aborted
    /// - Task 3: Would take longer (2s) if not aborted
    ///
    /// When Task 1 completes, the `select_all` in `TaskGroup::run` returns, and
    /// the remaining tasks should be aborted.
    #[tokio::test]
    async fn test_task_group_aborts_remaining_tasks() {
        // Create atomic flags to track task completion
        let task1_completed = Arc::new(AtomicBool::new(false));
        let task2_completed = Arc::new(AtomicBool::new(false));
        let task3_completed = Arc::new(AtomicBool::new(false));

        // Task 1: Will complete quickly
        let flag1 = task1_completed.clone();
        let task1 = tokio::spawn(async move {
            // Short delay to ensure all tasks are started
            sleep(Duration::from_millis(50)).await;
            flag1.store(true, Ordering::SeqCst);
            "Task 1 completed"
        });

        // Task 2: Would run longer if not aborted
        let flag2 = task2_completed.clone();
        let task2 = tokio::spawn(async move {
            // Long delay that should be interrupted
            sleep(Duration::from_secs(2)).await;
            flag2.store(true, Ordering::SeqCst);
            "Task 2 completed"
        });

        // Task 3: Would also run longer if not aborted
        let flag3 = task3_completed.clone();
        let task3 = tokio::spawn(async move {
            // Long delay that should be interrupted
            sleep(Duration::from_secs(2)).await;
            flag3.store(true, Ordering::SeqCst);
            "Task 3 completed"
        });

        // Create and run the task group
        let task_group = TaskGroup::new()
            .with_handle(task1)
            .with_handle(task2)
            .with_handle(task3);

        // Run the task group - this will exit when any task completes
        task_group.abort_all_if_one_resolves().await;

        // Wait a bit to allow any potential task completion
        sleep(Duration::from_millis(500)).await;

        // Check task completion status
        assert!(
            task1_completed.load(Ordering::SeqCst),
            "Task 1 should have completed"
        );
        assert!(
            !task2_completed.load(Ordering::SeqCst),
            "Task 2 should have been aborted"
        );
        assert!(
            !task3_completed.load(Ordering::SeqCst),
            "Task 3 should have been aborted"
        );
    }
}
