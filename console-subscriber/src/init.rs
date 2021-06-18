use crate::TasksLayer;
use std::thread;
use tokio::runtime;
use tracing_core::{subscriber::Subscriber, Dispatch};
use tracing_subscriber::{fmt, layer::Layered, prelude::*, EnvFilter, Layer, Registry};

type ConsoleSubscriberLayer =
    Layered<TasksLayer, Layered<EnvFilter, Layered<fmt::Layer<Registry>, Registry>>>;

/// Starts the console subscriber server on its own thread
///
/// This function represents the easiest way to get started using
/// tokio-console.
///
/// ## Configuration
///
/// Tokio console subscriber is configured with sensible defaults for most
/// use cases. If you need to tune these parameters, several environmental
/// configuration variables are available:
///
/// * `TOKIO_CONSOLE_RETENTION_SECS`: The number of seconds to accumulate
///   completed tracing data. Default: 60s.
/// * `TOKIO_CONSOLE_BIND`: a HOST:PORT description, such as
///   localhost:1234 or similar. Default: 127.0.0.1:6669
/// * `TOKIO_CONSOLE_PUBLISH_INTERVAL_MS`: The number of milliseconds to wait
///   between sending updates to the console. Default: 1000ms (1s)
/// * `RUST_LOG`: configure the tracing filter. Default: `tokio=trace`,
///   and any additional filtering directives will be appended to this
///   default. See [`EnvFilter`] for further information on the format
///   for this variable.
///
/// ## Adding additional layers
///
/// To add an additional [`Layer`], see [`init_with_layer`]
pub fn init() {
    init_inner(|console_subscriber| console_subscriber)
}

/// Starts the console subscriber server with an additional subscriber
/// layer, for example a log layer. This interface can be combined
/// with any of the environment configuration documented at [`init`].
pub fn init_with_layer<AdditionalLayer>(additional_layer: AdditionalLayer)
where
    AdditionalLayer: Layer<ConsoleSubscriberLayer> + Send + Sync + 'static,
{
    init_inner(move |console_subscriber| additional_layer.with_subscriber(console_subscriber))
}

fn init_inner<F, OutputLayer>(maybe_add_additional_layer: F)
where
    F: FnOnce(ConsoleSubscriberLayer) -> OutputLayer + Send + 'static,
    OutputLayer: Subscriber + Into<Dispatch>,
{
    thread::Builder::new()
        .name("console_subscriber".into())
        .spawn(move || {
            let (layer, server) = TasksLayer::builder().from_default_env().build();

            let filter =
                EnvFilter::from_default_env().add_directive("tokio=trace".parse().unwrap());

            let console_subscriber = tracing_subscriber::registry()
                .with(fmt::layer())
                .with(filter)
                .with(layer);

            maybe_add_additional_layer(console_subscriber).init();

            let runtime = runtime::Builder::new_current_thread()
                .enable_io()
                .enable_time()
                .build()
                .expect("console subscriber runtime initialization failed");

            runtime.block_on(async move {
                server
                    .serve()
                    .await
                    .expect("console subscriber server failed")
            });
        })
        .expect("console subscriber could not spawn thread");
}
