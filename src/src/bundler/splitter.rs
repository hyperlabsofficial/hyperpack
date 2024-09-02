use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use glob::glob;

/// Generates a random string of specified length.
///
/// # Arguments
///
/// * `length` - The length of the random string to generate.
///
/// # Returns
///
/// * A random string of the specified length.
fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Creates directories for CSS, HTML, and JS chunks if they don't exist.
///
/// # Arguments
///
/// * `output_dir` - The base output directory where chunk folders will be created.
fn create_chunk_directories(output_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(output_dir.join("css"))?;
    fs::create_dir_all(output_dir.join("html"))?;
    fs::create_dir_all(output_dir.join("js"))?;
    Ok(())
}

/// Writes chunk metadata to a manifest file.
///
/// # Arguments
///
/// * `manifest_path` - The path where the manifest file will be saved.
/// * `chunk_metadata` - A vector of tuples containing chunk names and their paths.
fn write_manifest(manifest_path: &Path, chunk_metadata: Vec<(String, String)>) -> io::Result<()> {
    let mut manifest_file = fs::File::create(manifest_path)?;
    for (name, path) in chunk_metadata {
        writeln!(manifest_file, "{}: {}", name, path)?;
    }
    Ok(())
}

/// Splits bundled JavaScript, CSS, and HTML files into separate chunks.
///
/// # Arguments
///
/// * `entry_file` - The entry file that starts the splitting process.
/// * `output_dir` - The directory where the split files will be saved.
/// * `seen_files` - A set of already processed files to avoid duplication.
/// * `chunk_metadata` - A vector to store metadata about chunks created.
fn split_code(
    entry_file: &Path,
    output_dir: &Path,
    seen_files: &mut HashSet<PathBuf>,
    chunk_metadata: &mut Vec<(String, String)>
) -> io::Result<()> {
    if seen_files.contains(entry_file) {
        return Ok(());
    }
    seen_files.insert(entry_file.to_path_buf());

    // Read the content of the entry file
    let code = fs::read_to_string(entry_file)?;

    // Regex to match import statements (JS and CSS)
    let re = Regex::new(r#"(?i)import\s+["']([^"']+)["'];?"#).unwrap();
    let mut remaining_code = code.clone();

    // Create directories for CSS, HTML, and JS chunks
    create_chunk_directories(output_dir)?;

    // Process each import statement
    for cap in re.captures_iter(&code) {
        let import_path = cap.get(1).unwrap().as_str();
        let import_full_path = entry_file.parent().unwrap().join(import_path);

        if import_full_path.exists() {
            // Recursively split the imported file
            split_code(&import_full_path, output_dir, seen_files, chunk_metadata)?;

            // Generate a random chunk name
            let chunk_name = format!("chunk_{}.{}", generate_random_string(6), import_full_path.extension().unwrap_or_default().to_str().unwrap());
            
            // Determine the folder based on file extension
            let chunk_folder = match import_full_path.extension().and_then(|s| s.to_str()) {
                Some("css") => output_dir.join("css"),
                Some("html") => output_dir.join("html"),
                Some("js") => output_dir.join("js"),
                _ => continue,
            };

            // Replace the import statement with a chunk loading mechanism
            let chunk_path = chunk_folder.join(&chunk_name);
            let chunk_loader = format!("loadChunk('{}');", chunk_name);
            remaining_code = remaining_code.replace(&cap[0], &chunk_loader);

            // Write the chunk file
            let mut chunk_file = fs::File::create(&chunk_path)?;
            let chunk_code = fs::read_to_string(&import_full_path)?;
            chunk_file.write_all(chunk_code.as_bytes())?;
            
            // Add chunk metadata
            chunk_metadata.push((chunk_name.clone(), chunk_path.to_string_lossy().into_owned()));
        }
    }

    // Write the remaining code (after splitting) to the output directory
    let output_file_path = output_dir.join(entry_file.file_name().unwrap());
    let mut output_file = fs::File::create(output_file_path)?;
    output_file.write_all(remaining_code.as_bytes())?;

    Ok(())
}

/// Function to load a chunk dynamically (placeholder for actual loading mechanism).
///
/// # Arguments
///
/// * `chunk_name` - The name of the chunk file to load.
fn load_chunk(chunk_name: &str) {
    println!("Loading chunk: {}", chunk_name);
}

fn main() -> io::Result<()> {
    // Define the entry point for the splitting process (e.g., "src/main.js").
    let entry_file = Path::new("src/main.js");
    // Define the output directory for split files.
    let output_dir = Path::new("dist");

    // Create the output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    let mut seen_files = HashSet::new();
    let mut chunk_metadata = Vec::new();

    // Start splitting the code from the entry file
    split_code(entry_file, output_dir, &mut seen_files, &mut chunk_metadata)?;

    // Write chunk metadata to a manifest file
    let manifest_path = output_dir.join("manifest.txt");
    write_manifest(&manifest_path, chunk_metadata)?;

    println!("Code splitting complete. Chunks saved to {:?}", output_dir);
    println!("Chunk metadata saved to {:?}", manifest_path);

    Ok(())
}