use tracing::Subscriber;
use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};

/// Builds a tracing subscriber writing human-readable events to `sink`.
///
/// `env_filter` is the fallback directive used when `RUST_LOG` is unset.
/// Spans report their own creation and closure, so an `#[instrument]`ed
/// function stays visible with its busy and idle time even when its body
/// logs nothing.
///
/// # Panics
///
/// Panics if `env_filter` cannot be converted into an [`EnvFilter`].
pub fn get_subscriber<Sink>(env_filter: &str, sink: Sink) -> impl Subscriber + Sync + Send
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = tracing_subscriber::fmt::layer()
        .with_writer(sink)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);
    Registry::default().with(env_filter).with(formatting_layer)
}

/// Installs the process-wide tracing subscriber and log bridge.
///
/// # Process termination
///
/// Aborts if another global logger or tracing subscriber is already installed.
pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send) {
    if LogTracer::init().is_err() {
        std::process::abort();
    }
    if set_global_default(subscriber).is_err() {
        std::process::abort();
    }
}
