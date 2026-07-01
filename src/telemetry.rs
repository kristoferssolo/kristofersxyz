use tracing::Subscriber;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};

/// Create a new tracing subscriber.
///
/// # Panics
///
/// This function may panic if there is a bug in the `EnvFilter::from` implementation,
/// causing the `env_filter.into()` conversion to fail. This is highly unlikely.
pub fn get_subscriber<Sink>(
    name: &str,
    env_filter: &str,
    sink: Sink,
) -> impl Subscriber + Sync + Send
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name.into(), sink);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Initialize a global subscriber for tracing and logging.
///
/// # Panics
///
/// This function may panic in the following cases:
///
/// - If `LogTracer::init()` fails because the global logger has already been initialized.
///   This typically happens if `init_subscriber` is called more than once.
/// - If `set_global_default(subscriber)` fails because another subscriber has already been set,
///   or if there's an issue with the provided subscriber.
pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
