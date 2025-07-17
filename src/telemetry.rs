use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{metrics::SdkMeterProvider, trace::SdkTracerProvider};
use thiserror::Error;
use tracing::{Subscriber, info, level_filters::LevelFilter};
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::{Layer, Registry, layer::SubscriberExt};

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
    #[error("Missing tracer provider")]
    MissingTracerProvider,
}

pub struct TelemetryBuilder {
    config: TelemetryConfig,
    meter_provider: Option<SdkMeterProvider>,
    tracer_provider: Option<SdkTracerProvider>,
}

impl Drop for TelemetryBuilder {
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
    pub fn init_tracing(&self) -> Result<(), TelemetryError> {
        match &self.tracer_provider {
            Some(provider) => {
                opentelemetry::global::set_tracer_provider(provider.to_owned());
                Ok(())
            }
            None => Err(TelemetryError::MissingTracerProvider),
        }
    }

    pub fn init_metering(&self) -> Result<(), TelemetryError> {
        match &self.meter_provider {
            Some(provider) => {
                opentelemetry::global::set_meter_provider(provider.to_owned());
                Ok(())
            }
            None => Err(TelemetryError::MissingTracerProvider),
        }
    }

    pub fn init_subscriber(&mut self) -> Result<(), TelemetryError> {
        let subscriber = self.build_registry()?;

        tracing::subscriber::set_global_default(subscriber)
            .map_err(|_e| TelemetryError::MissingTracerProvider)?;

        Ok(())
    }

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
            meter_provider: None,
            tracer_provider: None,
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

    fn build_registry(&mut self) -> Result<impl Subscriber + Send + Sync, TelemetryError> {
        let logging_layer = build_logging_layer()?;
        let service_name = self.config.service_name.clone();
        let mut layers: Vec<Box<dyn Layer<Registry> + Send + Sync>> = vec![logging_layer];

        if let Some(endpoint) = &self.config.metrics_endpoint {
            let provider = build_meter_provider(endpoint.to_owned(), service_name.clone())?;
            self.meter_provider = Some(provider.clone());
            layers.push(build_metrics_exporter(provider)?);
        }

        if let Some(endpoint) = &self.config.tracing_endpoint {
            let provider = build_tracer_provider(endpoint.to_owned(), service_name.clone())?;
            self.tracer_provider = Some(provider.clone());
            layers.push(build_tracing_exporter(provider, service_name.clone())?);
        }

        let registry = Registry::default().with(layers);

        Ok(registry)
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
