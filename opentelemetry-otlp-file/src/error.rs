use std::io;
use std::path::PathBuf;

/// Errors that can occur while building an exporter.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ExporterBuildError {
    /// The file configured as the output could not be opened for appending.
    #[error("failed to open {} for appending: {source}", .path.display())]
    OpenFile {
        /// The configured file path.
        path: PathBuf,
        /// The error reported by the operating system.
        source: io::Error,
    },
}
