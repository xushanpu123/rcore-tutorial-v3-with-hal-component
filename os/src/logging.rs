use core::fmt::{self, Write};

use polyhal::debug::DebugConsole;
use log::{self, info, Level, LevelFilter, Log, Metadata, Record};

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let file = record.file();
        let line = record.line();

        let color_code = match record.level() {
            Level::Error => 31u8, // Red
            Level::Warn => 93,    // BrightYellow
            Level::Info => 34,    // Blue
            Level::Debug => 32,   // Green
            Level::Trace => 90,   // BrightBlack
        };
        write!(
            Logger,
            "\u{1B}[{}m\
            [{}] {}:{} {}\
            \u{1B}[0m\n",
            color_code,
            record.level(),
            file.unwrap(),
            line.unwrap(),
            record.args()
        )
        .expect("can't write color string in logging module.");
    }

    fn flush(&self) {}
}

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut buffer = [0u8; 4];
        for c in s.chars() {
            puts(c.encode_utf8(&mut buffer).as_bytes())
        }
        Ok(())
    }
}

pub fn init(level: Option<&str>) {
    log::set_logger(&Logger).unwrap();
    log::set_max_level(match level {
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        _ => LevelFilter::Off,
    });
    info!("logging module initialized");
}

#[inline]
pub fn puts(buffer: &[u8]) {
    // use the main uart if it exists.
    for i in buffer {
        DebugConsole::putchar(*i);
    }
}
