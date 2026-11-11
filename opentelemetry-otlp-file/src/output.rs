use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use serde::Serialize;

use crate::ExporterBuildError;

/// The output stream configured on an exporter builder.
#[derive(Default)]
pub(crate) enum OutputStream {
    #[default]
    Stdout,
    File(PathBuf),
    Writer(Box<dyn Write + Send>),
}

impl OutputStream {
    /// Opens the stream. A file is created if it is missing and always appended to.
    pub(crate) fn open(self) -> Result<JsonLinesWriter, ExporterBuildError> {
        let name = self.to_string();
        let sink = match self {
            OutputStream::Stdout => Sink::Stdout,
            OutputStream::File(path) => {
                match OpenOptions::new().create(true).append(true).open(&path) {
                    // Syncing a pipe or a device, such as `/dev/stdout`, fails.
                    Ok(file) if file.metadata().is_ok_and(|metadata| metadata.is_file()) => {
                        Sink::File(file)
                    }
                    Ok(file) => Sink::Writer(Box::new(file)),
                    Err(source) => return Err(ExporterBuildError::OpenFile { path, source }),
                }
            }
            OutputStream::Writer(writer) => Sink::Writer(writer),
        };
        Ok(JsonLinesWriter::new(name, sink))
    }
}

/// Names the stream in error messages.
impl fmt::Display for OutputStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputStream::Stdout => f.write_str("stdout"),
            OutputStream::File(path) => write!(f, "{}", path.display()),
            OutputStream::Writer(_) => f.write_str("the configured writer"),
        }
    }
}

impl fmt::Debug for OutputStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputStream::Stdout => f.write_str("Stdout"),
            OutputStream::File(path) => f.debug_tuple("File").field(path).finish(),
            OutputStream::Writer(_) => f.write_str("Writer"),
        }
    }
}

/// Writes each export as one line of OTLP JSON to an opened [`OutputStream`].
pub(crate) struct JsonLinesWriter {
    /// The name of the output stream, for error messages and `Debug` output.
    name: String,
    /// `None` once the exporter has been shut down.
    state: Mutex<Option<State>>,
}

struct State {
    sink: Sink,
    /// The rest of a line that a failed write left unfinished. It is written
    /// before anything else, so a line is never followed by another one
    /// before it is complete.
    unfinished: Vec<u8>,
}

/// An opened [`OutputStream`].
enum Sink {
    /// Locked for each whole line, including a retried remainder, so that a
    /// line cannot interleave with `println!` output from other threads.
    Stdout,
    /// A regular file, which `force_flush` and `shutdown` sync to disk.
    File(File),
    Writer(Box<dyn Write + Send>),
}

impl Sink {
    /// Calls `f` with the writer, holding the lock of stdout during the call
    /// when writing there.
    fn with_writer<T>(&mut self, f: impl FnOnce(&mut dyn Write) -> T) -> T {
        match self {
            Sink::Stdout => f(&mut io::stdout().lock()),
            Sink::File(file) => f(file),
            Sink::Writer(writer) => f(writer),
        }
    }
}

impl JsonLinesWriter {
    fn new(name: String, sink: Sink) -> Self {
        JsonLinesWriter {
            name,
            state: Mutex::new(Some(State {
                sink,
                unfinished: Vec::new(),
            })),
        }
    }

    pub(crate) fn stdout() -> Self {
        JsonLinesWriter::new(OutputStream::Stdout.to_string(), Sink::Stdout)
    }

    /// Serializes `data` as compact JSON, appends a newline and flushes.
    ///
    /// The line is serialized up front and written while holding the lock of
    /// the writer, and of stdout when writing there, so it cannot interleave
    /// with other exports or with `println!` output from other threads.
    ///
    /// If a write fails after part of the line has been written, the rest of
    /// the line is kept and written before the next line, so a transient
    /// error never leaves a truncated line followed by another one. A line
    /// of which nothing was written is dropped.
    pub(crate) fn write<T: Serialize>(&self, data: &T) -> OTelSdkResult {
        let mut line = serde_json::to_vec(data).map_err(|err| {
            OTelSdkError::InternalFailure(format!("failed to serialize OTLP JSON: {err}"))
        })?;
        line.push(b'\n');
        self.with_state(|state| state.write(&line))
    }

    /// Returns an error if the exporter has been shut down.
    pub(crate) fn ensure_open(&self) -> OTelSdkResult {
        self.with_state(|_| Ok(()))
    }

    /// Completes an unfinished line, if any, and flushes, then syncs a regular
    /// file to disk.
    // `LogExporter` has no `force_flush`.
    #[cfg(any(feature = "trace", feature = "metrics"))]
    pub(crate) fn flush(&self) -> OTelSdkResult {
        self.with_state(State::flush)
    }

    /// Flushes like `flush`, then drops the writer, which closes a file
    /// opened by the exporter. The writer is dropped even if the flush fails.
    pub(crate) fn shutdown(&self) -> OTelSdkResult {
        let mut state = self.lock()?.take().ok_or(OTelSdkError::AlreadyShutdown)?;
        state.flush().map_err(|err| self.io_error(err))
    }

    fn with_state(&self, f: impl FnOnce(&mut State) -> io::Result<()>) -> OTelSdkResult {
        let mut guard = self.lock()?;
        let state = guard.as_mut().ok_or(OTelSdkError::AlreadyShutdown)?;
        f(state).map_err(|err| self.io_error(err))
    }

    fn lock(&self) -> Result<MutexGuard<'_, Option<State>>, OTelSdkError> {
        self.state.lock().map_err(|_| {
            OTelSdkError::InternalFailure(format!("a previous write to {} panicked", self.name))
        })
    }

    fn io_error(&self, err: io::Error) -> OTelSdkError {
        OTelSdkError::InternalFailure(format!("failed to write OTLP JSON to {}: {err}", self.name))
    }
}

impl State {
    /// Writes the unfinished line, if any, then `bytes`, then flushes.
    fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        let State { sink, unfinished } = self;
        sink.with_writer(|writer| {
            let (written, result) = write_fully(writer, unfinished);
            unfinished.drain(..written);
            result?;
            let (written, result) = write_fully(writer, bytes);
            if result.is_err() && written > 0 {
                // Keep the rest of the line so that the next write completes
                // it before starting a new one.
                unfinished.extend_from_slice(&bytes[written..]);
            }
            result?;
            writer.flush()
        })
    }

    /// Writes the unfinished line, if any, then flushes and syncs a regular
    /// file to disk.
    fn flush(&mut self) -> io::Result<()> {
        self.write(&[])?;
        match &self.sink {
            Sink::File(file) => file.sync_all(),
            Sink::Stdout | Sink::Writer(_) => Ok(()),
        }
    }
}

/// Writes all of `buf` like [`Write::write_all`], but also returns how many
/// bytes were written when it fails.
fn write_fully(writer: &mut dyn Write, buf: &[u8]) -> (usize, io::Result<()>) {
    let mut written = 0;
    while written < buf.len() {
        match writer.write(&buf[written..]) {
            Ok(0) => {
                let err = io::Error::new(io::ErrorKind::WriteZero, "failed to write whole buffer");
                return (written, Err(err));
            }
            Ok(n) => written += n,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => {}
            Err(err) => return (written, Err(err)),
        }
    }
    (written, Ok(()))
}

impl fmt::Debug for JsonLinesWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JsonLinesWriter")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::io::ErrorKind;
    use std::path::Path;
    use std::sync::Arc;

    /// Returns every line of the file at `path`, checking that each one is complete.
    pub(crate) fn read_lines(path: &Path) -> Vec<String> {
        complete_lines(&fs::read_to_string(path).unwrap())
    }

    fn complete_lines(contents: &str) -> Vec<String> {
        assert!(
            contents.is_empty() || contents.ends_with('\n'),
            "the last line is not terminated: {contents:?}"
        );
        contents.lines().map(ToOwned::to_owned).collect()
    }

    /// A cloneable in-memory writer for inspecting what an exporter wrote.
    #[derive(Clone, Debug, Default)]
    pub(crate) struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

    impl SharedBuffer {
        pub(crate) fn contents(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
        }

        /// Returns every line written so far, checking that each one is complete.
        pub(crate) fn lines(&self) -> Vec<String> {
            complete_lines(&self.contents())
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

    fn open(writer: impl Write + Send + 'static) -> JsonLinesWriter {
        OutputStream::Writer(Box::new(writer)).open().unwrap()
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

    /// Accepts at most a few bytes per call, so that writing a line takes many calls.
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

    /// Accepts bytes until its budget runs out, then fails every write until
    /// the budget is raised again. `None` means no limit.
    #[derive(Clone, Default)]
    struct LimitedWriter {
        buffer: SharedBuffer,
        budget: Arc<Mutex<Option<usize>>>,
    }

    impl LimitedWriter {
        fn set_budget(&self, budget: Option<usize>) {
            *self.budget.lock().unwrap() = budget;
        }
    }

    impl Write for LimitedWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let mut budget = self.budget.lock().unwrap();
            let len = match *budget {
                Some(0) => return Err(io::Error::other("disk full")),
                Some(remaining) => {
                    let len = buf.len().min(remaining);
                    *budget = Some(remaining - len);
                    len
                }
                None => buf.len(),
            };
            self.buffer.write(&buf[..len])
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn reports_write_errors() {
        let writer = LimitedWriter::default();
        writer.set_budget(Some(0));
        let output = open(writer);

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
    fn completes_a_partly_written_line_before_the_next_one() {
        let writer = LimitedWriter::default();
        let output = open(writer.clone());
        writer.set_budget(Some(5));

        assert!(output.write(&json!({"n": 1, "text": "first"})).is_err());
        assert_eq!(writer.buffer.contents(), "{\"n\":");

        writer.set_budget(None);
        output.write(&json!({"n": 2})).unwrap();

        assert_eq!(
            writer.buffer.lines(),
            vec!["{\"n\":1,\"text\":\"first\"}", "{\"n\":2}"]
        );
    }

    #[test]
    fn does_not_start_a_line_while_an_earlier_one_is_unfinished() {
        let writer = LimitedWriter::default();
        let output = open(writer.clone());
        writer.set_budget(Some(3));

        assert!(output.write(&json!({"n": 1})).is_err());
        assert!(output.write(&json!({"n": 2})).is_err());
        assert_eq!(writer.buffer.contents(), "{\"n");

        // Once the writer recovers, the unfinished line is completed first.
        // The line of which nothing was written is not retried.
        writer.set_budget(None);
        output.write(&json!({"n": 3})).unwrap();

        assert_eq!(writer.buffer.lines(), vec!["{\"n\":1}", "{\"n\":3}"]);
    }

    #[test]
    fn drops_a_line_of_which_nothing_was_written() {
        let writer = LimitedWriter::default();
        let output = open(writer.clone());
        writer.set_budget(Some("{\"n\":1}\n".len()));

        output.write(&json!({"n": 1})).unwrap();
        assert!(output.write(&json!({"n": 2})).is_err());

        writer.set_budget(None);
        output.write(&json!({"n": 3})).unwrap();

        assert_eq!(writer.buffer.lines(), vec!["{\"n\":1}", "{\"n\":3}"]);
    }

    #[cfg(any(feature = "trace", feature = "metrics"))]
    #[test]
    fn flush_completes_an_unfinished_line() {
        let writer = LimitedWriter::default();
        let output = open(writer.clone());
        writer.set_budget(Some(3));
        assert!(output.write(&json!({"n": 1})).is_err());

        assert!(output.flush().is_err());

        writer.set_budget(None);
        output.flush().unwrap();
        assert_eq!(writer.buffer.lines(), vec!["{\"n\":1}"]);
    }

    #[test]
    fn shutdown_completes_an_unfinished_line() {
        let writer = LimitedWriter::default();
        let output = open(writer.clone());
        writer.set_budget(Some(3));
        assert!(output.write(&json!({"n": 1})).is_err());

        writer.set_budget(None);
        output.shutdown().unwrap();
        assert_eq!(writer.buffer.lines(), vec!["{\"n\":1}"]);
    }

    #[test]
    fn shutdown_drops_the_writer_even_if_the_line_cannot_be_completed() {
        let writer = LimitedWriter::default();
        let output = open(writer.clone());
        writer.set_budget(Some(3));
        assert!(output.write(&json!({"n": 1})).is_err());

        assert!(matches!(
            output.shutdown(),
            Err(OTelSdkError::InternalFailure(_))
        ));
        assert!(matches!(
            output.write(&json!({})),
            Err(OTelSdkError::AlreadyShutdown)
        ));
    }

    /// Fails the first write with `Interrupted`, then behaves like `inner`.
    struct InterruptedOnce<W> {
        interrupted: bool,
        inner: W,
    }

    impl<W: Write> Write for InterruptedOnce<W> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(ErrorKind::Interrupted.into());
            }
            self.inner.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.inner.flush()
        }
    }

    #[test]
    fn retries_interrupted_writes() {
        let buffer = SharedBuffer::default();
        let output = open(InterruptedOnce {
            interrupted: false,
            inner: buffer.clone(),
        });

        output.write(&json!({"n": 1})).unwrap();

        assert_eq!(buffer.lines(), vec!["{\"n\":1}"]);
    }

    struct ZeroWriter;

    impl Write for ZeroWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Ok(0)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn reports_a_writer_that_accepts_no_bytes() {
        let output = open(ZeroWriter);

        match output.write(&json!({})) {
            Err(OTelSdkError::InternalFailure(message)) => assert_eq!(
                message,
                "failed to write OTLP JSON to the configured writer: failed to write whole buffer"
            ),
            other => panic!("unexpected result: {other:?}"),
        }
    }

    #[test]
    fn creates_a_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("traces.jsonl");
        let output = OutputStream::File(path.clone()).open().unwrap();

        output.write(&json!({"n": 1})).unwrap();
        output.shutdown().unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "{\"n\":1}\n");
    }

    #[test]
    fn appends_to_an_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("traces.jsonl");
        fs::write(&path, "{\"n\":1}\n").unwrap();

        let output = OutputStream::File(path.clone()).open().unwrap();
        output.write(&json!({"n": 2})).unwrap();
        output.shutdown().unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "{\"n\":1}\n{\"n\":2}\n");
    }

    #[test]
    fn fails_to_open_a_file_in_a_missing_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing").join("traces.jsonl");

        match OutputStream::File(path.clone()).open() {
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

    #[cfg(any(feature = "trace", feature = "metrics"))]
    #[test]
    fn flush_keeps_a_file_open() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("traces.jsonl");
        let output = OutputStream::File(path.clone()).open().unwrap();

        output.write(&json!({"n": 1})).unwrap();
        output.flush().unwrap();
        output.write(&json!({"n": 2})).unwrap();
        output.shutdown().unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "{\"n\":1}\n{\"n\":2}\n");
    }

    #[test]
    fn syncs_a_regular_file() {
        let dir = tempfile::tempdir().unwrap();
        let output = OutputStream::File(dir.path().join("traces.jsonl"))
            .open()
            .unwrap();

        let state = output.state.lock().unwrap();
        assert!(matches!(state.as_ref().unwrap().sink, Sink::File(_)));
    }

    /// Syncing a device fails, with `EINVAL` on Linux, so a device is written
    /// to without being synced.
    #[test]
    fn does_not_sync_a_device() {
        let device = if cfg!(windows) { "NUL" } else { "/dev/null" };
        let output = OutputStream::File(PathBuf::from(device)).open().unwrap();

        output.write(&json!({"n": 1})).unwrap();
        #[cfg(any(feature = "trace", feature = "metrics"))]
        output.flush().unwrap();
        output.shutdown().unwrap();
    }

    #[test]
    fn debug_output_names_the_output_stream() {
        assert_eq!(
            format!("{:?}", JsonLinesWriter::stdout()),
            "JsonLinesWriter { name: \"stdout\", .. }"
        );
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("traces.jsonl");
        assert_eq!(
            format!("{:?}", OutputStream::File(path.clone()).open().unwrap()),
            format!(
                "JsonLinesWriter {{ name: {:?}, .. }}",
                path.display().to_string()
            )
        );
        assert_eq!(
            format!("{:?}", OutputStream::File(PathBuf::from("traces.jsonl"))),
            "File(\"traces.jsonl\")"
        );
        assert_eq!(
            format!("{:?}", OutputStream::Writer(Box::new(io::sink()))),
            "Writer"
        );
    }
}
