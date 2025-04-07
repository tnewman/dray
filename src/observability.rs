use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};
use tracing::Level;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub async fn init_observability() -> Observability {
    let tracer_provider = init_tracer_provider();

    let tracer = tracer_provider.tracer(env!("CARGO_PKG_NAME"));

    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env()
        .unwrap();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .with(OpenTelemetryLayer::new(tracer))
        .init();

    Observability::new(tracer_provider)
}

pub struct Observability {
    tracer_provider: SdkTracerProvider,
}

impl Observability {
    fn new(tracer_provider: SdkTracerProvider) -> Observability {
        Observability { tracer_provider }
    }
}

impl Drop for Observability {
    fn drop(&mut self) {
        if let Err(err) = self.tracer_provider.shutdown() {
            eprintln!("{err:?}");
        }
    }
}

fn init_tracer_provider() -> SdkTracerProvider {
    let exporter = SpanExporter::builder().with_tonic().build().unwrap();

    SdkTracerProvider::builder()
        .with_resource(resource())
        .with_batch_exporter(exporter)
        .build()
}

fn resource() -> Resource {
    Resource::builder()
        .with_attribute(KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")))
        .with_attribute(KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")))
        .build()
}
