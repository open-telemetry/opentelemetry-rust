use std::{sync::{Arc, Mutex}, thread::sleep, time::SystemTime};

use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::logs::{
    LogRecord as _, LogResult, Logger as _, LoggerProvider as _, Severity,
};
use opentelemetry_sdk::{export::logs::LogData, logs::{LogProcessor, LoggerProvider}};

// Run this benchmark with:
// cargo bench --bench log_processor

#[derive(Debug)]
struct NoopProcessor;

impl LogProcessor for NoopProcessor {
    fn emit(&self, _data: &mut LogData) {}

    fn force_flush(&self) -> LogResult<()> {
        Ok(())
    }

    fn shutdown(&self) -> LogResult<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct CloningProcessor;

impl LogProcessor for CloningProcessor {
    fn emit(&self, data: &mut LogData) {
        let _data_cloned = data.clone();
    }

    fn force_flush(&self) -> LogResult<()> {
        Ok(())
    }

    fn shutdown(&self) -> LogResult<()> {
        Ok(())
    }

}


#[derive(Debug)]
struct SendToChannelProcessor {
    sender: std::sync::mpsc::Sender<LogData>,
    receiver: Arc<Mutex<std::sync::mpsc::Receiver<LogData>>>,
}

impl SendToChannelProcessor {
    fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let s = Self {             
             sender : sender,
             receiver : Arc::new(Mutex::new(receiver))
            };
        let receiver_cloned = s.receiver.clone();
        let _ = std::thread::spawn(move || {
            loop {
                sleep(std::time::Duration::from_millis(10));
                let data = receiver_cloned.lock().unwrap().recv();
                if data.is_err() {
                    println!("Error receiving log data from channel {0}", data.err().unwrap());
                    break;
                }
            }
        });
        s
    }
}

impl LogProcessor for SendToChannelProcessor {
    fn emit(&self, data: &mut LogData) {
        let data_cloned = data.clone();
        let res =  self.sender.send(data_cloned);
        if res.is_err() {
            println!("Error sending log data to channel {0}", res.err().unwrap());
        }
    }

    fn force_flush(&self) -> LogResult<()> {
        Ok(())
    }

    fn shutdown(&self) -> LogResult<()> {
        Ok(())
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    log_noop_processor(c);
    log_cloning_processor(c);
    log_cloning_and_send_to_channel_processor(c);
}

fn log_noop_processor(c: &mut Criterion) {
    let provider = LoggerProvider::builder()
        .with_log_processor(NoopProcessor {})
        .build();
    let logger = provider.logger("benchmark");

    c.bench_function("log_noop_processor", |b| {
        b.iter(|| {
            let mut log_record = logger.create_log_record();
            let now = SystemTime::now();
            log_record.set_observed_timestamp(now);
            log_record.set_target("my-target".to_string());
            log_record.set_event_name("CheckoutFailed");
            log_record.set_severity_number(Severity::Warn);
            log_record.set_severity_text("WARN");
            log_record.add_attribute("book_id", "12345");
            log_record.add_attribute("book_title", "Rust Programming Adventures");
            log_record.add_attribute("message", "Unable to process checkout.");

            logger.emit(log_record);
        });
    });
}

fn log_cloning_processor(c: &mut Criterion) {
    let provider = LoggerProvider::builder()
        .with_log_processor(CloningProcessor {})
        .build();
    let logger = provider.logger("benchmark");

    c.bench_function("log_cloning_processor", |b| {
        b.iter(|| {
            let mut log_record = logger.create_log_record();
            let now = SystemTime::now();
            log_record.set_observed_timestamp(now);
            log_record.set_target("my-target".to_string());
            log_record.set_event_name("CheckoutFailed");
            log_record.set_severity_number(Severity::Warn);
            log_record.set_severity_text("WARN");
            log_record.add_attribute("book_id", "12345");
            log_record.add_attribute("book_title", "Rust Programming Adventures");
            log_record.add_attribute("message", "Unable to process checkout.");

            logger.emit(log_record);
        });
    });
}

fn log_cloning_and_send_to_channel_processor(c: &mut Criterion) {
    let provider = LoggerProvider::builder()
        .with_log_processor(SendToChannelProcessor::new())
        .build();
    let logger = provider.logger("benchmark");

    c.bench_function("log_clone_and_send_to_channel_processor", |b| {
        b.iter(|| {
            let mut log_record = logger.create_log_record();
            let now = SystemTime::now();
            log_record.set_observed_timestamp(now);
            log_record.set_target("my-target".to_string());
            log_record.set_event_name("CheckoutFailed");
            log_record.set_severity_number(Severity::Warn);
            log_record.set_severity_text("WARN");
            log_record.add_attribute("book_id", "12345");
            log_record.add_attribute("book_title", "Rust Programming Adventures");
            log_record.add_attribute("message", "Unable to process checkout.");

            logger.emit(log_record);
        });
    });
}
criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
