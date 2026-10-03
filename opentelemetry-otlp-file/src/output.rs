use std::fmt;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use serde::Serialize;

use crate::ExporterBuildError;

/// The destination configured on an exporter builder.
#[derive(Default)]
pub(crate) enum Output {
    #[default]
    Stdout,
    File(PathBuf),
    Writer(Box<dyn Write + Send>),
}

impl Output {
    /// Opens the destination. Files are created if missing and always appended to.
    pub(crate) fn open(self) -> Result<JsonLinesWriter, ExporterBuildError> {
        let (destination, writer): (Destination, Box<dyn Write + Send>) = match self {
            Output::Stdout => (Destination::Stdout, Box::new(io::stdout())),
            Output::File(path) => match OpenOptions::new().create(true).append(true).open(&path) {
                Ok(file) => (Destination::File(path), Box::new(file)),
                Err(source) => return Err(ExporterBuildError::OpenFile { path, source }),
            },
            Output::Writer(writer) => (Destination::Writer, writer),
        };
        Ok(JsonLinesWriter {
            destination,
            writer: Mutex::new(Some(writer)),
        })
    }
}

impl fmt::Debug for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Output::Stdout => f.write_str("Stdout"),
            Output::File(path) => f.debug_tuple("File").field(path).finish(),
            Output::Writer(_) => f.write_str("Writer"),
        }
    }
}

#[derive(Debug)]
enum Destination {
    Stdout,
    File(PathBuf),
    Writer,
}

impl fmt::Display for Destination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Destination::Stdout => f.write_str("stdout"),
            Destination::File(path) => write!(f, "{}", path.display()),
            Destination::Writer => f.write_str("the configured writer"),
        }
    }
}

/// Writes each exported batch as one line of OTLP JSON.
pub(crate) struct JsonLinesWriter {
    destination: Destination,
    /// `None` once the exporter has been shut down.
    writer: Mutex<Option<Box<dyn Write + Send>>>,
}

impl JsonLinesWriter {
    pub(crate) fn stdout() -> Self {
        JsonLinesWriter {
            destination: Destination::Stdout,
            writer: Mutex::new(Some(Box::new(io::stdout()))),
        }
    }

    /// Serializes `data` as compact JSON, appends a newline and flushes.
    ///
    /// The line is serialized up front and handed to a single `write_all`
    /// call. `Stdout` holds its lock for the whole call, so the line cannot
    /// interleave with `println!` output from other threads.
    pub(crate) fn write<T: Serialize>(&self, data: &T) -> OTelSdkResult {
        let mut line = serde_json::to_vec(data).map_err(|err| {
            OTelSdkError::InternalFailure(format!("failed to serialize OTLP JSON: {err}"))
        })?;
        line.push(b'\n');
        self.with_writer(|writer| {
            writer.write_all(&line)?;
            writer.flush()
        })
    }

    /// Returns an error if the exporter has been shut down.
    pub(crate) fn ensure_open(&self) -> OTelSdkResult {
        self.with_writer(|_| Ok(()))
    }

    // `LogExporter` has no `force_flush`.
    #[cfg(any(feature = "trace", feature = "metrics"))]
    pub(crate) fn flush(&self) -> OTelSdkResult {
        self.with_writer(|writer| writer.flush())
    }

    /// Flushes and drops the writer, which closes a file opened by the exporter.
    pub(crate) fn shutdown(&self) -> OTelSdkResult {
        let mut writer = self.lock()?.take().ok_or(OTelSdkError::AlreadyShutdown)?;
        writer.flush().map_err(|err| self.io_error(err))
    }

    fn with_writer(&self, f: impl FnOnce(&mut dyn Write) -> io::Result<()>) -> OTelSdkResult {
        let mut guard = self.lock()?;
        let writer = guard.as_mut().ok_or(OTelSdkError::AlreadyShutdown)?;
        f(writer).map_err(|err| self.io_error(err))
    }

    fn lock(&self) -> Result<MutexGuard<'_, Option<Box<dyn Write + Send>>>, OTelSdkError> {
        self.writer.lock().map_err(|_| {
            OTelSdkError::InternalFailure(format!(
                "a previous write to {} panicked",
                self.destination
            ))
        })
    }

    fn io_error(&self, err: io::Error) -> OTelSdkError {
        OTelSdkError::InternalFailure(format!(
            "failed to write OTLP JSON to {}: {err}",
            self.destination
        ))
    }
}

impl fmt::Debug for JsonLinesWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JsonLinesWriter")
            .field("destination", &self.destination)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::io::ErrorKind;
    use std::sync::Arc;

    /// A cloneable in-memory writer for inspecting what an exporter wrote.
    #[derive(Clone, Debug, Default)]
    pub(crate) struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

    impl SharedBuffer {
        pub(crate) fn contents(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
        }

        /// Returns every line written so far, checking that each one is complete.
        pub(crate) fn lines(&self) -> Vec<String> {
            let contents = self.contents();
            assert!(
                contents.is_empty() || contents.ends_with('\n'),
                "the last line is not terminated: {contents:?}"
            );
            contents.lines().map(ToOwned::to_owned).collect()
        }
    }

    impl Write for SharedBuffer {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[derive(Debug, PartialEq)]
    enum Call {
        Write(String),
        Flush,
        Drop,
    }

    /// Records the calls an exporter makes on its writer.
    #[derive(Clone, Default)]
    struct RecordingWriter(Arc<Mutex<Vec<Call>>>);

    impl RecordingWriter {
        fn take_calls(&self) -> Vec<Call> {
            std::mem::take(&mut *self.0.lock().unwrap())
        }
    }

    impl Write for RecordingWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let data = String::from_utf8(buf.to_vec()).unwrap();
            self.0.lock().unwrap().push(Call::Write(data));
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.0.lock().unwrap().push(Call::Flush);
            Ok(())
        }
    }

    impl Drop for RecordingWriter {
        fn drop(&mut self) {
            self.0.lock().unwrap().push(Call::Drop);
        }
    }

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("disk full"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn open(writer: impl Write + Send + 'static) -> JsonLinesWriter {
        Output::Writer(Box::new(writer)).open().unwrap()
    }

    #[test]
    fn writes_each_value_as_one_compact_line_then_flushes() {
        let recorder = RecordingWriter::default();
        let output = open(recorder.clone());

        output
            .write(&json!({"name": "multi\nline", "values": [1, 2]}))
            .unwrap();

        assert_eq!(
            recorder.take_calls(),
            vec![
                Call::Write("{\"name\":\"multi\\nline\",\"values\":[1,2]}\n".to_owned()),
                Call::Flush,
            ]
        );
    }

    /// Accepts at most a few bytes per call, so `write_all` needs many calls per line.
    struct TrickleWriter(SharedBuffer);

    impl Write for TrickleWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.write(&buf[..buf.len().min(7)])
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn concurrent_writes_do_not_interleave() {
        let buffer = SharedBuffer::default();
        let output = Arc::new(open(TrickleWriter(buffer.clone())));

        let threads: Vec<_> = (0..8)
            .map(|thread| {
                let output = Arc::clone(&output);
                std::thread::spawn(move || {
                    for i in 0..50 {
                        let padding = "x".repeat(i * 3);
                        output
                            .write(&json!({"thread": thread, "i": i, "padding": padding}))
                            .unwrap();
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }

        let lines = buffer.lines();
        assert_eq!(lines.len(), 8 * 50);
        for line in lines {
            let value: serde_json::Value = serde_json::from_str(&line).unwrap();
            assert_eq!(
                value["padding"].as_str().unwrap().len(),
                value["i"].as_u64().unwrap() as usize * 3
            );
        }
    }

    #[test]
    fn shutdown_flushes_and_drops_the_writer() {
        let recorder = RecordingWriter::default();
        let output = open(recorder.clone());

        output.shutdown().unwrap();

        assert_eq!(recorder.take_calls(), vec![Call::Flush, Call::Drop]);
        assert!(matches!(
            output.write(&json!({})),
            Err(OTelSdkError::AlreadyShutdown)
        ));
        #[cfg(any(feature = "trace", feature = "metrics"))]
        assert!(matches!(output.flush(), Err(OTelSdkError::AlreadyShutdown)));
        assert!(matches!(
            output.ensure_open(),
            Err(OTelSdkError::AlreadyShutdown)
        ));
        assert!(matches!(
            output.shutdown(),
            Err(OTelSdkError::AlreadyShutdown)
        ));
        assert_eq!(recorder.take_calls(), vec![]);
    }

    #[test]
    fn reports_write_errors() {
        let output = open(FailingWriter);

        match output.write(&json!({})) {
            Err(OTelSdkError::InternalFailure(message)) => {
                assert_eq!(
                    message,
                    "failed to write OTLP JSON to the configured writer: disk full"
                );
            }
            other => panic!("unexpected result: {other:?}"),
        }
    }

    #[test]
    fn creates_a_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("traces.jsonl");
        let output = Output::File(path.clone()).open().unwrap();

        output.write(&json!({"n": 1})).unwrap();
        output.shutdown().unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "{\"n\":1}\n");
    }

    #[test]
    fn appends_to_an_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("traces.jsonl");
        fs::write(&path, "{\"n\":1}\n").unwrap();

        let output = Output::File(path.clone()).open().unwrap();
        output.write(&json!({"n": 2})).unwrap();
        output.shutdown().unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "{\"n\":1}\n{\"n\":2}\n");
    }

    #[test]
    fn fails_to_open_a_file_in_a_missing_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing").join("traces.jsonl");

        match Output::File(path.clone()).open() {
            Err(ExporterBuildError::OpenFile {
                path: error_path,
                source,
            }) => {
                assert_eq!(error_path, path);
                assert_eq!(source.kind(), ErrorKind::NotFound);
            }
            other => panic!("unexpected result: {other:?}"),
        }
    }

    #[test]
    fn debug_output_names_the_destination() {
        assert_eq!(
            format!("{:?}", JsonLinesWriter::stdout()),
            "JsonLinesWriter { destination: Stdout, .. }"
        );
        assert_eq!(
            format!("{:?}", Output::File(PathBuf::from("traces.jsonl"))),
            "File(\"traces.jsonl\")"
        );
        assert_eq!(
            format!("{:?}", Output::Writer(Box::new(io::sink()))),
            "Writer"
        );
    }
}
