use notify::{watcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::process::{Command, Stdio};
use std::fs::{self, OpenOptions, create_dir_all};
use std::env;
use std::path::{Path, PathBuf};
use std::io::{Write, Error as IoError};
use chrono::Local;
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

/// Retrieves the path to the log file from the environment variable `LOG_FILE_PATH`.
/// Defaults to `"file_watcher.log"` if the variable is not set.
///
/// # Returns
/// The path to the log file.
fn get_log_file_path() -> PathBuf {
    let log_path = env::var("LOG_FILE_PATH").unwrap_or_else(|_| "file_watcher.log".to_string());
    Path::new(&log_path).to_path_buf()
}

/// Sets up and returns a file handle for the log file in append mode.
/// Creates the file if it does not exist.
///
/// # Returns
/// A `std::fs::File` instance for appending log entries.
fn setup_log_file() -> fs::File {
    let log_file = get_log_file_path();
    if let Some(parent) = log_file.parent() {
        if !parent.exists() {
            create_dir_all(parent).expect("Failed to create log file directory");
        }
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .unwrap_or_else(|err| panic!("Failed to open or create log file: {}", err))
}

/// Logs file system events to the log file with a timestamp.
///
/// # Parameters
/// - `file`: The path of the file related to the event.
/// - `event`: A description of the event (e.g., "File created").
fn log_event(file: &str, event: &str) {
    let mut log = setup_log_file();
    let log_entry = format!(
        "{} - File: {}, Event: {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        file,
        event
    );
    if let Err(e) = writeln!(log, "{}", log_entry) {
        eprintln!("Failed to write to log file: {}", e);
    }
}

/// Executes a custom command based on the event type specified in the environment variables.
/// Logs the outcome of the command execution.
///
/// # Parameters
/// - `event_type`: The type of event (e.g., "Create", "Write").
fn execute_custom_command(event_type: &str) {
    let command_key = format!("{}_COMMAND", event_type.to_uppercase());
    let args_key = format!("{}_ARGS", event_type.to_uppercase());

    let command = env::var(&command_key).unwrap_or_else(|_| "".to_string());
    let args = env::var(&args_key).unwrap_or_else(|_| "".to_string());

    if command.is_empty() {
        return; // No command specified for this event type.
    }

    let status = Command::new(&command)
        .args(args.split_whitespace())
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap_or_else(|err| panic!("Failed to execute command: {}", err));

    if status.success() {
        println!("{} command succeeded", event_type);
        log_event("Command", &format!("{} command succeeded", event_type));
    } else {
        eprintln!("{} command failed with exit code: {:?}", event_type, status.code());
        log_event("Command", &format!("{} command failed with exit code: {:?}", event_type, status.code()));
    }
}

/// Handles different types of file system events and executes appropriate actions.
///
/// # Parameters
/// - `event`: The file system event to handle.
fn handle_event(event: notify::DebouncedEvent) {
    match event {
        notify::DebouncedEvent::Write(path) => {
            println!("File written: {:?}", path);
            log_event(&path.display().to_string(), "File written");
            execute_custom_command("WRITE");
        }
        notify::DebouncedEvent::Create(path) => {
            println!("File created: {:?}", path);
            log_event(&path.display().to_string(), "File created");
            execute_custom_command("CREATE");
        }
        notify::DebouncedEvent::Remove(path) => {
            println!("File removed: {:?}", path);
            log_event(&path.display().to_string(), "File removed");
            execute_custom_command("REMOVE");
        }
        notify::DebouncedEvent::Rename(src, dst) => {
            println!("File renamed from {:?} to {:?}", src, dst);
            log_event(&src.display().to_string(), "File renamed (source)");
            log_event(&dst.display().to_string(), "File renamed (destination)");
            execute_custom_command("RENAME");
        }
        _ => {}  // Ignore other event types.
    }
}

/// Sets up signal handling to gracefully shut down the application on SIGINT.
fn setup_signal_handling() -> Arc<Mutex<bool>> {
    let running = Arc::new(Mutex::new(true));
    let running_clone = running.clone();

    let signals = Signals::new(&[SIGINT]).expect("Failed to setup signal handling");

    thread::spawn(move || {
        for _ in signals.forever() {
            *running_clone.lock().unwrap() = false;
        }
    });

    running
}

fn main() {
    let (tx, rx) = channel();
    
    let directories_to_watch: Vec<PathBuf> = env::var("WATCH_DIRECTORIES")
        .unwrap_or_else(|_| ".".to_string())
        .split(',')
        .map(PathBuf::from)
        .collect();

    let exclude_directories: Vec<PathBuf> = env::var("EXCLUDE_DIRECTORIES")
        .unwrap_or_else(|_| "".to_string())
        .split(',')
        .map(PathBuf::from)
        .collect();

    let debounce_time = env::var("DEBOUNCE_TIME")
        .unwrap_or_else(|_| "2".to_string())
        .parse::<u64>()
        .unwrap_or(2);

    if env::var("CLEAR_LOG_ON_START").unwrap_or_else(|_| "false".to_string()) == "true" {
        clear_log_file();
    }

    let mut watcher = watcher(tx, Duration::from_secs(debounce_time))
        .unwrap_or_else(|err| panic!("Failed to create watcher: {}", err));

    for dir in &directories_to_watch {
        if dir.exists() && dir.is_dir() {
            let is_excluded = exclude_directories.iter().any(|excl| dir.starts_with(excl));
            if !is_excluded {
                watcher.watch(dir, RecursiveMode::Recursive)
                    .unwrap_or_else(|err| panic!("Failed to watch directory: {}", err));
                println!("Watching directory: {:?}", dir);
                log_event(&dir.display().to_string(), "Started watching");
            } else {
                println!("Directory is excluded: {:?}", dir);
                log_event(&dir.display().to_string(), "Excluded directory");
            }
        } else {
            eprintln!("Directory does not exist: {:?}", dir);
            log_event(&dir.display().to_string(), "Directory does not exist");
        }
    }

    println!("Watching for file changes...");

    let running = setup_signal_handling();

    loop {
        if !*running.lock().unwrap() {
            println!("Shutting down gracefully...");
            break;
        }

        match rx.recv() {
            Ok(event) => handle_event(event),
            Err(e) => {
                eprintln!("Watch error: {:?}", e);
                log_event("Watch", &format!("Error: {:?}", e));
            }
        }
    }
}