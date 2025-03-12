use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    runtime::AsyncStd,
    trace::{self, SdkTracerProvider, span_processor_with_async_runtime::BatchSpanProcessor},
};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

#[async_std::main]
async fn main() {
    init_tracing();

    loop {
        do_span()
    }
}

#[tracing::instrument(
    name = "do_span",
    skip_all,
    fields(
        my_long_field = {
            // 1MB field
            "a".repeat(1_000_000)
        }
    ),
)]
fn do_span() {}

fn init_tracing() {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317".to_string()) // Unreachable host to demonstrate the error
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .build()
        .unwrap_or_else(|e| panic!("failed to initialize `tracing`: {e}"));

    let provider = SdkTracerProvider::builder()
        .with_span_processor(
            BatchSpanProcessor::builder(exporter, AsyncStd)
                .with_batch_config(
                    // Limit the batch size to reach OOM faster.
                    trace::BatchConfigBuilder::default()
                        .with_max_concurrent_exports(1)
                        .with_max_export_batch_size(1)
                        .build(),
                )
                .build(),
        )
        .build();

    tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("otel")))
        .init();
}
