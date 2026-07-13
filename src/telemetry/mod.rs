use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{global, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use tracing::level_filters::LevelFilter;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(thiserror::Error, Debug)]
pub enum TelemetryError {
    #[error("trace error: {0}")]
    TraceError(#[from] opentelemetry_sdk::trace::TraceError),
    #[error("try init error: {0}")]
    TryInitError(#[from] tracing_subscriber::util::TryInitError),
    #[error("metrics error: {0}")]
    MetricsError(#[from] opentelemetry_sdk::metrics::MetricError),
    #[error("exporter build error: {0}")]
    ExporterBuildError(#[from] opentelemetry_otlp::ExporterBuildError),
    #[error("sdk error: {0}")]
    SdkError(#[from] opentelemetry_sdk::error::OTelSdkError),
}

pub struct ProviderSet {
    pub tracer_provider: Option<SdkTracerProvider>,
    pub logger_provider: Option<SdkLoggerProvider>,
    pub metrics_provider: Option<SdkMeterProvider>,
    shutdown_called: Arc<AtomicBool>,
}

impl ProviderSet {
    pub fn shutdown(&mut self) -> Result<(), TelemetryError> {
        self.shutdown_called.store(true, Ordering::Release);
        if let Some(tracer_provider) = self.tracer_provider.take() {
            tracer_provider.shutdown()?;
        }
        if let Some(logger_provider) = self.logger_provider.take() {
            logger_provider.shutdown()?;
        }
        if let Some(metrics_provider) = self.metrics_provider.take() {
            metrics_provider.shutdown()?;
        }
        Ok(())
    }
}

pub fn init_telemetry(
    app_name: &str,
    collection_endpoint: Option<String>,
) -> Result<ProviderSet, TelemetryError> {
    let tracing_subscriber = tracing_subscriber::registry().with(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::DEBUG.into())
            .from_env_lossy(),
    );

    let shutdown_called = Arc::new(AtomicBool::new(false));
    if let Some(endpoint) = collection_endpoint {
        let traces_exporter = std::env::var("OTEL_TRACES_EXPORTER").ok();
        let logs_exporter = std::env::var("OTEL_LOGS_EXPORTER").ok();
        let metrics_exporter = std::env::var("OTEL_METRICS_EXPORTER").ok();

        let tracer_provider = signal_export_enabled(traces_exporter.as_deref())
            .then(|| init_tracer_provider(app_name, &endpoint))
            .transpose()?;
        let logger_provider = signal_export_enabled(logs_exporter.as_deref())
            .then(|| init_logs_provider(app_name, &endpoint))
            .transpose()?;
        let metrics_provider = signal_export_enabled(metrics_exporter.as_deref())
            .then(|| init_meter_provider(app_name, &endpoint))
            .transpose()?;

        let trace_layer = tracer_provider.as_ref().map(|provider| {
            OpenTelemetryLayer::new(provider.tracer(format!("{app_name}-subscriber")))
        });
        let logs_layer = logger_provider
            .as_ref()
            .map(OpenTelemetryTracingBridge::new);

        tracing_subscriber
            .with(tracing_subscriber::fmt::layer())
            .with(trace_layer)
            .with(logs_layer)
            .try_init()?;

        Ok(ProviderSet {
            tracer_provider,
            logger_provider,
            metrics_provider,
            shutdown_called,
        })
    } else {
        // Ignore spans when collection_endpoint is None
        let filter = tracing_subscriber::filter::FilterFn::new(|metadata| !metadata.is_span());
        tracing_subscriber
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(false)
                    .with_file(true)
                    .with_line_number(true)
                    .compact()
                    .with_span_events(FmtSpan::NONE),
            )
            .try_init()?;

        Ok(ProviderSet {
            tracer_provider: None,
            logger_provider: None,
            metrics_provider: None,
            shutdown_called,
        })
    }
}

fn signal_export_enabled(exporter: Option<&str>) -> bool {
    !matches!(exporter, Some("none"))
}

fn init_tracer_provider(
    app_name: &str,
    collection_endpoint: &str,
) -> Result<SdkTracerProvider, TelemetryError> {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(collection_endpoint)
        .build()?;

    let resource = Resource::builder()
        .with_attribute(KeyValue::new(
            SERVICE_NAME,
            format!("{app_name}-trace-service"),
        ))
        .build();

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    global::set_tracer_provider(provider.clone());
    Ok(provider)
}

fn init_logs_provider(
    app_name: &str,
    collection_endpoint: &str,
) -> Result<SdkLoggerProvider, TelemetryError> {
    let exporter = LogExporter::builder()
        .with_tonic()
        .with_endpoint(collection_endpoint)
        .build()?;

    let resource = Resource::builder()
        .with_attribute(KeyValue::new(
            SERVICE_NAME,
            format!("{app_name}-logs-service"),
        ))
        .build();

    let logger_provider = SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    Ok(logger_provider)
}

pub fn init_meter_provider(
    app_name: &str,
    collection_endpoint: &str,
) -> Result<SdkMeterProvider, TelemetryError> {
    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_endpoint(collection_endpoint)
        .build()?;

    let reader = PeriodicReader::builder(exporter)
        .with_interval(std::time::Duration::from_secs(5))
        .build();

    let resource = Resource::builder()
        .with_attribute(KeyValue::new(
            SERVICE_NAME,
            format!("{app_name}-meter-service"),
        ))
        .build();

    let metrics_provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource)
        .build();

    global::set_meter_provider(metrics_provider.clone());

    Ok(metrics_provider)
}

#[cfg(test)]
mod tests {
    use super::signal_export_enabled;

    #[test]
    fn none_disables_a_signal_exporter_without_changing_defaults() {
        assert!(!signal_export_enabled(Some("none")));
        assert!(signal_export_enabled(None));
        assert!(signal_export_enabled(Some("")));
        assert!(signal_export_enabled(Some("otlp")));
    }
}
