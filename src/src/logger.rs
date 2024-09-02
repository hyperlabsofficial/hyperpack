use chrono::{DateTime, Local};
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::path::Path;
use std::thread;
use std::sync::mpsc;

/// A custom logger that writes log messages to a file with asynchronous capabilities.
struct FileLogger {
    file: Arc<Mutex<File>>,
    level: LevelFilter,
    tx: Option<mpsc::Sender<String>>, // Optional sender for async logging
}

impl FileLogger {
    /// Creates a new `FileLogger` instance.
    ///
    /// # Arguments
    ///
    /// * `file_path` - A string slice that holds the path to the log file.
    /// * `level` - The logging level filter.
    /// * `async_mode` - Boolean to enable asynchronous logging.
    ///
    /// # Returns
    ///
    /// * `Result<FileLogger, std::io::Error>` - Returns a `FileLogger` on success or an error on failure.
    pub fn new(file_path: &str, level: LevelFilter, async_mode: bool) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new().append(true).create(true).open(file_path)?;
        let (tx, rx) = mpsc::channel();
        
        if async_mode {
            let file = Arc::new(Mutex::new(file));
            let file_clone = file.clone();
            thread::spawn(move || {
                while let Ok(message) = rx.recv() {
                    let mut file = file_clone.lock().unwrap();
                    writeln!(file, "{}", message).unwrap();
                }
            });
            Ok(Self {
                file: file_clone,
                level,
                tx: Some(tx),
            })
        } else {
            Ok(Self {
                file: Arc::new(Mutex::new(file)),
                level,
                tx: None,
            })
        }
    }

    /// Writes a log message to the file, or sends it to the async channel.
    ///
    /// # Arguments
    ///
    /// * `message` - A string slice that holds the log message.
    fn write_log(&self, message: &str) {
        if let Some(tx) = &self.tx {
            tx.send(message.to_string()).unwrap();
        } else {
            let mut file = self.file.lock().unwrap();
            writeln!(file, "{}", message).unwrap();
        }
    }
}

impl log::Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let now: DateTime<Local> = Local::now();
            let message = format!(
                "{} [{}] {} - {}",
                now.format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                record.args()
            );
            self.write_log(&message);
        }
    }

    fn flush(&self) {}
}

/// Initializes the custom logger with optional asynchronous logging.
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the path to the log file.
/// * `level` - The logging level filter.
/// * `async_mode` - Boolean to enable asynchronous logging.
///
/// # Returns
///
/// * `Result<(), SetLoggerError>` - Returns `Ok(())` on success or an error on failure.
pub fn init_logger(file_path: &str, level: LevelFilter, async_mode: bool) -> Result<(), SetLoggerError> {
    let logger = FileLogger::new(file_path, level, async_mode)?;
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(level);
    Ok(())
}

/// A utility function to test the logger.
fn test_logger() {
    log::info!("This is an info message from the test function.");
    log::warn!("This is a warning message from the test function.");
    log::error!("This is an error message from the test function.");
}

fn main() {
    // Define log levels and file paths for different environments
    let log_level = LevelFilter::Debug;
    let log_file_path = "my_website.log";

    // Initialize logger with asynchronous logging
    if let Err(e) = init_logger(log_file_path, log_level, true) {
        eprintln!("Failed to initialize logger: {}", e);
        return;
    }

    // Log some messages
    log::info!("Logger initialized with level: {:?}", log_level);
    log::debug!("This is a debug message.");
    log::trace!("This is a trace message.");

    // Test logging
    test_logger();

    // Demonstrate logging with custom target
    log::info!(target: "custom_target", "Logging with a custom target.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_initialization() {
        // Initialize logger for testing
        let file_path = "test.log";
        let _ = init_logger(file_path, LevelFilter::Info, false).expect("Failed to initialize test logger");

        // Test logging
        log::info!("Test info message.");
        log::warn!("Test warning message.");
        log::error!("Test error message.");

        // Check if file was created and contains logs
        let path = Path::new(file_path);
        assert!(path.exists(), "Log file should be created");

        // Clean up
        std::fs::remove_file(file_path).expect("Failed to remove test log file");
    }

    #[test]
    fn test_log_message_format() {
        let file_path = "format_test.log";
        let _ = init_logger(file_path, LevelFilter::Debug, false).expect("Failed to initialize test logger");

        log::info!("Testing message formatting.");

        let content = std::fs::read_to_string(file_path).expect("Failed to read log file");
        assert!(content.contains("Testing message formatting."), "Log message not found in file");

        // Clean up
        std::fs::remove_file(file_path).expect("Failed to remove test log file");
    }

    #[test]
    fn test_async_logging() {
        let file_path = "async_test.log";
        let _ = init_logger(file_path, LevelFilter::Debug, true).expect("Failed to initialize async logger");

        log::info!("Testing async logging.");

        thread::sleep(std::time::Duration::from_secs(1)); // Ensure async log is written

        let content = std::fs::read_to_string(file_path).expect("Failed to read log file");
        assert!(content.contains("Testing async logging."), "Log message not found in file");

        // Clean up
        std::fs::remove_file(file_path).expect("Failed to remove test log file");
    }
}