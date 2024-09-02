// Import necessary crates for WASM parsing and encoding
use wasmparser::{Parser, Payload, Type, FunctionType, ModuleReader};
use wasm_encoder::{Module, Function, Instruction, Type as EncodedType};
use std::fs::File;
use std::io::{self, Read, Write};

// Function to handle parsing errors
fn handle_parse_error(error: wasmparser::ParseError) {
    eprintln!("Parse error: {:?}", error);
}

// Function to add a function to the WASM module
fn add_function_to_module(module: &mut Module, index: u32) {
    module.function()
        .params(&[EncodedType::I32])
        .returns(&[EncodedType::I32])
        .body(|b| {
            b.instruction(Instruction::I32Const(42)) // Example instruction
        });
}

// Function to modify an existing function in the WASM module
fn modify_function_in_module(module: &mut Module, function_index: u32) {
    // For demonstration, modify the function at `function_index`
    // This assumes you know the function signature and type
    module.function()
        .params(&[EncodedType::I32])
        .returns(&[EncodedType::I32])
        .body(|b| {
            b.instruction(Instruction::I32Add) // Example instruction
        });
}

// Function to add an import to the WASM module
fn add_import_to_module(module: &mut Module, module_name: &str, field_name: &str, func_index: u32) {
    module.import()
        .module(module_name)
        .name(field_name)
        .kind(wasm_encoder::ImportKind::Function)
        .type_(func_index);
}

// Function to extract and print function information from the Function section
fn print_function_info(functions: &[u32]) {
    for func in functions {
        println!("Function index: {}", func);
    }
}

fn main() -> io::Result<()> {
    // Define input and output file paths
    let input_file = "input.wasm";
    let output_file = "output.wasm";

    // Open and read the input WASM file into a byte vector
    let mut file = File::open(input_file)?;
    let mut wasm_bytes = Vec::new();
    file.read_to_end(&mut wasm_bytes)?;

    // Create a new WASM module to hold the transformed code
    let mut module = Module::default();

    // Initialize the WASM parser with a start offset of 0
    let mut parser = Parser::new(0);
    let mut function_types = Vec::new();
    let mut functions = Vec::new(); // To store function indices

    // Parse the WASM bytes
    parser.parse_all(&wasm_bytes).for_each(|payload| {
        match payload {
            Ok(Payload::TypeSection(types)) => {
                for type_ in types {
                    match type_ {
                        Ok(Type::Function(func_type)) => {
                            function_types.push(func_type);
                        }
                        _ => {} // Ignore other types of sections
                    }
                }
            }
            Ok(Payload::FunctionSection(funcs)) => {
                for func in funcs {
                    match func {
                        Ok(index) => {
                            functions.push(index);
                        }
                        Err(e) => {
                            eprintln!("Error parsing function index: {:?}", e);
                        }
                    }
                }
            }
            Ok(Payload::ExportSection(exports)) => {
                for export in exports {
                    match export {
                        Ok(export) => {
                            println!("Exported: {} as {:?}", export.name, export.kind);
                        }
                        Err(e) => {
                            eprintln!("Error parsing export: {:?}", e);
                        }
                    }
                }
            }
            Ok(Payload::ImportSection(imports)) => {
                for import in imports {
                    match import {
                        Ok(import) => {
                            println!("Imported: {} from module {:?}", import.field, import.module);
                        }
                        Err(e) => {
                            eprintln!("Error parsing import: {:?}", e);
                        }
                    }
                }
            }
            _ => {}
        }
    });

    // Add a new function to the module
    let function_type_index = function_types.len() as u32;
    add_function_to_module(&mut module, function_type_index);

    // Modify an existing function if necessary
    if !functions.is_empty() {
        modify_function_in_module(&mut module, functions[0]);
    }

    // Add a new import to the module
    add_import_to_module(&mut module, "env", "imported_function", function_type_index);

    // Add a new export for the newly added function
    module.export()
        .name("my_function")
        .kind(wasm_encoder::ExportKind::Function)
        .index(function_type_index);

    // Encode the final WASM module and write it to the output file
    let mut output = File::create(output_file)?;
    let wasm_bytes = module.finish(); // Finalize the WASM module and get the encoded bytes
    output.write_all(&wasm_bytes)?;

    println!("WASM transformation complete. Output written to {}", output_file);
    Ok(())
}use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct Mapping {
    original_line: usize,
    original_column: usize,
    generated_line: usize,
    generated_column: usize,
    name_index: Option<usize>,
}

fn generate_source_map(
    sources: Vec<PathBuf>,
    file: &Path,
    sources_content: Vec<String>,
    mappings: Vec<Mapping>,
    names: Vec<String>
) -> io::Result<()> {
    let source_paths: Vec<String> = sources.into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    
    let source_map = json!({
        "version": 3,
        "file": file.file_name().unwrap_or(&"output.js".into()).to_string_lossy(),
        "sources": source_paths,
        "sourcesContent": sources_content,
        "names": names,
        "mappings": generate_mappings_string(mappings)
    });

    let mut file = fs::File::create(file.with_extension("map"))?;
    file.write_all(source_map.to_string().as_bytes())?;
    Ok(())
}

fn generate_mappings_string(mappings: Vec<Mapping>) -> String {
    let mut lines: HashMap<usize, Vec<Mapping>> = HashMap::new();
    
    for mapping in mappings {
        lines.entry(mapping.generated_line).or_default().push(mapping);
    }

    lines.into_iter()
        .map(|(line, mappings)| {
            let segments: Vec<String> = mappings.into_iter().map(|mapping| {
                let mut seg = String::new();
                seg.push_str(&mapping.original_line.to_string());
                seg.push_str(":");
                seg.push_str(&mapping.original_column.to_string());
                seg.push_str(",");
                seg.push_str(&mapping.generated_column.to_string());
                seg.push_str(",");
                seg.push_str(&mapping.name_index.unwrap_or(0).to_string());
                seg
            }).collect();
            segments.join(",")
        })
        .collect::<Vec<String>>()
        .join(";")
}

fn generate_detailed_source_map(
    sources: Vec<PathBuf>,
    file: &Path,
    sources_content: Vec<String>,
    mappings: Vec<Mapping>,
    names: Vec<String>
) -> io::Result<()> {
    let source_paths: Vec<String> = sources.into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    
    let source_map = json!({
        "version": 3,
        "file": file.file_name().unwrap_or(&"output.js".into()).to_string_lossy(),
        "sources": source_paths,
        "sourcesContent": sources_content,
        "names": names,
        "mappings": generate_mappings_string(mappings),
        "sourceRoot": "",
        "x_filenames": source_paths,
        "x_sources_content": sources_content
    });

    let mut file = fs::File::create(file.with_extension("detailed_map"))?;
    file.write_all(source_map.to_string().as_bytes())?;
    Ok(())
}

fn load_sources_and_generate_mappings(source_paths: Vec<PathBuf>) -> (Vec<String>, Vec<Mapping>) {
    let sources_content: Vec<String> = source_paths.iter()
        .map(|path| fs::read_to_string(path).unwrap_or_else(|_| String::new()))
        .collect();

    let mappings = vec![
        Mapping { original_line: 1, original_column: 0, generated_line: 1, generated_column: 0, name_index: Some(0) },
        Mapping { original_line: 2, original_column: 5, generated_line: 2, generated_column: 10, name_index: Some(1) },
    ];

    (sources_content, mappings)
}

fn validate_source_map(file: &Path) -> io::Result<()> {
    let source_map_content = fs::read_to_string(file)?;
    let source_map: serde_json::Value = serde_json::from_str(&source_map_content)?;

    if !source_map["version"].is_number() || !matches!(source_map["version"].as_u64(), Some(3 | 2)) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported source map version"));
    }

    if !source_map["file"].is_string() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Missing or invalid file field"));
    }

    if !source_map["sources"].is_array() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Missing or invalid sources field"));
    }

    if !source_map["sourcesContent"].is_array() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Missing or invalid sourcesContent field"));
    }

    if !source_map["names"].is_array() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Missing or invalid names field"));
    }

    if !source_map["mappings"].is_string() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Missing or invalid mappings field"));
    }

    Ok(())
}

fn merge_source_maps(maps: Vec<serde_json::Value>) -> serde_json::Value {
    let mut sources = HashSet::new();
    let mut sources_content = HashSet::new();
    let mut names = HashSet::new();
    let mut mappings = Vec::new();
    let mut file = "merged.js".to_string();

    for map in maps {
        sources.extend(map["sources"].as_array().unwrap_or(&vec![]).iter().cloned());
        sources_content.extend(map["sourcesContent"].as_array().unwrap_or(&vec![]).iter().cloned());
        names.extend(map["names"].as_array().unwrap_or(&vec![]).iter().cloned());
        mappings.push(map["mappings"].as_str().unwrap_or("").to_string());
    }

    json!({
        "version": 3,
        "file": file,
        "sources": sources.into_iter().collect::<Vec<_>>(),
        "sourcesContent": sources_content.into_iter().collect::<Vec<_>>(),
        "names": names.into_iter().collect::<Vec<_>>(),
        "mappings": mappings.concat()
    })
}

fn compress_source_map(file: &Path) -> io::Result<()> {
    let source_map_content = fs::read_to_string(file)?;
    let mut source_map: serde_json::Value = serde_json::from_str(&source_map_content)?;

    let mut mappings = source_map["mappings"].as_str().unwrap_or("").to_string();
    // Simple compression: remove redundant characters (you can implement a more complex compression if needed)
    mappings = mappings.replace(";;;;", ";");

    source_map["mappings"] = serde_json::Value::String(mappings);

    let mut file = fs::File::create(file.with_extension("compressed_map"))?;
    file.write_all(source_map.to_string().as_bytes())?;
    Ok(())
}

fn main() -> io::Result<()> {
    let source_paths = vec![
        PathBuf::from("src/main.ts"),
        PathBuf::from("src/utils.ts"),
    ];

    let (sources_content, mappings) = load_sources_and_generate_mappings(source_paths.clone());

    let names = vec![
        "variableName".to_string(),
        "functionName".to_string(),
    ];

    let output_file = Path::new("dist/output.js");

    generate_detailed_source_map(source_paths.clone(), output_file, sources_content.clone(), mappings.clone(), names.clone())?;

    validate_source_map(&output_file.with_extension("detailed_map"))?;

    let maps = vec![
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string("dist/output.js.map")?)?,
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string("dist/output.js.detailed_map")?)?,
    ];
    
    let merged_map = merge_source_maps(maps);

    let mut file = fs::File::create(output_file.with_extension("merged_map"))?;
    file.write_all(merged_map.to_string().as_bytes())?;

    compress_source_map(&output_file.with_extension("merged_map"))?;

    Ok(())
}