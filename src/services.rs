/// Inspiration from:
/// <https://github.com/madara-alliance/madara/blob/main/crates/madara/primitives/utils/src/service.rs>
use std::{panic, time::Duration};

use anyhow::{anyhow, Context};
use futures::Future;
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
        drive_critical_joinset(join_set).await
    }
}

/// Wrapper around `JoinSet` and `ServiceContext` to enforce shutdown behavior
pub struct ServiceRunner<'a> {
    ctx: ServiceContext,
    join_set: &'a mut JoinSet<anyhow::Result<()>>,
}

impl<'a> ServiceRunner<'a> {
    pub fn new(ctx: ServiceContext, join_set: &'a mut JoinSet<anyhow::Result<()>>) -> Self {
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
    critical_services: Vec<Box<dyn Service>>,
    auxiliary_services: Vec<Box<dyn Service>>,
    critical_join_set: Option<JoinSet<anyhow::Result<()>>>,
    auxiliary_join_set: Option<JoinSet<anyhow::Result<()>>>,
}

impl ServiceGroup {
    pub fn new(
        critical_services: Vec<Box<dyn Service>>,
        auxiliary_services: Vec<Box<dyn Service>>,
    ) -> Self {
        let has_critical_services = !critical_services.is_empty();
        let has_auxiliary_services = !auxiliary_services.is_empty();

        Self {
            critical_services,
            auxiliary_services,
            critical_join_set: if has_critical_services {
                Some(JoinSet::default())
            } else {
                None
            },
            auxiliary_join_set: if has_auxiliary_services {
                Some(JoinSet::default())
            } else {
                None
            },
        }
    }

    pub fn push_critical(&mut self, service: impl Service) {
        if self.critical_join_set.is_none() {
            self.critical_join_set = Some(JoinSet::default());
        }
        self.critical_services.push(Box::new(service));
    }

    pub fn push_auxiliary(&mut self, service: impl Service) {
        if self.auxiliary_join_set.is_none() {
            self.auxiliary_join_set = Some(JoinSet::default());
        }
        self.auxiliary_services.push(Box::new(service));
    }

    #[must_use]
    pub fn with_critical(mut self, service: impl Service) -> Self {
        self.push_critical(service);
        self
    }

    #[must_use]
    pub fn with_auxiliary(mut self, service: impl Service) -> Self {
        self.push_auxiliary(service);
        self
    }
}

#[async_trait::async_trait]
impl Service for ServiceGroup {
    async fn start<'a>(&mut self, runner: ServiceRunner<'a>) -> anyhow::Result<()> {
        if self.critical_services.is_empty() {
            return Err(anyhow!("ServiceGroup started without any critical service"));
        }

        let mut own_critical_join_set = self
            .critical_join_set
            .take()
            .context("ServiceGroup has already been started")?;

        for service in &mut self.critical_services {
            let ctx = runner.ctx.clone();
            service
                .start(ServiceRunner::new(ctx, &mut own_critical_join_set))
                .await
                .context("Starting critical service")?;
        }

        if !self.auxiliary_services.is_empty() {
            let mut own_auxiliary_join_set = self
                .auxiliary_join_set
                .take()
                .context("ServiceGroup has already been started")?;

            for service in &mut self.auxiliary_services {
                let ctx = runner.ctx.clone();
                // Ignore start result for auxiliary services
                let _ = service
                    .start(ServiceRunner::new(ctx, &mut own_auxiliary_join_set))
                    .await;
            }

            runner.join_set.spawn(drive_critical_and_auxiliary_joinsets(
                own_critical_join_set,
                own_auxiliary_join_set,
            ));
        } else {
            runner
                .join_set
                .spawn(drive_critical_joinset(own_critical_join_set));
        };

        Ok(())
    }
}

async fn drive_critical_joinset(mut join_set: JoinSet<anyhow::Result<()>>) -> anyhow::Result<()> {
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

async fn drive_critical_and_auxiliary_joinsets(
    critical_join_set: JoinSet<anyhow::Result<()>>,
    mut auxiliary_join_set: JoinSet<anyhow::Result<()>>,
) -> anyhow::Result<()> {
    let (res_critical, _ret_auxiliary) = futures::future::join(
        drive_critical_joinset(critical_join_set),
        // Ignore result for auxiliary services
        async { while let Some(_result) = auxiliary_join_set.join_next().await {} },
    )
    .await;

    res_critical?;

    Ok(())
}
