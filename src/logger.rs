use log::{Metadata, Record, Log};

pub struct Logger;
impl Log for Logger {
    fn enabled(&self, _: &Metadata) -> bool { true }

    fn log(&self, record: &Record) {
        let level = record.level();
        use log::Level::*;
        let meth = match level {
            Error => "error",
            Warn => "warn",
            Info => "info",
            Debug => "debug",
            Trace => "debug",
        };
        let msg = format!("{} -- {}", record.target(), record.args());
        js! { console[@{meth}](@{msg}); }
    }

    fn flush(&self) {}
}
