use regex::Regex;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use glob::glob;

/// Recursively bundles JavaScript files.
/// 
/// # Arguments
///
/// * `path` - The path to the JavaScript file to bundle.
/// * `seen_files` - A set of already processed files to avoid infinite loops due to circular dependencies.
///
/// # Returns
///
/// * A `Result` containing the bundled code or an I/O error.
fn bundle_js_file(path: &Path, seen_files: &mut HashSet<PathBuf>) -> io::Result<String> {
    // Check if this file has already been processed to avoid reprocessing and infinite loops
    if seen_files.contains(path) {
        return Ok(String::new());
    }
    seen_files.insert(path.to_path_buf());

    // Read the content of the JavaScript file
    let code = fs::read_to_string(path)?;
    let mut bundled_code = String::new();
    
    // Regex to match import statements in the JavaScript code
    let re = Regex::new(r#"import\s+["']([^"']+)["'];"#).unwrap();

    // Modify the code to include the content of imported files
    let mut modified_code = code.clone();
    for cap in re.captures_iter(&code) {
        // Extract the import path from the import statement
        let import_path = cap.get(1).unwrap().as_str();
        // Construct the full path to the imported file
        let import_full_path = path.parent().unwrap().join(import_path);

        if import_full_path.exists() {
            // Recursively bundle the imported file
            let import_code = bundle_js_file(&import_full_path, seen_files)?;
            // Replace the import statement with the content of the imported file
            modified_code = modified_code.replace(&cap[0], &import_code);
        }
    }

    // Append the processed code to the bundled code
    bundled_code.push_str(&modified_code);
    bundled_code.push_str("\n");

    Ok(bundled_code)
}

/// Recursively bundles CSS files.
/// 
/// # Arguments
///
/// * `path` - The path to the CSS file to bundle.
/// * `seen_files` - A set of already processed files to avoid infinite loops due to circular dependencies.
///
/// # Returns
///
/// * A `Result` containing the bundled code or an I/O error.
fn bundle_css_file(path: &Path, seen_files: &mut HashSet<PathBuf>) -> io::Result<String> {
    // Check if this file has already been processed to avoid reprocessing and infinite loops
    if seen_files.contains(path) {
        return Ok(String::new());
    }
    seen_files.insert(path.to_path_buf());

    // Read the content of the CSS file
    let code = fs::read_to_string(path)?;
    let mut bundled_code = String::new();
    
    // Regex to match @import statements in the CSS code
    let re = Regex::new(r#"@import\s+["']([^"']+)["'];"#).unwrap();

    // Modify the code to include the content of imported files
    let mut modified_code = code.clone();
    for cap in re.captures_iter(&code) {
        // Extract the import path from the @import statement
        let import_path = cap.get(1).unwrap().as_str();
        // Construct the full path to the imported file
        let import_full_path = path.parent().unwrap().join(import_path);

        if import_full_path.exists() {
            // Recursively bundle the imported file
            let import_code = bundle_css_file(&import_full_path, seen_files)?;
            // Replace the @import statement with the content of the imported file
            modified_code = modified_code.replace(&cap[0], &import_code);
        }
    }

    // Append the processed code to the bundled code
    bundled_code.push_str(&modified_code);
    bundled_code.push_str("\n");

    Ok(bundled_code)
}

/// Recursively bundles HTML files.
/// 
/// # Arguments
///
/// * `path` - The path to the HTML file to bundle.
/// * `seen_files` - A set of already processed files to avoid infinite loops due to circular dependencies.
///
/// # Returns
///
/// * A `Result` containing the bundled code or an I/O error.
fn bundle_html_file(path: &Path, seen_files: &mut HashSet<PathBuf>) -> io::Result<String> {
    // Check if this file has already been processed to avoid reprocessing and infinite loops
    if seen_files.contains(path) {
        return Ok(String::new());
    }
    seen_files.insert(path.to_path_buf());

    // Read the content of the HTML file
    let code = fs::read_to_string(path)?;
    let mut bundled_code = String::new();
    
    // Regex to match <link rel="import" href="..."> statements in the HTML code
    let re = Regex::new(r#"<link\s+rel=["']import["']\s+href=["']([^"']+)["'];"#).unwrap();

    // Modify the code to include the content of imported files
    let mut modified_code = code.clone();
    for cap in re.captures_iter(&code) {
        // Extract the import path from the <link> statement
        let import_path = cap.get(1).unwrap().as_str();
        // Construct the full path to the imported file
        let import_full_path = path.parent().unwrap().join(import_path);

        if import_full_path.exists() {
            // Recursively bundle the imported file
            let import_code = bundle_html_file(&import_full_path, seen_files)?;
            // Replace the <link> statement with the content of the imported file
            modified_code = modified_code.replace(&cap[0], &import_code);
        }
    }

    // Append the processed code to the bundled code
    bundled_code.push_str(&modified_code);
    bundled_code.push_str("\n");

    Ok(bundled_code)
}

/// Recursively bundles JSON files.
/// 
/// # Arguments
///
/// * `path` - The path to the JSON file to bundle.
/// * `seen_files` - A set of already processed files to avoid infinite loops due to circular dependencies.
///
/// # Returns
///
/// * A `Result` containing the bundled code or an I/O error.
fn bundle_json_file(path: &Path, seen_files: &mut HashSet<PathBuf>) -> io::Result<String> {
    // Check if this file has already been processed to avoid reprocessing and infinite loops
    if seen_files.contains(path) {
        return Ok(String::new());
    }
    seen_files.insert(path.to_path_buf());

    // Read the content of the JSON file
    let code = fs::read_to_string(path)?;
    let mut bundled_code = String::new();
    
    // Regex to match "$import": "..." statements in the JSON code
    let re = Regex::new(r#""\$import":\s*["']([^"']+)["']"#).unwrap();

    // Modify the code to include the content of imported files
    let mut modified_code = code.clone();
    for cap in re.captures_iter(&code) {
        // Extract the import path from the $import statement
        let import_path = cap.get(1).unwrap().as_str();
        // Construct the full path to the imported file
        let import_full_path = path.parent().unwrap().join(import_path);

        if import_full_path.exists() {
            // Recursively bundle the imported file
            let import_code = bundle_json_file(&import_full_path, seen_files)?;
            // Replace the $import statement with the content of the imported file
            modified_code = modified_code.replace(&cap[0], &import_code);
        }
    }

    // Append the processed code to the bundled code
    bundled_code.push_str(&modified_code);
    bundled_code.push_str("\n");

    Ok(bundled_code)
}

fn main() -> io::Result<()> {
    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();
    
    // Ensure the correct number of arguments are provided
    if args.len() < 3 {
        eprintln!("Usage: {} <input_glob_pattern> <output_file>", args[0]);
        return Ok(());
    }

    let input_pattern = &args[1];
    let output_file = &args[2];

    let mut bundled_code = String::new();
    let mut seen_files = HashSet::new();

    // Process all files matching the input glob pattern
    for entry in glob(input_pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                // Check if the file has a .js extension
                if path.extension().and_then(|s| s.to_str()) == Some("js") {
                    // Bundle the JavaScript file
                    let code = bundle_js_file(&path, &mut seen_files)?;
                    // Append the bundled code to the final output
                    bundled_code.push_str(&code);
                } 
                // Check if the file has a .css extension
                else if path.extension().and_then(|s| s.to_str()) == Some("css") {
                    // Bundle the CSS file
                    let code = bundle_css_file(&path, &mut seen_files)?;
                    // Append the bundled code to the final output
                    bundled_code.push_str(&code);
                } 
                // Check if the file has a .html extension
                else if path.extension().and_then(|s| s.to_str()) == Some("html") {
                    // Bundle the HTML file
                    let code = bundle_html_file(&path, &mut seen_files)?;
                    // Append the bundled code to the final output
                    bundled_code.push_str(&code);
                } 
                // Check if the file has a .json extension
                else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    // Bundle the JSON file
                    let code = bundle_json_file(&path, &mut seen_files)?;
                    // Append the bundled code to the final output
                    bundled_code.push_str(&code);
                }
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }

    // Write the final bundled code to the output file
    let mut output = fs::File::create(output_file)?;
    output.write_all(bundled_code.as_bytes())?;

    Ok(())
}