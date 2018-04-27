use log::{Metadata, Record, Log};
use alert::{show_alert, AlertMode};

pub struct Logger;
impl Log for Logger {
    fn enabled(&self, _: &Metadata) -> bool { true }

    fn log(&self, record: &Record) {
        let level = record.level();
        let msg = format!("{}", record.args());

        use log::Level::*;
        let meth = match level {
            Error => {
                show_alert(&msg, AlertMode::Error);
                "error"
            },
            Warn => {
                show_alert(&msg, AlertMode::Warn);
                "warn"
            },
            Info => {
                show_alert(&msg, AlertMode::Info);
                "info"
            },
            Debug => "debug",
            Trace => "debug",
        };
        let msg = format!("{} -- {}", record.target(), record.args());
        js! { console[@{meth}](@{msg}); }
    }

    fn flush(&self) {}
}
