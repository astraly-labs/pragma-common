/// Inspiration from:
/// <https://github.com/madara-alliance/madara/blob/main/crates/madara/primitives/utils/src/service.rs>
use anyhow::Context;
use futures::Future;
use std::{panic, time::Duration};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

/// Maximum duration a service is allowed to take to shutdown, after which it
/// will be forcefully cancelled
pub const SERVICE_GRACE_PERIOD: Duration = Duration::from_secs(10);

/// Provides a way to manage service state and lifecycle
#[derive(Clone)]
pub struct ServiceContext {
    pub token: CancellationToken,
}

impl Default for ServiceContext {
    fn default() -> Self {
        Self {
            token: CancellationToken::new(),
        }
    }
}

impl ServiceContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Stops all services under this context
    pub fn cancel(&self) {
        self.token.cancel();
    }

    /// Returns true if this context has been cancelled
    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    /// Runs a future until the service is cancelled
    pub async fn run_until_cancelled<T, F>(&self, f: F) -> Option<T>
    where
        T: Sized + Send + Sync,
        F: Future<Output = T>,
    {
        tokio::select! {
            res = f => Some(res),
            () = self.token.cancelled() => None
        }
    }
}

/// Core trait for implementing services
#[async_trait::async_trait]
pub trait Service: 'static + Send + Sync {
    /// Start the service. Default implementation does nothing.
    async fn start<'a>(&mut self, _runner: ServiceRunner<'a>) -> anyhow::Result<()> {
        Ok(())
    }

    /// Helper to start and drive a service to completion
    async fn start_and_drive_to_end(mut self) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        let ctx = ServiceContext::new();
        let mut join_set = JoinSet::new();
        let runner = ServiceRunner::new(ctx, &mut join_set);

        self.start(runner).await.context("Starting service")?;
        drive_joinset(join_set).await
    }
}

/// Wrapper around `JoinSet` and `ServiceContext` to enforce shutdown behavior
pub struct ServiceRunner<'a> {
    ctx: ServiceContext,
    join_set: &'a mut JoinSet<anyhow::Result<()>>,
}

impl<'a> ServiceRunner<'a> {
    pub const fn new(ctx: ServiceContext, join_set: &'a mut JoinSet<anyhow::Result<()>>) -> Self {
        Self { ctx, join_set }
    }

    /// Spawn a service loop that handles graceful shutdown
    pub fn spawn_loop<F, E>(&mut self, runner: impl FnOnce(ServiceContext) -> F + Send + 'static)
    where
        F: Future<Output = Result<(), E>> + Send + 'static,
        E: Into<anyhow::Error> + Send,
    {
        let ctx = self.ctx.clone();
        self.join_set.spawn(async move {
            tokio::select! {
                res = runner(ctx.clone()) => res.map_err(Into::into)?,
                () = async {
                    ctx.token.cancelled().await;
                    tokio::time::sleep(SERVICE_GRACE_PERIOD).await;
                } => {}
            }
            Ok(())
        });
    }
}

/// A group of services that can be started together
#[derive(Default)]
pub struct ServiceGroup {
    services: Vec<Box<dyn Service>>,
    join_set: Option<JoinSet<anyhow::Result<()>>>,
}

impl ServiceGroup {
    pub fn new(services: Vec<Box<dyn Service>>) -> Self {
        Self {
            services,
            join_set: Some(JoinSet::default()),
        }
    }

    pub fn push(&mut self, service: impl Service) {
        if self.join_set.is_none() {
            self.join_set = Some(JoinSet::default());
        }
        self.services.push(Box::new(service));
    }

    #[must_use]
    pub fn with(mut self, service: impl Service) -> Self {
        self.push(service);
        self
    }
}

#[async_trait::async_trait]
impl Service for ServiceGroup {
    async fn start<'a>(&mut self, runner: ServiceRunner<'a>) -> anyhow::Result<()> {
        let mut own_join_set = self
            .join_set
            .take()
            .expect("Service has already been started");

        for service in &mut self.services {
            let ctx = runner.ctx.clone();
            service
                .start(ServiceRunner::new(ctx, &mut own_join_set))
                .await
                .context("Starting service")?;
        }

        runner.join_set.spawn(drive_joinset(own_join_set));
        Ok(())
    }
}

async fn drive_joinset(mut join_set: JoinSet<anyhow::Result<()>>) -> anyhow::Result<()> {
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(result) => result?,
            Err(panic_error) if panic_error.is_panic() => {
                panic::resume_unwind(panic_error.into_panic());
            }
            Err(_) => {}
        }
    }
    Ok(())
}
