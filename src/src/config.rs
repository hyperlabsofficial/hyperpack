pub struct Config {
    pub entry_file: String,
    pub output_file: String,
}

impl Config {
    pub fn new(entry_file: &str, output_file: &str) -> Self {
        Self {
            entry_file: entry_file.to_string(),
            output_file: output_file.to_string(),
        }
    }
}