use regex::Regex;
use std::env;
use std::fs;
use std::process;

/// Main entry point of the linter.
fn main() {
    // Collect command-line arguments.
    let args: Vec<String> = env::args().collect();

    // Check if the correct number of arguments are provided.
    if args.len() != 3 {
        eprintln!("Usage: {} <file> <type>", args[0]);
        eprintln!("Types: html, css, js");
        process::exit(1);
    }

    // Extract filename and file type from arguments.
    let filename = &args[1];
    let file_type = &args[2];
    
    // Read the file content.
    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file: {}", err);
            process::exit(1);
        }
    };

    // Run the appropriate check based on the file type.
    let issues = match file_type.as_str() {
        "html" => check_html(&content),
        "css" => check_css(&content),
        "js" => check_js(&content),
        _ => {
            eprintln!("Unsupported file type: {}", file_type);
            process::exit(1);
        }
    };

    // Output the issues found or a message indicating no issues.
    if issues.is_empty() {
        println!("No issues found.");
    } else {
        for issue in issues {
            println!("{}", issue);
        }
    }
}

/// Check HTML for common issues.
fn check_html(content: &str) -> Vec<String> {
    let mut issues = Vec::new();

    // Regex to detect unclosed tags.
    let unclosed_tag_re = Regex::new(r"<([a-zA-Z][^\s>/]*)(?![^>]*<\/\1)[^>]*>").unwrap();
    
    // Regex to detect missing alt attributes in img tags.
    let missing_alt_re = Regex::new(r"<img(?![^>]*\salt=)[^>]*>").unwrap();
    
    // Regex to detect tags with multiple spaces between attributes.
    let multiple_spaces_re = Regex::new(r"<[^>]*\s\s[^>]*>").unwrap();
    
    // Regex to detect inline styles (not recommended).
    let inline_styles_re = Regex::new(r"<[^>]*style\s*=\s*\'[^\']*\'[^>]*>").unwrap();
    
    // Regex to detect missing closing tags (basic check).
    let missing_closing_tag_re = Regex::new(r"<[a-zA-Z][^\s>/]*[^>]*>(?!.*<\/[a-zA-Z][^\s>/]*>)").unwrap();

    // Regex to detect missing doctype.
    let missing_doctype_re = Regex::new(r"(?i)(<!DOCTYPE\s+html>)").unwrap();
    
    // Regex to detect empty tags.
    let empty_tag_re = Regex::new(r"<([a-zA-Z][^\s>/]*)(?![^>]*\/>)\s*[^>]*>\s*<\/\1>").unwrap();
    
    // Regex to detect deprecated tags.
    let deprecated_tags_re = Regex::new(r"</?(font|center|marquee|big|strike|tt)[^>]*>").unwrap();

    // Iterate through each line of the HTML content.
    for (line_number, line) in content.lines().enumerate() {
        // Check for missing doctype.
        if !missing_doctype_re.is_match(line) {
            issues.push(format!("Line {}: Missing doctype declaration", line_number + 1));
        }
        // Check for unclosed tags.
        if unclosed_tag_re.is_match(line) {
            issues.push(format!("Line {}: Unclosed tag detected", line_number + 1));
        }
        // Check for missing alt attributes in <img> tags.
        if missing_alt_re.is_match(line) {
            issues.push(format!("Line {}: Missing alt attribute in <img> tag", line_number + 1));
        }
        // Check for multiple spaces between attributes.
        if multiple_spaces_re.is_match(line) {
            issues.push(format!("Line {}: Multiple spaces between attributes", line_number + 1));
        }
        // Check for inline styles.
        if inline_styles_re.is_match(line) {
            issues.push(format!("Line {}: Inline styles detected", line_number + 1));
        }
        // Check for missing closing tags (basic check).
        if missing_closing_tag_re.is_match(line) {
            issues.push(format!("Line {}: Potential missing closing tag", line_number + 1));
        }
        // Check for empty tags.
        if empty_tag_re.is_match(line) {
            issues.push(format!("Line {}: Empty tag detected", line_number + 1));
        }
        // Check for deprecated tags.
        if deprecated_tags_re.is_match(line) {
            issues.push(format!("Line {}: Deprecated tag detected", line_number + 1));
        }
    }

    issues
}

/// Check CSS for common issues.
fn check_css(content: &str) -> Vec<String> {
    let mut issues = Vec::new();

    // Regex to detect missing semicolons before closing braces.
    let missing_semicolon_re = Regex::new(r"[^;\s}\n]}\s*").unwrap();
    
    // Regex to detect duplicate properties within the same selector.
    let duplicate_properties_re = Regex::new(r"(?s)(?P<selector>[^{]+)\{(?P<properties>[^}]+)\}\s*(?P=selector)\{(?P=properties)\}").unwrap();
    
    // Regex to detect empty rules (e.g., .class{} with no properties).
    let empty_rule_re = Regex::new(r"[^{]+\{\s*\}").unwrap();
    
    // Regex to detect invalid property names (basic check).
    let invalid_property_re = Regex::new(r"[^{]*\{\s*[^;]+[^};\s]*\s*[^}\s]*\s*\}").unwrap();
    
    // Regex to detect invalid hex color codes.
    let invalid_hex_color_re = Regex::new(r"#[^0-9a-fA-F]{1,6}[^a-fA-F]|\b#[^0-9a-fA-F]{1,6}\b").unwrap();
    
    // Regex to detect non-standard properties (vendor prefixes).
    let non_standard_properties_re = Regex::new(r"(?i)\b(?:-webkit-|-moz-|-ms-|-o-)\w+").unwrap();
    
    // Regex to detect CSS hacks.
    let css_hacks_re = Regex::new(r"(?i)\/\*[^*]*\*\/").unwrap();

    // Iterate through each line of the CSS content.
    for (line_number, line) in content.lines().enumerate() {
        // Check for missing semicolons before closing braces.
        if missing_semicolon_re.is_match(line) {
            issues.push(format!("Line {}: Missing semicolon before closing brace", line_number + 1));
        }
        // Check for duplicate CSS properties.
        if duplicate_properties_re.is_match(line) {
            issues.push(format!("Line {}: Duplicate CSS properties detected", line_number + 1));
        }
        // Check for empty CSS rules.
        if empty_rule_re.is_match(line) {
            issues.push(format!("Line {}: Empty CSS rule detected", line_number + 1));
        }
        // Check for invalid property names.
        if invalid_property_re.is_match(line) {
            issues.push(format!("Line {}: Invalid property detected", line_number + 1));
        }
        // Check for invalid hex color codes.
        if invalid_hex_color_re.is_match(line) {
            issues.push(format!("Line {}: Invalid hex color code detected", line_number + 1));
        }
        // Check for non-standard CSS properties (vendor prefixes).
        if non_standard_properties_re.is_match(line) {
            issues.push(format!("Line {}: Non-standard CSS property detected", line_number + 1));
        }
        // Check for CSS hacks.
        if css_hacks_re.is_match(line) {
            issues.push(format!("Line {}: CSS hack detected", line_number + 1));
        }
    }

    issues
}

/// Check JavaScript for common issues.
fn check_js(content: &str) -> Vec<String> {
    let mut issues = Vec::new();

    // Regex to detect missing semicolons.
    let missing_semicolon_re = Regex::new(r"[^;\s}\n]}\s*").unwrap();
    
    // Regex to detect console.log statements.
    let console_log_re = Regex::new(r"console\.log\(").unwrap();
    
    // Regex to detect unused variables (e.g., defined but not used).
    let unused_variable_re = Regex::new(r"\bvar\b|\blet\b|\bconst\b[^;]*;[^}]*\b\w+\b").unwrap();
    
    // Regex to detect potentially unsafe eval usage.
    let eval_re = Regex::new(r"eval\(").unwrap();
    
    // Regex to detect var usage instead of let/const.
    let var_usage_re = Regex::new(r"\bvar\b\s+\w+[^;]*;").unwrap();
    
    // Regex to detect functions with no names.
    let anonymous_function_re = Regex::new(r"function\s+\(\s*\)\s*\{").unwrap();
    
    // Regex to detect const variables without initialization.
    let uninitialized_const_re = Regex::new(r"\bconst\b\s+\w+\s*[^=]").unwrap();

    // Iterate through each line of the JavaScript content.
    for (line_number, line) in content.lines().enumerate() {
        // Check for missing semicolons.
        if missing_semicolon_re.is_match(line) {
            issues.push(format!("Line {}: Missing semicolon", line_number + 1));
        }
        // Check for console.log statements.
        if console_log_re.is_match(line) {
            issues.push(format!("Line {}: console.log() detected", line_number + 1));
        }
        // Check for unused variables.
        if unused_variable_re.is_match(line) {
            issues.push(format!("Line {}: Potential unused variable", line_number + 1));
        }
        // Check for eval usage.
        if eval_re.is_match(line) {
            issues.push(format!("Line {}: Use of eval() detected", line_number + 1));
        }
        // Check for var usage instead of let/const.
        if var_usage_re.is_match(line) {
            issues.push(format!("Line {}: Usage of 'var' instead of 'let' or 'const'", line_number + 1));
        }
        // Check for anonymous functions.
        if anonymous_function_re.is_match(line) {
            issues.push(format!("Line {}: Anonymous function detected", line_number + 1));
        }
        // Check for uninitialized const variables.
        if uninitialized_const_re.is_match(line) {
            issues.push(format!("Line {}: Const variable declared but not initialized", line_number + 1));
        }
    }

    issues
}