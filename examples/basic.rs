use opentelemetry::sdk::trace::TracerProvider;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_stackdriver::{StackDriverExporter, YupAuthorizer;
use tracing::{span, Level};
use tracing_subscriber::prelude::*;

use std::{
  path::{Path, PathBuf},
  thread::sleep,
  time::Duration,
};

#[tokio::main]
async fn main() {
  let args = std::env::args().collect::<Vec<_>>();
  if args.len() < 2 {
    eprintln!("This example requires a path to your stackdriver json credentials as the first argument.");
    return;
  }
  init_tracing(&args[1]).await;
  span!(Level::INFO, "example_span").in_scope(|| {
    sleep(Duration::from_secs(2));
    span!(Level::INFO, "example_child_span").in_scope(|| {
      sleep(Duration::from_secs(2));
    });
  });
  sleep(Duration::from_secs(5));
}

async fn init_tracing(stackdriver_creds: impl AsRef<Path>) {
  let authorizer = YupAuthorizer::new(stackdriver_creds, PathBuf::from("tokens.json")).await.unwrap();
  let exporter = StackDriverExporter::connect(authorizer, &TokioSpawner, None, 5).await.unwrap();

  let provider = TracerProvider::builder().with_simple_exporter(exporter).build();
  tracing_subscriber::registry()
    .with(tracing_opentelemetry::layer().with_tracer(provider.get_tracer("tracing", None)))
    .with(tracing_subscriber::filter::LevelFilter::DEBUG)
    .with(tracing_subscriber::fmt::Layer::new().pretty())
    .try_init()
    .unwrap();
}

use futures::{
  future::FutureObj,
  task::{Spawn, SpawnError},
};

/// For some reason tokio decided not to implement `futures::task::Spawn` anywhere.
/// https://github.com/tokio-rs/tokio/issues/2018
/// So here's a little struct that will do so for you.
pub struct TokioSpawner;

impl Spawn for TokioSpawner {
  fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
    // TODO: check that executor is active; return SpawnError if not.
    tokio::runtime::Handle::current().spawn(future);
    Ok(())
  }
}
