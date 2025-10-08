# rust-web-common

`rust-web-common` provides shared utilities for Rust web applications, including telemetry (logging, metrics, and tracing) and HTML templating capabilities. This library is intended to help standardize and simplify observability setup and template rendering across multiple services.

## Features

- **Unified Telemetry Setup**: Configure logging, metrics, and tracing layers with minimal boilerplate.
- **Environment-Driven Configuration**: Easily adapt to environment-specific endpoints and log levels.
- **OpenTelemetry Integration**: Metrics and traces are exported using OpenTelemetry's OTLP protocol.
- **HTML Templating**: Handlebars-based template rendering with asset digest helper for cache busting.

## Modules

### Telemetry

The telemetry module provides a unified interface for setting up structured logging, metrics collection, and distributed tracing using the OpenTelemetry ecosystem.

#### Environment Variables

The following environment variables are used to configure telemetry at runtime:

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

#### Telemetry Usage

```rust
use rust_web_common::telemetry::TelemetryBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Basic setup - reads endpoints and log level from environment variables
    let mut telemetry = TelemetryBuilder::new("my-service");
    telemetry.init()?;

    // Or configure explicitly
    let mut telemetry = TelemetryBuilder::new("my-service")
        .with_log_level(tracing::Level::DEBUG)
        .with_metrics_endpoint("http://localhost:4318/v1/metrics")
        .with_tracing_endpoint("http://localhost:4318/v1/traces");

    telemetry.init()?;

    // Your application logic here
    tracing::info!("Application started");

    Ok(())
}
```

### Templating

The templating module provides a Handlebars-based template renderer with built-in helpers for common web application needs, such as asset cache busting.

#### Features

- **Handlebars Templates**: Full Handlebars templating support with strict mode enabled
- **Directory-based Templates**: Automatically loads templates from a specified directory
- **Asset Digest Helper**: Built-in `digest_asset` helper for cache-busted asset URLs
- **Context Management**: Thread-safe context management for template variables
- **Development Mode**: Automatic template reloading in development

#### Templating Usage

```rust
use rust_web_common::templating::{Renderer, to_json};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a renderer pointing to your templates directory
    let renderer = Renderer::new("templates".to_string())?;

    // Add data to the template context
    renderer.insert("title", "My Web App")?;
    renderer.insert("user", json!({
        "name": "User",
        "email": "user@example.com"
    }))?;

    // Render a template
    let html = renderer.render("index.hbs")?;
    println!("{}", html);

    Ok(())
}
```

#### Template Helpers

##### `digest_asset`

The `digest_asset` helper automatically appends a cache-busting query parameter to asset URLs:

```handlebars
<!-- In your template -->
<link rel="stylesheet" href="{{digest_asset 'styles.css'}}" />
<script src="{{digest_asset 'app.js'}}"></script>
```

This will render as:

```html
<link rel="stylesheet" href="/assets/styles.css?v=1703001234" />
<script src="/assets/app.js?v=1703001234"></script>
```

The cache key is automatically generated based on the application startup time.

#### Template Directory Structure

Templates should be organized in a directory structure that the renderer can discover:

```
templates/
├── index.hbs
├── layout.hbs
├── partials/
│   ├── header.hbs
│   └── footer.hbs
└── pages/
    ├── about.hbs
    └── contact.hbs
```

#### Error Handling

The templating module provides comprehensive error handling:

```rust
use rust_web_common::templating::{Renderer, RendererError};

match renderer.render("nonexistent.hbs") {
    Ok(html) => println!("{}", html),
    Err(RendererError::RenderError(e)) => {
        eprintln!("Template render error: {}", e);
    },
    Err(RendererError::TemplateError(e)) => {
        eprintln!("Template compilation error: {}", e);
    },
    Err(RendererError::ContextUpdateError) => {
        eprintln!("Failed to update template context");
    },
}
```

## Dependencies

- **handlebars**: Template engine with directory source support
- **opentelemetry**: Core OpenTelemetry functionality
- **opentelemetry-otlp**: OTLP exporter for metrics and traces
- **tracing**: Structured logging and instrumentation
- **serde_json**: JSON serialization for template context
- **thiserror**: Error handling utilities

## License

MIT

## Notes

- This README was written by AI.
