use regex::Regex;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

#[derive(Deserialize)]
struct Config {
    output_format: String,
    include_index: bool,
}

fn main() -> std::io::Result<()> {
    // Load configuration
    let config: Config = load_config("docgen.config.json")?;

    // Get command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: docgen <directory>");
        std::process::exit(1);
    }

    let dir = &args[1];
    let mut markdown = String::new();
    let mut html = String::new();
    let mut index = String::new();
    let mut file_count = 0;
    let mut error_count = 0;
    let mut file_paths = Vec::new();

    // Process the specified directory
    if let Err(e) = process_directory(dir, &mut markdown, &mut html, &mut index, &mut file_count, &mut error_count, &mut file_paths) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // Write the generated documentation to files
    if config.output_format == "markdown" || config.output_format == "both" {
        fs::write("DOCUMENTATION.md", markdown)?;
    }
    if config.output_format == "html" || config.output_format == "both" {
        fs::write("DOCUMENTATION.html", html)?;
    }
    if config.include_index {
        fs::write("INDEX.md", index)?;
    }

    println!("Processed {} files with {} errors.", file_count, error_count);

    Ok(())
}

/// Loads the configuration from a JSON file.
/// 
/// # Arguments
/// * `filename` - The name of the configuration file.
/// 
/// # Returns
/// * `Result<Config, serde_json::Error>` - The loaded configuration.
fn load_config(filename: &str) -> Result<Config, serde_json::Error> {
    let config_str = fs::read_to_string(filename)?;
    serde_json::from_str(&config_str)
}

/// Recursively processes files in the specified directory.
/// 
/// # Arguments
/// * `dir` - The directory to process.
/// * `markdown` - A mutable reference to a string to accumulate Markdown content.
/// * `html` - A mutable reference to a string to accumulate HTML content.
/// * `index` - A mutable reference to a string to accumulate file index.
/// * `file_count` - A mutable reference to count processed files.
/// * `error_count` - A mutable reference to count errors encountered.
/// * `file_paths` - A mutable reference to store paths of processed files.
/// 
/// # Returns
/// * `Ok(())` if successful.
/// * `Err(e)` if an error occurs.
fn process_directory(
    dir: &str,
    markdown: &mut String,
    html: &mut String,
    index: &mut String,
    file_count: &mut u32,
    error_count: &mut u32,
    file_paths: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    // Iterate over entries in the directory
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively process subdirectories
            process_directory(&path.to_string_lossy(), markdown, html, index, file_count, error_count, file_paths)?;
        } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if ext == "js" || ext == "ts" {
                // Process JavaScript and TypeScript files
                match process_file(&path) {
                    Ok((file_markdown, file_html)) => {
                        // Add file documentation to Markdown and HTML
                        markdown.push_str(&format!("# {}\n\n", path.display()));
                        markdown.push_str(&file_markdown);
                        
                        html.push_str(&format!("<h1>{}</h1>\n\n", path.display()));
                        html.push_str(&file_html);

                        if !file_paths.contains(&path) {
                            file_paths.push(path.clone());
                        }
                        
                        *file_count += 1;
                    },
                    Err(e) => {
                        // Log errors and increment error count
                        eprintln!("Failed to process file {}: {}", path.display(), e);
                        *error_count += 1;
                    }
                }
            }
        }
    }

    // Generate file index
    if !file_paths.is_empty() {
        index.push_str("# File Index\n\n");
        for path in file_paths {
            index.push_str(&format!("- [{}]({})\n", path.display(), path.display().to_string()));
        }
    }

    Ok(())
}

/// Processes a JavaScript or TypeScript file to extract comments and generate documentation.
/// 
/// # Arguments
/// * `path` - The path to the JavaScript or TypeScript file.
/// 
/// # Returns
/// * `Ok((String, String))` - Markdown and HTML content of the file documentation.
/// * `Err(e)` - If an error occurs while reading the file.
fn process_file(path: &PathBuf) -> std::io::Result<(String, String)> {
    let content = fs::read_to_string(path)?;
    let comments = extract_jsdoc_comments(&content);

    // Generate Markdown and HTML from comments
    let file_markdown = comments.iter().map(|comment| format!("{}\n\n", comment)).collect::<String>();
    let file_html = comments.iter().map(|comment| format!("<p>{}</p>\n\n", comment)).collect::<String>();

    Ok((file_markdown, file_html))
}

/// Extracts JSDoc comments from the provided content, including TypeScript syntax.
/// 
/// # Arguments
/// * `content` - The content of a JavaScript or TypeScript file.
/// 
/// # Returns
/// * `Vec<String>` - A vector of extracted JSDoc comments.
fn extract_jsdoc_comments(content: &str) -> Vec<String> {
    let mut comments = Vec::new();
    let mut in_comment = false;
    let mut current_comment = String::new();
    let re = Regex::new(r"/\*\*.*?\*/|//.*").unwrap();

    // Iterate over lines in the file content
    for line in content.lines() {
        if re.is_match(line) {
            if line.trim().starts_with("/**") {
                // Start of a JSDoc comment
                in_comment = true;
                current_comment.clear();
                current_comment.push_str(&line.trim_start_matches("/**").trim().to_string());
            } else if line.trim().starts_with("*/") {
                // End of a JSDoc comment
                if in_comment {
                    in_comment = false;
                    comments.push(format_comment(&current_comment));
                }
            } else if in_comment {
                // Continuation of a JSDoc comment
                current_comment.push_str(&format!("{}\n", line.trim()));
            }
        }
    }

    if in_comment {
        // Capture any unclosed comment at the end of the file
        comments.push(format_comment(&current_comment));
    }

    comments
}

/// Formats a raw JSDoc comment by removing leading asterisks and trimming whitespace.
/// 
/// # Arguments
/// * `comment` - The raw JSDoc comment.
/// 
/// # Returns
/// * `String` - The formatted comment.
fn format_comment(comment: &str) -> String {
    let re = Regex::new(r"^\s*\*\s?").unwrap();
    let formatted = re.replace_all(comment, "");
    formatted.trim().to_string()
}