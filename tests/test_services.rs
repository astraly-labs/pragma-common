#[cfg(feature = "services")]
mod test_services {
    use pragma_common::services::{Service, ServiceContext, ServiceGroup, ServiceRunner};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::task::JoinSet;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_service_context_cancellation() {
        let ctx = ServiceContext::new();
        assert!(!ctx.is_cancelled());

        // Test cancellation
        ctx.cancel();
        assert!(ctx.is_cancelled());
    }

    #[tokio::test]
    async fn test_service_context_run_until_cancelled() {
        let ctx = ServiceContext::new();

        // Test that it returns the future's result when completed before cancellation
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let handle = tokio::spawn(async move {
            let result = ctx
                .run_until_cancelled(async move {
                    *counter_clone.lock().unwrap() += 1;
                    "completed"
                })
                .await;

            assert_eq!(result, Some("completed"));
        });

        handle.await.unwrap();
        assert_eq!(*counter.lock().unwrap(), 1);

        // Test that it returns None when cancelled before completion
        let ctx = ServiceContext::new();
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        let ctx_clone = ctx.clone();

        let handle = tokio::spawn(async move {
            let result = ctx_clone
                .run_until_cancelled(async move {
                    sleep(Duration::from_secs(1)).await;
                    *counter_clone.lock().unwrap() += 1;
                    "completed"
                })
                .await;

            assert_eq!(result, None);
        });

        // Cancel immediately
        ctx.cancel();
        handle.await.unwrap();

        // Ensure the future was cancelled before incrementing the counter
        assert_eq!(*counter.lock().unwrap(), 0);
    }

    struct TestService {
        counter: Arc<Mutex<i32>>,
        sleep_duration: Option<Duration>,
        should_panic: bool,
    }

    #[async_trait::async_trait]
    impl Service for TestService {
        async fn start<'a>(&mut self, mut runner: ServiceRunner<'a>) -> anyhow::Result<()> {
            let counter = self.counter.clone();
            let sleep_duration = self.sleep_duration;
            let should_panic = self.should_panic;

            runner.spawn_loop(move |ctx| async move {
                if should_panic {
                    panic!("Service panic as requested");
                }

                loop {
                    if ctx.is_cancelled() {
                        break;
                    }

                    {
                        let mut locked = counter.lock().unwrap();
                        *locked += 1;
                    }

                    if let Some(duration) = sleep_duration {
                        sleep(duration).await;
                    } else {
                        // Small sleep to avoid busy loop
                        sleep(Duration::from_millis(10)).await;
                    }
                }

                Ok::<(), anyhow::Error>(())
            });

            Ok(())
        }
    }

    #[tokio::test]
    async fn test_service_runner_spawn_loop() {
        let ctx = ServiceContext::new();
        let mut join_set = JoinSet::new();
        let mut runner = ServiceRunner::new(ctx.clone(), &mut join_set);

        let counter = Arc::new(Mutex::new(0));
        let counter_for_task = counter.clone();

        // Create and start a test service, but we don't need to store it
        let _test_service = TestService {
            counter: counter.clone(),
            sleep_duration: Some(Duration::from_millis(50)),
            should_panic: false,
        };

        runner.spawn_loop(move |_ctx| async move {
            for _i in 0..5 {
                {
                    let mut locked = counter_for_task.lock().unwrap();
                    *locked += 1;
                }
                sleep(Duration::from_millis(10)).await;
            }
            Ok::<(), anyhow::Error>(())
        });

        // Let it run for a bit, then cancel
        sleep(Duration::from_millis(100)).await;
        ctx.cancel();

        // Wait for all tasks to complete
        while let Some(result) = join_set.join_next().await {
            result.unwrap().unwrap();
        }

        // Verify counter was incremented
        let final_count = *counter.lock().unwrap();
        assert!(
            final_count >= 5,
            "Counter should be at least 5, got {final_count}",
        );
    }

    #[tokio::test]
    async fn test_service_lifecycle() {
        let counter = Arc::new(Mutex::new(0));

        let mut service = TestService {
            counter: counter.clone(),
            sleep_duration: Some(Duration::from_millis(50)),
            should_panic: false,
        };

        let ctx = ServiceContext::new();
        let mut join_set = JoinSet::new();
        let runner = ServiceRunner::new(ctx.clone(), &mut join_set);

        // Start the service
        service.start(runner).await.unwrap();

        // Let it run for a bit
        sleep(Duration::from_millis(200)).await;

        // Verify the service is running
        let count_before_cancel = *counter.lock().unwrap();
        assert!(
            count_before_cancel > 0,
            "Service should have incremented counter"
        );

        // Cancel the service
        ctx.cancel();

        // Wait for all tasks to complete
        while let Some(result) = join_set.join_next().await {
            result.unwrap().unwrap();
        }

        // Verify service stopped
        let count_after_cancel = *counter.lock().unwrap();
        assert!(
            count_after_cancel >= count_before_cancel,
            "Counter should not decrease"
        );

        // Wait a bit more to ensure service is truly stopped
        let count_before_wait = count_after_cancel;
        sleep(Duration::from_millis(200)).await;
        let count_after_wait = *counter.lock().unwrap();

        assert_eq!(
            count_before_wait, count_after_wait,
            "Service should have stopped incrementing counter"
        );
    }

    #[tokio::test]
    async fn test_service_group() {
        let counter1 = Arc::new(Mutex::new(0));
        let counter2 = Arc::new(Mutex::new(0));

        let service1 = TestService {
            counter: counter1.clone(),
            sleep_duration: Some(Duration::from_millis(50)),
            should_panic: false,
        };

        let service2 = TestService {
            counter: counter2.clone(),
            sleep_duration: Some(Duration::from_millis(30)),
            should_panic: false,
        };

        let mut group = ServiceGroup::default()
            .with_critical(service1)
            .with_critical(service2);

        let ctx = ServiceContext::new();
        let mut join_set = JoinSet::new();
        let runner = ServiceRunner::new(ctx.clone(), &mut join_set);

        // Start service group
        group.start(runner).await.unwrap();

        // Let services run
        sleep(Duration::from_millis(200)).await;

        // Verify both services are running
        let count1 = *counter1.lock().unwrap();
        let count2 = *counter2.lock().unwrap();

        assert!(count1 > 0, "Service 1 should have incremented counter");
        assert!(count2 > 0, "Service 2 should have incremented counter");
        assert!(
            count2 > count1,
            "Service 2 should increment faster than Service 1"
        );

        // Cancel all services
        ctx.cancel();

        // Wait for all services to complete
        while let Some(result) = join_set.join_next().await {
            result.unwrap().unwrap();
        }

        // Verify all services stopped
        let count1_before = *counter1.lock().unwrap();
        let count2_before = *counter2.lock().unwrap();

        sleep(Duration::from_millis(200)).await;

        let count1_after = *counter1.lock().unwrap();
        let count2_after = *counter2.lock().unwrap();

        assert_eq!(count1_before, count1_after, "Service 1 should have stopped");
        assert_eq!(count2_before, count2_after, "Service 2 should have stopped");
    }

    #[tokio::test]
    async fn test_empty_service_group() {
        let mut group = ServiceGroup::default();

        let ctx = ServiceContext::new();
        let mut join_set = JoinSet::new();
        let runner = ServiceRunner::new(ctx.clone(), &mut join_set);

        // Start service group
        assert!(group.start(runner).await.is_err());
    }

    #[tokio::test]
    async fn test_aux_only_service_group() {
        let counter1 = Arc::new(Mutex::new(0));
        let counter2 = Arc::new(Mutex::new(0));

        let service1 = TestService {
            counter: counter1.clone(),
            sleep_duration: Some(Duration::from_millis(50)),
            should_panic: false,
        };

        let service2 = TestService {
            counter: counter2.clone(),
            sleep_duration: Some(Duration::from_millis(30)),
            should_panic: false,
        };

        let mut group = ServiceGroup::default()
            .with_auxiliary(service1)
            .with_auxiliary(service2);

        let ctx = ServiceContext::new();
        let mut join_set = JoinSet::new();
        let runner = ServiceRunner::new(ctx.clone(), &mut join_set);

        // Start service group
        assert!(group.start(runner).await.is_err());
    }

    #[tokio::test]
    async fn test_mixed_service_group() {
        let counter1 = Arc::new(Mutex::new(0));
        let counter2 = Arc::new(Mutex::new(0));

        let service1 = TestService {
            counter: counter1.clone(),
            sleep_duration: Some(Duration::from_millis(50)),
            should_panic: false,
        };

        let service2 = TestService {
            counter: counter2.clone(),
            sleep_duration: Some(Duration::from_millis(30)),
            should_panic: false,
        };

        let mut group = ServiceGroup::default()
            .with_critical(service1)
            .with_auxiliary(service2);

        let ctx = ServiceContext::new();
        let mut join_set = JoinSet::new();
        let runner = ServiceRunner::new(ctx.clone(), &mut join_set);

        // Start service group
        group.start(runner).await.unwrap();

        // Let services run
        sleep(Duration::from_millis(200)).await;

        // Verify both services are running
        let count1 = *counter1.lock().unwrap();
        let count2 = *counter2.lock().unwrap();

        assert!(count1 > 0, "Service 1 should have incremented counter");
        assert!(count2 > 0, "Service 2 should have incremented counter");
        assert!(
            count2 > count1,
            "Service 2 should increment faster than Service 1"
        );

        // Cancel all services
        ctx.cancel();

        // Wait for all services to complete
        while let Some(result) = join_set.join_next().await {
            result.unwrap().unwrap();
        }

        // Verify all services stopped
        let count1_before = *counter1.lock().unwrap();
        let count2_before = *counter2.lock().unwrap();

        sleep(Duration::from_millis(200)).await;

        let count1_after = *counter1.lock().unwrap();
        let count2_after = *counter2.lock().unwrap();

        assert_eq!(count1_before, count1_after, "Service 1 should have stopped");
        assert_eq!(count2_before, count2_after, "Service 2 should have stopped");
    }

    #[tokio::test]
    async fn test_auxiliary_service_failure() {
        let counter1 = Arc::new(Mutex::new(0));
        let counter2 = Arc::new(Mutex::new(0));

        let service1 = TestService {
            counter: counter1.clone(),
            sleep_duration: Some(Duration::from_millis(50)),
            should_panic: false,
        };

        let service2 = TestService {
            counter: counter2.clone(),
            sleep_duration: Some(Duration::from_millis(30)),
            should_panic: true,
        };

        let mut group = ServiceGroup::default()
            .with_critical(service1)
            .with_auxiliary(service2);

        let ctx = ServiceContext::new();
        let mut join_set = JoinSet::new();
        let runner = ServiceRunner::new(ctx.clone(), &mut join_set);

        // Start service group
        group.start(runner).await.unwrap();

        // Let services run
        sleep(Duration::from_millis(200)).await;

        // Verify both services are running
        let count1 = *counter1.lock().unwrap();
        let count2 = *counter2.lock().unwrap();

        assert!(count1 > 0, "Service 1 should have incremented counter");
        assert!(count2 == 0, "Service 2 should not have incremented counter");

        // Cancel all services
        ctx.cancel();

        // Wait for all services to complete
        while let Some(result) = join_set.join_next().await {
            result.unwrap().unwrap();
        }

        // Verify all services stopped
        let count1_before = *counter1.lock().unwrap();

        sleep(Duration::from_millis(200)).await;

        let count1_after = *counter1.lock().unwrap();

        assert_eq!(count1_before, count1_after, "Service 1 should have stopped");
    }

    #[tokio::test]
    #[should_panic(expected = "Service panic as requested")]
    async fn test_critical_service_failure() {
        let counter1 = Arc::new(Mutex::new(0));
        let counter2 = Arc::new(Mutex::new(0));

        let service1 = TestService {
            counter: counter1.clone(),
            sleep_duration: Some(Duration::from_millis(50)),
            should_panic: true,
        };

        let service2 = TestService {
            counter: counter2.clone(),
            sleep_duration: Some(Duration::from_millis(30)),
            should_panic: false,
        };

        let mut group = ServiceGroup::default()
            .with_critical(service1)
            .with_critical(service2);

        let ctx = ServiceContext::new();
        let mut join_set = JoinSet::new();
        let runner = ServiceRunner::new(ctx.clone(), &mut join_set);

        // Start service group
        group.start(runner).await.unwrap();

        // Let services run
        sleep(Duration::from_millis(200)).await;

        // Verify both services are running
        let count1 = *counter1.lock().unwrap();
        let count2 = *counter2.lock().unwrap();

        assert!(count1 == 0, "Service 1 should not have incremented counter");
        assert!(count2 > 0, "Service 2 should have incremented counter");

        // Cancel all services
        ctx.cancel();

        // Wait for all services to complete
        while let Some(result) = join_set.join_next().await {
            result.unwrap().unwrap();
        }
    }

    #[tokio::test]
    async fn test_service_lifecycle_with_controlled_shutdown() {
        // Create a counter to track executions
        let counter = Arc::new(Mutex::new(0));
        let counter_for_task = counter.clone();

        // Flag to track if shutdown completed
        let shutdown_completed = Arc::new(Mutex::new(false));
        let shutdown_completed_for_task = shutdown_completed.clone();

        let ctx = ServiceContext::new();
        let mut join_set = JoinSet::new();
        let mut runner = ServiceRunner::new(ctx.clone(), &mut join_set);

        // Spawn a service with controlled shutdown behavior
        runner.spawn_loop(move |inner_ctx| {
            let counter_inner = counter_for_task.clone();
            let shutdown_inner = shutdown_completed_for_task.clone();

            async move {
                // Run until cancelled
                while !inner_ctx.is_cancelled() {
                    {
                        let mut locked = counter_inner.lock().unwrap();
                        *locked += 1;
                    }
                    sleep(Duration::from_millis(10)).await;
                }

                // When cancelled, do proper shutdown
                *shutdown_inner.lock().unwrap() = true;
                Ok::<(), anyhow::Error>(())
            }
        });

        // Let the service run a bit
        sleep(Duration::from_millis(50)).await;

        // Record the counter value before cancellation
        let count_before = *counter.lock().unwrap();
        assert!(count_before > 0, "Service should have started");

        // Cancel and wait for complete shutdown
        ctx.cancel();

        // Wait for the service to properly shut down
        while let Some(result) = join_set.join_next().await {
            result.unwrap().unwrap();
        }

        // Verify the service did proper shutdown
        assert!(
            *shutdown_completed.lock().unwrap(),
            "Service should have completed its shutdown sequence"
        );

        // Verify the service stopped incrementing the counter
        let count_after = *counter.lock().unwrap();
        sleep(Duration::from_millis(50)).await;
        let final_count = *counter.lock().unwrap();

        assert_eq!(
            count_after, final_count,
            "Counter should not increase after shutdown"
        );
    }

    // This test verifies that services respond to context cancellation
    #[tokio::test]
    async fn test_service_cancellation_response() {
        // Create a counter to track executions
        let counter = Arc::new(Mutex::new(0));
        let counter_for_task = counter.clone();

        let ctx = ServiceContext::new();
        let mut join_set = JoinSet::new();
        let mut runner = ServiceRunner::new(ctx.clone(), &mut join_set);

        // Spawn a service that checks for cancellation
        runner.spawn_loop(move |inner_ctx| {
            let counter_inner = counter_for_task.clone();

            async move {
                let mut iterations = 0;

                // Run up to 200 iterations maximum, but will stop earlier if cancelled
                while iterations < 200 && !inner_ctx.is_cancelled() {
                    {
                        let mut locked = counter_inner.lock().unwrap();
                        *locked += 1;
                    }
                    iterations += 1;
                    sleep(Duration::from_millis(10)).await;
                }

                // Finish naturally or due to cancellation
                Ok::<(), anyhow::Error>(())
            }
        });

        // Let the service run briefly
        sleep(Duration::from_millis(30)).await;

        // Cancel the context
        ctx.cancel();

        // Wait for tasks to complete
        while let Some(result) = join_set.join_next().await {
            result.unwrap().unwrap();
        }

        // Verify the service ran at least a bit, but not all 100 iterations
        let final_count = *counter.lock().unwrap();
        assert!(final_count > 0, "Service should have run at least once");
        assert!(
            final_count < 100,
            "Service should have been cancelled before completing all iterations"
        );
    }

    #[tokio::test]
    #[should_panic(expected = "Service panic as requested")]
    async fn test_service_panic_propagation() {
        let counter = Arc::new(Mutex::new(0));

        // Create a service that will panic
        let service = TestService {
            counter: counter.clone(),
            sleep_duration: None,
            should_panic: true,
        };

        // Use start_and_drive_to_end which should propagate panics
        service.start_and_drive_to_end().await.unwrap();
    }

    #[tokio::test]
    async fn test_start_and_drive_to_end() {
        let counter = Arc::new(Mutex::new(0));

        // Create a service that completes successfully
        let service = TestService {
            counter: counter.clone(),
            sleep_duration: Some(Duration::from_millis(50)),
            should_panic: false,
        };

        // Use tokio::spawn to avoid blocking the test
        let handle = tokio::spawn(async move {
            // This future will complete when all service tasks are done
            let result = service.start_and_drive_to_end().await;
            assert!(result.is_ok(), "Service should complete successfully");
        });

        // Give the service time to run
        sleep(Duration::from_millis(200)).await;

        // Force the handle to timeout by timing out the test
        tokio::select! {
            _ = handle => {},
            _ = sleep(Duration::from_millis(300)) => {
                // This is expected since we never cancel the service
            }
        }

        // Verify the service ran
        let count = *counter.lock().unwrap();
        assert!(count > 0, "Service should have incremented counter");
    }
}
