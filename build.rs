use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir: String = env::var("OUT_DIR").unwrap();
    let dest_path: std::path::PathBuf = Path::new(&out_dir).join("modules.rs");

    let mut file: File = File::create(dest_path).unwrap();

    let src_dir: &Path = Path::new("src");
    for entry in fs::read_dir(src_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "rs" {
                    let file_stem = path.file_stem().unwrap().to_str().unwrap();
                    if file_stem != "main" {
                        writeln!(file, "mod {};", file_stem).unwrap();
                    }
                }
            }
        }
    }
}