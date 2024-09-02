use regex::Regex;

/// Minifies HTML content by removing comments, reducing whitespace,
/// and collapsing unnecessary spaces.
///
/// # Arguments
///
/// * `content` - A string slice that holds the HTML content to be minified.
///
/// # Returns
///
/// * A `String` containing the minified HTML content.
pub fn minify_html(content: &str) -> String {
    // Regular expression to match HTML comments
    let comments = Regex::new(r"(?s)<!--.*?-->").unwrap();
    // Regular expression to match sequences of two or more whitespace characters
    let spaces = Regex::new(r"\s{2,}").unwrap();
    // Regular expression to match leading and trailing whitespace on each line
    let trim = Regex::new(r"(?m)^\s+|\s+$").unwrap();
    // Regular expression to match sequences of whitespace between HTML tags
    let tags = Regex::new(r">\s+<").unwrap();
    // Regular expression to match empty lines
    let empty_lines = Regex::new(r"(?m)^\s*\n").unwrap();
    // Regular expression to match extra spaces between attributes
    let extra_spaces_between_attrs = Regex::new(r"\s+(\w+)=\s*").unwrap();
    // Regular expression to remove extra spaces around DOCTYPE declaration
    let remove_doctype_spaces = Regex::new(r"(?i)\s*<!DOCTYPE[^>]+>\s*").unwrap();
    // Regular expression to remove optional closing tags
    let remove_optional_closing_tags = Regex::new(r"(?i)</(li|dt|dd|p|colgroup|thead|tfoot|tbody|tr|th|td)>").unwrap();
    // Regular expression to collapse multiple spaces within tags
    let collapse_whitespace_in_tags = Regex::new(r"\s*(<[^>]+>)\s*").unwrap();

    // Remove HTML comments
    let no_comments = comments.replace_all(content, " ");
    // Trim leading and trailing whitespace from each line
    let trimmed = trim.replace_all(&no_comments, "");
    // Remove extra whitespace between tags
    let no_tags = tags.replace_all(&trimmed, "><");
    // Remove empty lines
    let no_empty_lines = empty_lines.replace_all(&no_tags, "");
    // Remove extra spaces around attributes
    let no_extra_spaces_attrs = extra_spaces_between_attrs.replace_all(&no_empty_lines, " $1=");
    // Remove extra spaces around DOCTYPE declaration
    let no_doctype_spaces = remove_doctype_spaces.replace_all(&no_extra_spaces_attrs, "");
    // Strip unnecessary quotes around attribute values
    let no_quotes = strip_quotes.replace_all(&no_doctype_spaces, "=$1");
    // Remove optional closing tags
    let no_closing_tags = remove_optional_closing_tags.replace_all(&no_quotes, "");
    // Collapse multiple spaces within tags
    let minified = collapse_whitespace_in_tags.replace_all(&no_closing_tags, "$1");

    minified.to_string()
}

/// Minifies CSS content by removing comments, reducing whitespace,
/// and collapsing unnecessary spaces.
///
/// # Arguments
///
/// * `content` - A string slice that holds the CSS content to be minified.
///
/// # Returns
///
/// * A `String` containing the minified CSS content.
pub fn minify_css(content: &str) -> String {
    // Regular expression to match CSS comments
    let comments = Regex::new(r"(?s)/\*.*?\*/").unwrap();
    // Regular expression to match sequences of two or more whitespace characters
    let spaces = Regex::new(r"\s{2,}").unwrap();
    // Regular expression to match leading and trailing whitespace on each line
    let trim = Regex::new(r"(?m)^\s+|\s+$").unwrap();
    // Regular expression to match semicolons followed by optional whitespace
    let semicolons = Regex::new(r";\s*").unwrap();
    // Regular expression to remove whitespace around braces
    let remove_whitespace_around_braces = Regex::new(r"\s*{\s*|\s*}\s*").unwrap();
    // Regular expression to remove whitespace around colons
    let remove_whitespace_around_colons = Regex::new(r"\s*:\s*").unwrap();
    // Regular expression to remove whitespace around commas
    let remove_whitespace_around_commas = Regex::new(r"\s*,\s*").unwrap();
    // Regular expression to remove whitespace around operators
    let remove_whitespace_around_operators = Regex::new(r"\s*([\+\-\*/])\s*").unwrap();

    // Remove CSS comments
    let no_comments = comments.replace_all(content, " ");
    // Trim leading and trailing whitespace from each line
    let trimmed = trim.replace_all(&no_comments, "");
    // Remove unnecessary semicolons
    let no_semicolons = semicolons.replace_all(&trimmed, ";");
    // Remove whitespace around braces
    let no_braces = remove_whitespace_around_braces.replace_all(&no_semicolons, "{}");
    // Remove whitespace around colons
    let no_colons = remove_whitespace_around_colons.replace_all(&no_braces, ":");
    // Remove whitespace around commas
    let no_commas = remove_whitespace_around_commas.replace_all(&no_colons, ",");
    // Remove whitespace around operators
    let minified = remove_whitespace_around_operators.replace_all(&no_commas, "$1");

    minified.to_string()
}

/// Minifies JavaScript content by removing comments, reducing whitespace,
/// and collapsing unnecessary spaces.
///
/// # Arguments
///
/// * `content` - A string slice that holds the JavaScript content to be minified.
///
/// # Returns
///
/// * A `String` containing the minified JavaScript content.
pub fn minify_js(content: &str) -> String {
    // Regular expression to match JavaScript comments
    let comments = Regex::new(r"(?s)//.*?(\r?\n)|/\*.*?\*/").unwrap();
    // Regular expression to match sequences of two or more whitespace characters
    let spaces = Regex::new(r"\s{2,}").unwrap();
    // Regular expression to match leading and trailing whitespace on each line
    let trim = Regex::new(r"(?m)^\s+|\s+$").unwrap();
    // Regular expression to match whitespace around braces, parentheses, and brackets
    let brackets = Regex::new(r"\s*([{}()])\s*").unwrap();
    // Regular expression to remove whitespace around operators
    let remove_whitespace_around_operators = Regex::new(r"\s*([\+\-\*/=<>!])\s*").unwrap();
    // Regular expression to remove whitespace around commas
    let remove_whitespace_around_commas = Regex::new(r"\s*,\s*").unwrap();
    // Regular expression to remove whitespace around colons
    let remove_whitespace_around_colons = Regex::new(r"\s*:\s*").unwrap();
    // Regular expression to remove extra semicolons
    let remove_extra_semicolons = Regex::new(r";+\s*").unwrap();
    // Regular expression to collapse empty blocks
    let collapse_empty_blocks = Regex::new(r"\{\s*\}").unwrap();

    // Remove JavaScript comments
    let no_comments = comments.replace_all(content, " ");
    // Trim leading and trailing whitespace from each line
    let trimmed = trim.replace_all(&no_comments, "");
    // Remove whitespace around braces, parentheses, and brackets
    let no_brackets = brackets.replace_all(&trimmed, "$1");
    // Remove whitespace around operators   
    let no_operators = remove_whitespace_around_operators.replace_all(&no_brackets, "$1");
    // Remove whitespace around commas
    let no_commas = remove_whitespace_around_commas.replace_all(&no_operators, ",");
    // Remove whitespace around colons
    let no_colons = remove_whitespace_around_colons.replace_all(&no_commas, ":");
    // Remove extra semicolons
    let no_extra_semicolons = remove_extra_semicolons.replace_all(&no_colons, ";");
    // Collapse empty blocks
    let minified = collapse_empty_blocks.replace_all(&no_extra_semicolons, "{}");

    minified.to_string()
}