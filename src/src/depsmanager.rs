use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use clap::{App, Arg};
use std::error::Error;

// Struct to represent a dependency
#[derive(Serialize, Deserialize, Debug)]
struct Dependency {
    name: String,  // Name of the dependency
    path: String,  // Path to the dependency file
}

// Struct to manage the dependencies
#[derive(Debug)]
struct DependencyManager {
    dependencies: HashMap<String, Dependency>, // Mapping from dependency name to Dependency
}

impl DependencyManager {
    // Create a new DependencyManager
    fn new() -> Self {
        DependencyManager {
            dependencies: HashMap::new(),
        }
    }

    // Add a dependency to the manager
    fn add_dependency(&mut self, name: &str, path: &str) {
        let dep = Dependency {
            name: name.to_string(),
            path: path.to_string(),
        };
        self.dependencies.insert(name.to_string(), dep);
    }

    // Load dependencies from a file
    fn load_dependencies(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        // Read the file contents
        let content = fs::read_to_string(file_path)?;
        // Deserialize the JSON content into a vector of Dependency structs
        let deps: Vec<Dependency> = serde_json::from_str(&content)?;

        // Insert each dependency into the manager
        for dep in deps {
            self.add_dependency(&dep.name, &dep.path);
        }

        Ok(())
    }

    // Save the current state of dependencies to a file
    fn save_dependencies(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        // Serialize dependencies to JSON
        let deps: Vec<&Dependency> = self.dependencies.values().collect();
        let content = serde_json::to_string_pretty(&deps)?;
        // Write to file
        fs::write(file_path, content)?;
        Ok(())
    }

    // List all the dependencies
    fn list_dependencies(&self) {
        println!("Listing all dependencies:");
        for (name, dep) in &self.dependencies {
            println!("Name: {}, Path: {}", name, dep.path);
        }
    }

    // Resolve and print the path of a given dependency
    fn resolve_dependency(&self, name: &str) {
        match self.dependencies.get(name) {
            Some(dep) => println!("Path for {}: {}", name, dep.path),
            None => println!("Dependency {} not found", name),
        }
    }

    // Remove a dependency by name
    fn remove_dependency(&mut self, name: &str) {
        if self.dependencies.remove(name).is_some() {
            println!("Removed dependency: {}", name);
        } else {
            println!("Dependency {} not found", name);
        }
    }

    // Update the path of an existing dependency
    fn update_dependency(&mut self, name: &str, new_path: &str) {
        if let Some(dep) = self.dependencies.get_mut(name) {
            dep.path = new_path.to_string();
            println!("Updated dependency {} to new path: {}", name, new_path);
        } else {
            println!("Dependency {} not found", name);
        }
    }

    // Validate if a dependency path exists
    fn validate_dependency_path(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    // Load dependencies from the directory
    fn load_dependencies_from_dir(&mut self, dir_path: &str) -> Result<(), Box<dyn Error>> {
        // Read the directory contents
        let paths = fs::read_dir(dir_path)?;

        // Iterate over the entries
        for path in paths {
            let path = path?.path();
            // Only process JSON files
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Load dependencies from each JSON file
                self.load_dependencies(path.to_str().unwrap())?;
            }
        }
        Ok(())
    }
}

// Function to print the usage information
fn print_usage() {
    println!("Usage:");
    println!("  dependency_manager --load <file>  Load dependencies from a JSON file");
    println!("  dependency_manager --list        List all dependencies");
    println!("  dependency_manager --resolve <name>  Resolve a dependency by name");
    println!("  dependency_manager --remove <name>  Remove a dependency by name");
    println!("  dependency_manager --update <name> <path>  Update a dependency's path");
    println!("  dependency_manager --save <file>  Save dependencies to a JSON file");
    println!("  dependency_manager --load-dir <dir>  Load dependencies from all JSON files in a directory");
}

// Main function to handle command-line arguments and perform actions
fn main() {
    // Set up the command-line interface
    let matches = App::new("Dependency Manager")
        .version("0.1.0")
        .author("Your Name <your.email@example.com>")
        .about("Manages dependencies for HTML, CSS, JS, and TypeScript files")
        .arg(
            Arg::new("load")
                .short('l')
                .long("load")
                .takes_value(true)
                .about("Load dependencies from a JSON file"),
        )
        .arg(
            Arg::new("list")
                .short('t')
                .long("list")
                .about("List all dependencies"),
        )
        .arg(
            Arg::new("resolve")
                .short('r')
                .long("resolve")
                .takes_value(true)
                .about("Resolve a dependency by name"),
        )
        .arg(
            Arg::new("remove")
                .short('x')
                .long("remove")
                .takes_value(true)
                .about("Remove a dependency by name"),
        )
        .arg(
            Arg::new("update")
                .short('u')
                .long("update")
                .takes_value(true)
                .multiple_values(true)
                .about("Update a dependency's path"),
        )
        .arg(
            Arg::new("save")
                .short('s')
                .long("save")
                .takes_value(true)
                .about("Save dependencies to a JSON file"),
        )
        .arg(
            Arg::new("load-dir")
                .short('d')
                .long("load-dir")
                .takes_value(true)
                .about("Load dependencies from all JSON files in a directory"),
        )
        .get_matches();

    // Create a new DependencyManager
    let mut manager = DependencyManager::new();

    // Check if the 'load' argument is provided
    if let Some(file) = matches.value_of("load") {
        if let Err(e) = manager.load_dependencies(file) {
            eprintln!("Error loading dependencies: {}", e);
            return;
        }
    }

    // Check if the 'save' argument is provided
    if let Some(file) = matches.value_of("save") {
        if let Err(e) = manager.save_dependencies(file) {
            eprintln!("Error saving dependencies: {}", e);
            return;
        }
    }

    // Check if the 'list' argument is provided
    if matches.is_present("list") {
        manager.list_dependencies();
    }

    // Check if the 'resolve' argument is provided
    if let Some(name) = matches.value_of("resolve") {
        manager.resolve_dependency(name);
    }

    // Check if the 'remove' argument is provided
    if let Some(name) = matches.value_of("remove") {
        manager.remove_dependency(name);
    }

    // Check if the 'update' argument is provided
    if let Some(values) = matches.values_of("update") {
        let mut args = values.into_iter();
        if let (Some(name), Some(path)) = (args.next(), args.next()) {
            if manager.validate_dependency_path(path) {
                manager.update_dependency(name, path);
            } else {
                println!("The specified path does not exist: {}", path);
            }
        } else {
            eprintln!("Error: 'update' requires both a name and a new path.");
            print_usage();
        }
    }

    // Check if the 'load-dir' argument is provided
    if let Some(dir) = matches.value_of("load-dir") {
        if let Err(e) = manager.load_dependencies_from_dir(dir) {
            eprintln!("Error loading dependencies from directory: {}", e);
            return;
        }
    }

    // If no arguments are provided, print usage information
    if !matches.is_present("load") 
        && !matches.is_present("list") 
        && !matches.is_present("resolve") 
        && !matches.is_present("remove") 
        && !matches.is_present("update") 
        && !matches.is_present("save")
        && !matches.is_present("load-dir") {
        print_usage();
    }
}