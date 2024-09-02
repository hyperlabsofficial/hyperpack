use std::process::Command;
use std::str;

fn check_module(module_name: &str) {
    // Use `npm ls` command to check if the module is installed globally.
    let output = Command::new("npm")
        .arg("ls")
        .arg("-g")
        .arg("--depth=0")
        .arg(module_name)
        .output()
        .expect("Failed to execute npm command");

    // Convert output to string and check for the presence of the module name
    let output_str = str::from_utf8(&output.stdout).unwrap();

    if output_str.contains(module_name) {
        println!("Module \"{}\" is installed globally.", module_name);
    } else {
        println!("Module \"{}\" is not installed globally.", module_name);
    }
}

fn main() {
    let module_name = "typescript"; // Change this to the module you want to check
    check_module(module_name);
}