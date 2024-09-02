use std::fs;
use std::io::prelude::*;
use regex::Regex;

/// Formats TypeScript and JavaScript code.
///
/// This function normalizes whitespace, handles indentation, and removes comments.
/// It supports basic formatting rules for code blocks, strings, and templates.
fn format_js_ts(content: &str) -> String {
    let mut result = String::new();
    let re_whitespace = Regex::new(r"\s+").unwrap();
    let re_single_line_comment = Regex::new(r"//.*").unwrap();
    let re_multi_line_comment = Regex::new(r"/\*[\s\S]*?\*/").unwrap();

    // Remove single-line and multi-line comments
    let no_comments = re_multi_line_comment.replace_all(
        &re_single_line_comment.replace_all(content, ""),
        ""
    );

    // Normalize spaces
    let normalized_spaces = re_whitespace.replace_all(&no_comments, " ").to_string();

    let mut in_string = false;
    let mut in_template = false;
    let mut indent_level = 0;

    for c in normalized_spaces.chars() {
        match c {
            '"' | '\'' => {
                in_string = !in_string;
                result.push(c);
            },
            '`' => {
                in_template = !in_template;
                result.push(c);
            },
            '{' if !in_string && !in_template => {
                indent_level += 1;
                result.push(c);
                result.push('\n');
                result.push_str(&"  ".repeat(indent_level));
            },
            '}' if !in_string && !in_template => {
                indent_level -= 1;
                result.push('\n');
                result.push_str(&"  ".repeat(indent_level));
                result.push(c);
            },
            ';' if !in_string && !in_template => {
                result.push(c);
                result.push('\n');
                result.push_str(&"  ".repeat(indent_level));
            },
            _ => result.push(c),
        }
    }
    
    result
}

/// Formats HTML code.
///
/// This function normalizes whitespace, handles indentation, and adjusts tag spacing.
fn format_html(content: &str) -> String {
    let mut result = String::new();
    let re_whitespace = Regex::new(r"\s+").unwrap();
    let re_tag_whitespace = Regex::new(r">\s+<").unwrap();

    // Normalize multiple spaces
    let normalized_spaces = re_whitespace.replace_all(content, " ").to_string();

    let mut indent_level = 0;
    let mut in_tag = false;
    let mut is_closing_tag = false;

    for c in normalized_spaces.chars() {
        match c {
            '<' => {
                if !in_tag {
                    if !is_closing_tag {
                        result.push('\n');
                        result.push_str(&"  ".repeat(indent_level));
                    }
                    in_tag = true;
                }
                result.push(c);
            },
            '>' => {
                result.push(c);
                in_tag = false;
                if is_closing_tag {
                    indent_level -= 1;
                } else {
                    indent_level += 1;
                }
                is_closing_tag = false;
            },
            '/' if in_tag => {
                is_closing_tag = true;
                result.push(c);
            },
            _ => result.push(c),
        }
    }

    // Adjust spacing between tags
    re_tag_whitespace.replace_all(&result, "><").to_string()
}

/// Formats CSS code.
///
/// This function normalizes whitespace, handles indentation for blocks, and removes comments.
fn format_css(content: &str) -> String {
    let mut result = String::new();
    let re_whitespace = Regex::new(r"\s+").unwrap();
    let re_comment = Regex::new(r"/\*[\s\S]*?\*/").unwrap();

    // Remove comments
    let no_comments = re_comment.replace_all(content, "");

    // Normalize spaces
    let normalized_spaces = re_whitespace.replace_all(&no_comments, " ").to_string();
    let mut in_block = false;
    let mut indent_level = 0;

    for c in normalized_spaces.chars() {
        match c {
            '{' => {
                if !in_block {
                    result.push(c);
                    result.push('\n');
                    result.push_str(&"  ".repeat(indent_level));
                    in_block = true;
                    indent_level += 1;
                }
            },
            '}' => {
                if in_block {
                    indent_level -= 1;
                    result.push('\n');
                    result.push_str(&"  ".repeat(indent_level));
                    result.push(c);
                    in_block = false;
                }
            },
            ';' if !in_block => {
                result.push(c);
                result.push('\n');
                result.push_str(&"  ".repeat(indent_level));
            },
            _ => result.push(c),
        }
    }

    result
}

/// Main function to format files.
///
/// Reads content from specified files, formats it based on the file type, and writes it back.
/// Supported file types: TypeScript, JavaScript, HTML, and CSS.
fn main() {
    let file_paths = vec![
        "example.ts",
        "example.js",
        "example.html",
        "example.css"
    ];

    for path in file_paths {
        match fs::read_to_string(path) {
            Ok(content) => {
                let formatted_content = match path.split('.').last().unwrap() {
                    "ts" | "js" => format_js_ts(&content),
                    "html" => format_html(&content),
                    "css" => format_css(&content),
                    _ => content,
                };

                let mut file = fs::File::create(path).expect("Unable to create file");
                file.write_all(formatted_content.as_bytes()).expect("Unable to write data");
            }
            Err(e) => eprintln!("Error reading {}: {}", path, e),
        }
    }
}