# rust-web-common

`rust-web-common` provides shared telemetry utilities (logging, metrics, and tracing) for Rust web applications. This library is intended to help standardize and simplify observability setup across multiple services, leveraging the OpenTelemetry ecosystem and `tracing` for structured logging.

## Features

- **Unified Telemetry Setup**: Configure logging, metrics, and tracing layers with minimal boilerplate.
- **Environment-Driven Configuration**: Easily adapt to environment-specific endpoints and log levels.
- **OpenTelemetry Integration**: Metrics and traces are exported using OpenTelemetry’s OTLP protocol.

## Environment Variables

The following environment variables are used to configure the library at runtime:

- **`LOG_LEVEL`**  
  Sets the logging verbosity.  
  Example values: `info`, `debug`, `warn`, `error`  
  _Default_: `info`

- **`METRICS_ENDPOINT`**  
  The URL or address to which metrics are exported.  
  If not set, metrics exporting will be disabled.

- **`TRACING_ENDPOINT`**  
  The URL or address to which traces are exported.  
  If not set, tracing exporting will be disabled.

## Example Usage

```rust
use rust_web_common::telemetry::TelemetryBuilder;

fn main() {
    // Reads endpoints and log level from environment variables
    let _telemetry = TelemetryBuilder::new("my-service")
        .build()
        .expect("Failed to initialize telemetry");
    // Your application logic here
}
```

## Intent

This library is designed to centralize the configuration of telemetry (logging, metrics, and tracing) for Rust-based web services, ensuring consistent observability practices and reducing duplicated setup code across projects.

## Notes

- This README was written by AI.

## License

MIT