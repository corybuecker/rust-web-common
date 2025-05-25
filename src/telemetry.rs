use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{metrics::SdkMeterProvider, trace::SdkTracerProvider};
use thiserror::Error;
use tracing::{info, level_filters::LevelFilter};
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::{Layer, Registry, layer::SubscriberExt, util::SubscriberInitExt};

pub struct TelemetryConfig {
    pub service_name: String,
    pub service_version: Option<String>,
    pub log_level: tracing::Level,
    pub metrics_endpoint: Option<String>,
    pub tracing_endpoint: Option<String>,
    pub protocol: opentelemetry_otlp::Protocol,
}

#[derive(Error, Debug)]
pub enum TelemetryError {
    #[error("Failed to create metric exporter: {0}")]
    MetricExporter(#[from] opentelemetry_otlp::ExporterBuildError),
    #[error("Invalid configuration: {0}")]
    Configuration(String),
    #[error("Provider shutdown failed: {0}")]
    Shutdown(String),
}

pub struct TelemetryBuilder {
    config: TelemetryConfig,
}

#[derive(Default)]
pub struct TelemetryProviders {
    pub meter_provider: Option<SdkMeterProvider>,
    pub tracer_provider: Option<SdkTracerProvider>,
}

impl Drop for TelemetryProviders {
    fn drop(&mut self) {
        info!("Shutting down telemetry providers...");

        if let Some(provider) = &self.meter_provider {
            if let Err(e) = provider.shutdown() {
                tracing::error!("Failed to shutdown meter provider: {}", e);
            }
        }

        if let Some(provider) = &self.tracer_provider {
            if let Err(e) = provider.shutdown() {
                tracing::error!("Failed to shutdown tracer provider: {}", e);
            }
        }
    }
}

#[derive(Clone)]
pub struct EnvironmentConfig {
    pub metrics_endpoint: Option<String>,
    pub tracing_endpoint: Option<String>,
    pub log_level: tracing::Level,
}

impl TelemetryBuilder {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            config: TelemetryConfig {
                service_name: service_name.into(),
                service_version: None,
                log_level: tracing::Level::INFO,
                metrics_endpoint: std::env::var("METRICS_ENDPOINT").ok(),
                tracing_endpoint: std::env::var("TRACING_ENDPOINT").ok(),
                protocol: opentelemetry_otlp::Protocol::HttpBinary,
            },
        }
    }

    pub fn with_log_level(mut self, level: tracing::Level) -> Self {
        self.config.log_level = level;
        self
    }

    pub fn with_metrics_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.metrics_endpoint = Some(endpoint.into());
        self
    }

    pub fn with_tracing_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.tracing_endpoint = Some(endpoint.into());
        self
    }

    pub fn build(self) -> Result<TelemetryProviders, TelemetryError> {
        let logging_layer = build_logging_layer()?;
        let service_name = self.config.service_name.clone();
        let mut layers: Vec<Box<dyn Layer<Registry> + Send + Sync>> = vec![logging_layer];
        let mut providers = TelemetryProviders::default();

        match self.config.metrics_endpoint {
            Some(endpoint) => {
                let provider = build_meter_provider(endpoint, service_name.clone())?;
                providers.meter_provider = Some(provider.clone());
                layers.push(build_metrics_exporter(provider)?);
            }
            None => {
                tracing::warn!("No metrics endpoint configured, metrics will not be exported.");
            }
        }

        match self.config.tracing_endpoint {
            Some(endpoint) => {
                let provider = build_tracer_provider(endpoint, service_name.clone())?;
                providers.tracer_provider = Some(provider.clone());
                layers.push(build_tracing_exporter(provider, service_name.clone())?);
            }
            None => {
                tracing::warn!("No tracer endpoint configured, traces will not be exported.");
            }
        }

        tracing_subscriber::registry().with(layers).init();

        Ok(providers)
    }
}

fn build_logging_layer() -> Result<Box<dyn Layer<Registry> + Send + Sync>, TelemetryError> {
    let env_log_level = std::env::var("LOG_LEVEL")
        .unwrap_or("info".to_string())
        .parse()
        .ok()
        .unwrap_or(LevelFilter::INFO);

    let target = tracing_subscriber::filter::Targets::new().with_default(env_log_level);

    Ok(tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_filter(target)
        .boxed())
}

fn build_meter_provider(
    endpoint: String,
    service_name: String,
) -> Result<SdkMeterProvider, TelemetryError> {
    let metrics_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
        .with_endpoint(endpoint)
        .build()
        .map_err(TelemetryError::MetricExporter)?;

    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_periodic_exporter(metrics_exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_service_name(service_name)
                .build(),
        )
        .build();

    Ok(meter_provider)
}

fn build_metrics_exporter(
    meter_provider: SdkMeterProvider,
) -> Result<Box<dyn Layer<Registry> + Send + Sync>, TelemetryError> {
    Ok(MetricsLayer::new(meter_provider).boxed())
}

fn build_tracer_provider(
    endpoint: String,
    service_name: String,
) -> Result<SdkTracerProvider, TelemetryError> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
        .with_endpoint(endpoint)
        .build()
        .map_err(TelemetryError::MetricExporter)?;

    Ok(SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_service_name(service_name)
                .build(),
        )
        .build())
}

fn build_tracing_exporter(
    tracer_provider: SdkTracerProvider,
    service_name: String,
) -> Result<Box<dyn Layer<Registry> + Send + Sync>, TelemetryError> {
    Ok(tracing_opentelemetry::layer()
        .with_tracer(tracer_provider.tracer(service_name))
        .boxed())
}
