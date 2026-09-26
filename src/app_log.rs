//! Shared in-memory application log for the GUI log viewer.

use std::{
    collections::VecDeque,
    sync::{Mutex, OnceLock},
};

const MAX_ENTRIES: usize = 1_000;

static ENTRIES: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();
static LOGGER: AppLogger = AppLogger;

fn entries() -> &'static Mutex<VecDeque<String>> {
    ENTRIES.get_or_init(|| Mutex::new(VecDeque::with_capacity(MAX_ENTRIES)))
}

/// Return the currently retained log lines, oldest first.
pub fn recent_logs() -> Vec<String> {
    entries()
        .lock()
        .map(|logs| logs.iter().cloned().collect())
        .unwrap_or_default()
}

/// Remove all retained log lines.
pub fn clear_logs() {
    if let Ok(mut logs) = entries().lock() {
        logs.clear();
    }
}

/// Install the application logger. Calling this more than once is harmless.
pub fn init() {
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(log::LevelFilter::Info);
    }
}

struct AppLogger;

impl log::Log for AppLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let timestamp = time::OffsetDateTime::now_utc()
            .format(&time::macros::format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second]"
            ))
            .unwrap_or_else(|_| "time unavailable".to_owned());
        let line = format!(
            "[{timestamp}] [{:>5}] [{}] {}",
            record.level(),
            record.target(),
            record.args()
        );

        if let Ok(mut logs) = entries().lock() {
            if logs.len() == MAX_ENTRIES {
                logs.pop_front();
            }
            logs.push_back(line.clone());
        }

        #[cfg(debug_assertions)]
        println!("{line}");
    }

    fn flush(&self) {}
}
