use opentelemetry::trace::TracerProvider;
use opentelemetry::{global, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::logs::{BatchConfig, LoggerProvider};
use opentelemetry_sdk::metrics::reader::DefaultTemporalitySelector;
use opentelemetry_sdk::metrics::{MeterProviderBuilder, PeriodicReader};
use opentelemetry_sdk::{runtime, trace::BatchConfigBuilder};
use opentelemetry_sdk::{
    trace::{Config, Tracer},
    Resource,
};
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
    TraceError(#[from] opentelemetry::trace::TraceError),
    #[error("try init error: {0}")]
    TryInitError(#[from] tracing_subscriber::util::TryInitError),
    #[error("metrics error: {0}")]
    MetricsError(#[from] opentelemetry::metrics::MetricsError),
    #[error("log error: {0}")]
    LogError(#[from] opentelemetry::logs::LogError),
}

pub fn init_telemetry(
    app_name: &str,
    collection_endpoint: Option<String>,
) -> Result<(), TelemetryError> {
    let tracing_subscriber = tracing_subscriber::registry().with(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::DEBUG.into())
            .from_env_lossy(),
    );

    if let Some(endpoint) = collection_endpoint {
        let tracer_provider = init_tracer_provider(app_name, &endpoint);
        let logger_provider = init_logs_provider(app_name, &endpoint)?;
        init_meter_provider(app_name, &endpoint)?;

        tracing_subscriber
            .with(tracing_subscriber::fmt::layer())
            .with(OpenTelemetryLayer::new(tracer_provider))
            .with(OpenTelemetryTracingBridge::new(&logger_provider))
            .try_init()?;
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
    }

    Ok(())
}

fn init_tracer_provider(app_name: &str, collection_endpoint: &str) -> Tracer {
    // Set a custom error handler to log OpenTelemetry errors with timestamps
    global::set_error_handler(|error| {
        tracing::error!(error = %error, "OpenTelemetry error occurred");
    })
    .expect("Failed to set error handler");

    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_batch_config(
            BatchConfigBuilder::default()
                .with_max_queue_size(4096)
                .build(),
        )
        .with_trace_config(
            Config::default().with_resource(Resource::new(vec![KeyValue::new(
                SERVICE_NAME,
                format!("{app_name}-trace-service"),
            )])),
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(collection_endpoint),
        )
        .install_batch(runtime::Tokio)
        .expect("Failed to install tracer provider");

    global::set_tracer_provider(provider.clone());
    provider.tracer(format!("{app_name}-subscriber"))
}

fn init_logs_provider(
    app_name: &str,
    collection_endpoint: &str,
) -> Result<LoggerProvider, TelemetryError> {
    let logger = opentelemetry_otlp::new_pipeline()
        .logging()
        .with_batch_config(BatchConfig::default())
        .with_resource(Resource::new(vec![KeyValue::new(
            SERVICE_NAME,
            format!("{app_name}-logs-service"),
        )]))
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(collection_endpoint),
        )
        .install_batch(runtime::Tokio)?;

    Ok(logger)
}

pub fn init_meter_provider(
    app_name: &str,
    collection_endpoint: &str,
) -> Result<(), TelemetryError> {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(collection_endpoint)
        .build_metrics_exporter(Box::new(DefaultTemporalitySelector::new()))?;

    let reader = PeriodicReader::builder(exporter, runtime::Tokio)
        .with_interval(std::time::Duration::from_secs(5))
        .build();

    let metrics_provider = MeterProviderBuilder::default()
        .with_reader(reader)
        .with_resource(Resource::new(vec![KeyValue::new(
            SERVICE_NAME,
            format!("{app_name}-meter-service"),
        )]))
        .build();

    global::set_meter_provider(metrics_provider);

    Ok(())
}
