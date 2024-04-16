pub use log::{error, info, warn};

use tracing::{Level, Subscriber};
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::*, registry::Registry, EnvFilter};

pub fn initialize() {
    let log_plugin = LogPlugin::default();

    let finished_subscriber;
    let default_filter = { format!("{},{}", log_plugin.level, log_plugin.filter) };
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&default_filter))
        .unwrap();
    let subscriber = Registry::default().with(filter_layer);

    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    {
        let fmt_layer = tracing_subscriber::fmt::Layer::default().with_writer(std::io::stderr);

        let subscriber = subscriber.with(fmt_layer);

        if let Some(update_subscriber) = log_plugin.update_subscriber {
            finished_subscriber = update_subscriber(Box::new(subscriber));
        } else {
            finished_subscriber = Box::new(subscriber);
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        finished_subscriber = subscriber.with(tracing_wasm::WASMLayer::new(
            tracing_wasm::WASMLayerConfig::default(),
        ));
    }

    let logger_already_set = LogTracer::init().is_err();
    let subscriber_already_set =
        tracing::subscriber::set_global_default(finished_subscriber).is_err();

    match (logger_already_set, subscriber_already_set) {
        (true, true) => tracing::error!(
                "Could not set global logger and tracing subscriber as they are already set. Consider disabling LogPlugin."
            ),
        (true, false) => tracing::error!("Could not set global logger as it is already set. Consider disabling LogPlugin."),
        (false, true) => tracing::error!("Could not set global tracing subscriber as it is already set. Consider disabling LogPlugin."),
        (false, false) => (),
    }
}

pub struct LogPlugin {
    /// Filters logs using the [`EnvFilter`] format
    pub filter: String,

    /// Filters out logs that are "less than" the given level.
    /// This can be further filtered using the `filter` setting.
    pub level: Level,

    /// Optionally apply extra transformations to the tracing subscriber.
    /// For example add [`Layers`](tracing_subscriber::layer::Layer)
    pub update_subscriber: Option<fn(BoxedSubscriber) -> BoxedSubscriber>,
}

/// Alias for a boxed [`Subscriber`].
pub type BoxedSubscriber = Box<dyn Subscriber + Send + Sync + 'static>;

impl Default for LogPlugin {
    fn default() -> Self {
        Self {
            filter: "wgpu=error,naga=warn".to_string(),
            level: Level::INFO,
            update_subscriber: None,
        }
    }
}
