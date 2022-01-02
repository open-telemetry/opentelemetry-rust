use futures::{
    future::FutureObj,
    task::{Spawn, SpawnError},
};
use tokio::runtime::Handle;

/// Tokio doesn't provide anything that implements `futures::task::Spawn`
/// (see <https://github.com/tokio-rs/tokio/issues/2018>).
/// `TokioSpawner` wraps a `tokio::runtime::Handle` and implements `futures::task::Spawn`.
///
/// ```no_run
/// # use opentelemetry_stackdriver::StackDriverExporter;
/// # use opentelemetry_stackdriver::tokio_adapter::TokioSpawner;
///
/// # let stackdriver_creds = std::path::Pathbuf::from("/path/to/creds.json");
/// let handle = tokio::runtime::Handle::current();
/// let spawner = tokio_adapter::TokioSpawner::new(handle);
/// let _ = opentelemetry_stackdriver::StackDriverExporter::connect(stackdriver_creds, "my_project", &spawner);
/// ```
#[derive(Debug, Clone)]
pub struct TokioSpawner {
    handle: Handle,
}

impl TokioSpawner {
    pub fn new(handle: Handle) -> Self {
        Self { handle }
    }
}

impl Spawn for TokioSpawner {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        self.handle.spawn(future);
        Ok(())
    }
}

impl From<Handle> for TokioSpawner {
    fn from(h: Handle) -> Self {
        Self::new(h)
    }
}
