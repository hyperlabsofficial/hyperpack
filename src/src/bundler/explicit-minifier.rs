use clap::{Arg, Command};
use minify::js::Minifier;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use log::{info, warn, error, debug, LevelFilter};
use simple_logger::SimpleLogger;
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::time::Instant;

struct MinificationContext {
    input_path: PathBuf,
    output_path: PathBuf,
    use_parallel_processing: bool,
    keep_comments: bool,
    cache: HashMap<String, String>,
    dry_run: bool,
}

impl MinificationContext {
    fn new(input_path: PathBuf, output_path: PathBuf, use_parallel_processing: bool, keep_comments: bool, dry_run: bool) -> Self {
        Self {
            input_path,
            output_path,
            use_parallel_processing,
            keep_comments,
            cache: HashMap::new(),
            dry_run,
        }
    }
}

fn minify_code(code: &str, keep_comments: bool) -> Result<String, String> {
    let mut minifier = Minifier::new();
    if keep_comments {
        minifier.keep_comments();
    }
    minifier.minify(code).map_err(|err| format!("Minification failed: {}", err))
}

fn process_file(ctx: &mut MinificationContext, input_path: &Path, output_path: &Path) -> Result<(), String> {
    if ctx.cache.contains_key(input_path.to_str().unwrap()) {
        debug!("Cache hit for file: {:?}", input_path);
        if !ctx.dry_run {
            fs::write(output_path, ctx.cache.get(input_path.to_str().unwrap()).unwrap())
                .map_err(|err| format!("Failed to write output file: {}", err))?;
        }
        return Ok(());
    }

    let code = fs::read_to_string(input_path)
        .map_err(|err| format!("Failed to read input file: {}", err))?;
    
    let minified_code = minify_code(&code, ctx.keep_comments)?;
    if !ctx.dry_run {
        fs::write(output_path, &minified_code)
            .map_err(|err| format!("Failed to write output file: {}", err))?;
    }

    ctx.cache.insert(input_path.to_str().unwrap().to_string(), minified_code);
    Ok(())
}

fn process_directory(ctx: &mut MinificationContext, input_dir: &Path, output_dir: &Path) -> Result<(), String> {
    if !output_dir.exists() {
        fs::create_dir_all(output_dir).map_err(|err| format!("Failed to create output directory: {}", err))?;
    }

    let files: Vec<_> = fs::read_dir(input_dir)
        .map_err(|err| format!("Failed to read input directory: {}", err))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .collect();

    let progress_bar = ProgressBar::new(files.len() as u64);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .progress_chars("#>-"));

    if ctx.use_parallel_processing {
        files.par_iter().try_for_each(|entry| {
            let input_path = entry.path();
            let output_path = output_dir.join(input_path.file_name().unwrap());
            match process_file(ctx, &input_path, &output_path) {
                Ok(_) => {
                    progress_bar.inc(1);
                    Ok(())
                },
                Err(err) => {
                    error!("Error processing file {:?}: {}", input_path, err);
                    Err(())
                }
            }
        })?;
    } else {
        for entry in files {
            let input_path = entry.path();
            let output_path = output_dir.join(input_path.file_name().unwrap());
            if let Err(err) = process_file(ctx, &input_path, &output_path) {
                error!("Error processing file {:?}: {}", input_path, err);
            }
            progress_bar.inc(1);
        }
    }

    progress_bar.finish_with_message("Processing complete");
    Ok(())
}

fn main() {
    SimpleLogger::new().with_level(LevelFilter::Info).init().expect("Failed to initialize logger");

    let matches = Command::new("minify")
        .arg(
            Arg::new("input")
                .about("Input file or directory to minify")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("output")
                .about("Output file or directory for minified code")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::new("parallel")
                .short('p')
                .long("parallel")
                .about("Use parallel processing"),
        )
        .arg(
            Arg::new("keep-comments")
                .short('k')
                .long("keep-comments")
                .about("Keep comments in the minified output"),
        )
        .arg(
            Arg::new("dry-run")
                .short('d')
                .long("dry-run")
                .about("Perform a dry run without writing output files"),
        )
        .arg(
            Arg::new("log-level")
                .short('l')
                .long("log-level")
                .about("Set the log level (off, error, warn, info, debug, trace)")
                .takes_value(true),
        )
        .get_matches();

    let input_path = PathBuf::from(matches.value_of("input").unwrap());
    let output_path = PathBuf::from(matches.value_of("output").unwrap());
    let use_parallel_processing = matches.is_present("parallel");
    let keep_comments = matches.is_present("keep-comments");
    let dry_run = matches.is_present("dry-run");

    if let Some(log_level) = matches.value_of("log-level") {
        match log_level.to_lowercase().as_str() {
            "off" => log::set_max_level(LevelFilter::Off),
            "error" => log::set_max_level(LevelFilter::Error),
            "warn" => log::set_max_level(LevelFilter::Warn),
            "info" => log::set_max_level(LevelFilter::Info),
            "debug" => log::set_max_level(LevelFilter::Debug),
            "trace" => log::set_max_level(LevelFilter::Trace),
            _ => warn!("Invalid log level specified, using default: info"),
        }
    }

    info!("Starting minification process");

    let mut ctx = MinificationContext::new(input_path.clone(), output_path.clone(), use_parallel_processing, keep_comments, dry_run);

    let start_time = Instant::now();

    if input_path.is_file() {
        if let Err(err) = process_file(&mut ctx, &input_path, &output_path) {
            error!("{}", err);
            std::process::exit(1);
        }
    } else if input_path.is_dir() {
        if let Err(err) = process_directory(&mut ctx, &input_path, &output_path) {
            error!("{}", err);
            std::process::exit(1);
        }
    } else {
        error!("Invalid input path specified");
        std::process::exit(1);
    }

    let elapsed_time = start_time.elapsed();
    info!("Minification completed successfully in {:.2?}", elapsed_time);
}