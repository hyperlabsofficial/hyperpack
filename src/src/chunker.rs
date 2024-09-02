use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::thread;
use clap::{Arg, Command};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use walkdir::WalkDir;

// Function to generate a random string for file and folder names
fn generate_random_name(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

// Function to chunk the content of a file into smaller parts
fn chunk_file_content(content: &str, chunk_size: usize) -> Vec<String> {
    content
        .as_bytes()
        .chunks(chunk_size)
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect()
}

// Function to read a file's content
fn read_file(file_path: &Path) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

// Function to write chunks into files within a randomly named directory
fn write_chunks_to_files(base_path: &Path, chunks: Vec<String>, ext: &str) -> io::Result<()> {
    let dir_name = generate_random_name(10);
    let output_dir = base_path.join(&dir_name);

    fs::create_dir_all(&output_dir)?;

    for (i, chunk) in chunks.iter().enumerate() {
        let file_name = format!("{}.{}", generate_random_name(10), ext);
        let file_path = output_dir.join(file_name);

        let mut file = File::create(file_path)?;
        file.write_all(chunk.as_bytes())?;
    }

    Ok(())
}

// Function to process files in a directory
fn process_files(input_dir: &Path, output_dir: &Path, chunk_size: usize) -> io::Result<()> {
    let (tx, rx) = channel();

    for entry in WalkDir::new(input_dir).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let file_path = entry.path().to_path_buf();
            let output_dir = output_dir.to_path_buf();
            let tx = tx.clone();

            thread::spawn(move || {
                if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                    match ext {
                        "js" | "css" | "html" => {
                            if let Ok(content) = read_file(&file_path) {
                                let chunks = chunk_file_content(&content, chunk_size);
                                let _ = write_chunks_to_files(&output_dir, chunks, ext);
                                let _ = tx.send(format!("Processed: {:?}", file_path));
                            }
                        },
                        _ => {},
                    }
                }
            });
        }
    }

    drop(tx); // Close the channel

    for received in rx {
        println!("{}", received);
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let matches = Command::new("chunker")
        .arg(Arg::new("input")
            .short('i')
            .long("input")
            .value_name("INPUT_DIR")
            .about("Sets the input directory to use")
            .takes_value(true)
            .default_value("input"))
        .arg(Arg::new("output")
            .short('o')
            .long("output")
            .value_name("OUTPUT_DIR")
            .about("Sets the output directory to use")
            .takes_value(true)
            .default_value("dist/static"))
        .arg(Arg::new("chunk_size")
            .short('c')
            .long("chunk_size")
            .value_name("CHUNK_SIZE")
            .about("Sets the chunk size in bytes")
            .takes_value(true)
            .default_value("1024"))
        .get_matches();

    let input_dir = PathBuf::from(matches.value_of("input").unwrap());
    let output_dir = PathBuf::from(matches.value_of("output").unwrap());
    let chunk_size: usize = matches.value_of("chunk_size").unwrap().parse().expect("Invalid chunk size");

    fs::create_dir_all(&output_dir)?;

    process_files(&input_dir, &output_dir, chunk_size)?;

    Ok(())
}