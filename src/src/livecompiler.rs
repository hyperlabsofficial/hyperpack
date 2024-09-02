use warp::Filter;
use serde::{Deserialize, Serialize};
use swc_common::{FileName, SourceMap, Globals};
use swc_ecmascript::parser::{Syntax, TsConfig, Parser};
use swc_ecmascript::transforms::{resolver::Resolver, typescript::TsTransform, react::React};
use swc_ecmascript::visit::VisitMut;
use swc_ecmascript::codegen::{Emitter, CodeGenerator};
use thiserror::Error;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Write;
use std::fmt;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
struct CompileRequest {
    files: Option<HashMap<String, String>>,
    code: Option<String>,
    minify: Option<bool>,
    syntax: Option<String>,
    bundle: Option<bool>,
    react: Option<bool>,
    ts: Option<bool>,
    extra_options: Option<HashMap<String, Value>>,
    source_map: Option<bool>,
    globals: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
struct CompileResponse {
    code: String,
    errors: Vec<String>,
    source_map: Option<String>,
}

#[derive(Error, Debug)]
enum CompileError {
    #[error("Parsing error: {0}")]
    ParseError(String),
    #[error("Code generation error: {0}")]
    CodegenError(String),
    #[error("Custom error: {0}")]
    CustomError(String),
    #[error("Source map generation error: {0}")]
    SourceMapError(String),
    #[error("Global configuration error: {0}")]
    GlobalError(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

async fn compile(req: CompileRequest) -> Result<impl warp::Reply, warp::Rejection> {
    let cm = SourceMap::new();
    let globals = Arc::new(Mutex::new(Globals::default()));
    
    // Combine code from files or direct code input
    let code = if let Some(code) = req.code {
        HashMap::from([("input.js".to_string(), code)])
    } else if let Some(files) = req.files {
        files
    } else {
        return Ok(warp::reply::json(&CompileResponse { code: "".into(), errors: vec!["No code or files provided".to_string()], source_map: None }));
    };

    // Concatenate all code if bundling is enabled
    let concatenated_code = if req.bundle.unwrap_or(false) {
        code.values().cloned().collect::<Vec<_>>().join("\n")
    } else {
        code.values().next().cloned().unwrap_or_default()
    };

    // Determine syntax based on request
    let syntax = match req.syntax.as_deref() {
        Some("ts") => Syntax::Typescript(TsConfig::default()),
        _ => Syntax::Es(Default::default()),
    };

    // Parse the module
    let fm = cm.new_source_file(FileName::Custom("input.js".into()), concatenated_code);
    let parser = Parser::new(syntax, TsConfig::default(), fm);
    let module = match parser.parse_module() {
        Ok(module) => module,
        Err(e) => return Ok(warp::reply::json(&CompileResponse { code: "".into(), errors: vec![format!("Parse error: {:?}", e)], source_map: None })),
    };

    let mut module = module;
    let mut resolver = Resolver::default();
    resolver.visit_mut_module(&mut module);

    // Apply React and TypeScript transformations if requested
    if req.react.unwrap_or(false) {
        let react_transform = React::default();
        react_transform.visit_mut_module(&mut module);
    }

    if req.ts.unwrap_or(false) {
        let ts_transform = TsTransform::default();
        ts_transform.visit_mut_module(&mut module);
    }

    // Apply custom transformations based on extra options
    if let Some(extra_options) = req.extra_options {
        apply_custom_transformations(&extra_options, &mut module)?;
    }

    // Handle global configurations if provided
    if let Some(globals_config) = req.globals {
        apply_globals(&globals_config, &globals)?;
    }

    // Configure the emitter for code generation
    let minify = req.minify.unwrap_or(false);
    let mut emitter = Emitter {
        cfg: swc_ecmascript::codegen::Config {
            minify,
            ..Default::default()
        },
        cm: cm.clone(),
        comments: None,
    };

    // Generate the compiled code
    let mut buf = Vec::new();
    let code = match emitter.emit_module(&module, &mut buf) {
        Ok(_) => String::from_utf8(buf).unwrap_or_default(),
        Err(e) => return Ok(warp::reply::json(&CompileResponse { code: "".into(), errors: vec![format!("Code generation error: {:?}", e)], source_map: None })),
    };

    // Generate source maps if requested
    let source_map = if req.source_map.unwrap_or(false) {
        match generate_source_map(&concatenated_code) {
            Ok(map) => Some(map),
            Err(e) => return Ok(warp::reply::json(&CompileResponse { code, errors: vec![format!("Source map generation error: {:?}", e)], source_map: None })),
        }
    } else {
        None
    };

    Ok(warp::reply::json(&CompileResponse { code, errors: vec![], source_map }))
}

fn apply_custom_transformations(extra_options: &HashMap<String, Value>, module: &mut swc_ecmascript::ast::Module) -> Result<(), CompileError> {
    for (key, value) in extra_options {
        match key.as_str() {
            "minify" => {
                if let Value::Bool(enable) = value {
                    // Configure minification if necessary
                    println!("Custom minify option: {}", enable);
                }
            },
            "custom_plugin" => {
                if let Value::String(plugin) = value {
                    // Apply custom plugin logic
                    println!("Applying custom plugin: {}", plugin);
                }
            },
            _ => return Err(CompileError::CustomError(format!("Unknown custom option: {}", key))),
        }
    }

    Ok(())
}

fn apply_globals(globals_config: &HashMap<String, String>, globals: &Arc<Mutex<Globals>>) -> Result<(), CompileError> {
    let mut globals = globals.lock().await;
    for (key, value) in globals_config {
        // Apply global configurations here
        println!("Setting global {} to {}", key, value);
        globals.set(key.clone(), value.clone());
    }

    Ok(())
}

fn generate_source_map(code: &str) -> Result<String, CompileError> {
    // Generate source map for the provided code
    // Placeholder logic for demonstration purposes
    let source_map = format!("Source map for code: {}", code);
    Ok(source_map)
}

#[tokio::main]
async fn main() {
    let compile = warp::post()
        .and(warp::path("compile"))
        .and(warp::body::json())
        .and_then(compile);

    warp::serve(compile).run(([127, 0, 0, 1], 3030)).await;
}