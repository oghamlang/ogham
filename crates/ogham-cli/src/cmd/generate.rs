//! `ogham generate` — compile .ogham files and run plugins.

use crate::cli::GenerateArgs;
use ogham_compiler::ast::AstNode;
use ogham_compiler::lower;
use ogham_compiler::pipeline::{self, SourceFile};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn run(args: GenerateArgs) -> Result<(), String> {
    let dir = &args.dir;

    // Discover .ogham files
    let sources = discover_ogham_files(dir)?;
    if sources.is_empty() {
        return Err(format!("no .ogham files found in {}", dir.display()));
    }

    eprintln!("compiling {} file(s)...", sources.len());

    // Compile
    let result = pipeline::compile(&sources);

    // Report diagnostics with source context
    let source_pairs: Vec<(String, String)> = sources
        .iter()
        .map(|s| (s.name.clone(), s.content.clone()))
        .collect();
    ogham_compiler::diagnostics::render_diagnostics(&result.diagnostics, &source_pairs);
    ogham_compiler::diagnostics::render_summary(&result.diagnostics);

    if result.diagnostics.has_errors() {
        return Err("compilation failed".to_string());
    }

    // Determine package name
    let package = sources
        .first()
        .and_then(|s| {
            let parse = ogham_compiler::parser::parse(&s.content);
            let root = ogham_compiler::ast::Root::cast(parse.syntax())?;
            root.package_decl()
                .and_then(|p| p.name())
                .map(|t| t.text().to_string())
        })
        .unwrap_or_else(|| "default".to_string());

    // Lower to IR
    let module = lower::inflate(&result.interner, &result.arenas, &result.symbols, &package);
    let request = lower::build_request(module, env!("CARGO_PKG_VERSION"), Default::default(), ".");

    // Serialize request
    let request_bytes = {
        use prost::Message;
        let mut buf = Vec::new();
        request.encode(&mut buf).map_err(|e| format!("failed to encode request: {}", e))?;
        buf
    };

    eprintln!("compiled successfully ({} bytes IR)", request_bytes.len());

    // TODO: Read ogham.gen.yaml and run configured plugins
    // For now, if --plugin is specified, try to run it
    if let Some(plugin_name) = &args.plugin {
        run_plugin(plugin_name, &request_bytes)?;
    } else {
        eprintln!("no plugins configured (use --plugin <name> or create ogham.gen.yaml)");
    }

    Ok(())
}

fn discover_ogham_files(dir: &Path) -> Result<Vec<SourceFile>, String> {
    let mut sources = Vec::new();
    discover_recursive(dir, &mut sources)?;
    sources.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(sources)
}

fn discover_recursive(dir: &Path, sources: &mut Vec<SourceFile>) -> Result<(), String> {
    if !dir.is_dir() {
        return Err(format!("{} is not a directory", dir.display()));
    }

    let entries = std::fs::read_dir(dir).map_err(|e| format!("cannot read {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("read error: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            // Skip hidden dirs, vendor, node_modules, target
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with('.') || name == "vendor" || name == "node_modules" || name == "target" {
                continue;
            }
            discover_recursive(&path, sources)?;
        } else if path.extension().is_some_and(|ext| ext == "ogham") {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
            sources.push(SourceFile {
                name: path.to_string_lossy().to_string(),
                content,
            });
        }
    }

    Ok(())
}

fn run_plugin(plugin: &str, request_bytes: &[u8]) -> Result<(), String> {
    // Try to find plugin binary: ogham-gen-<name> in PATH or $OGHAM_BIN
    let bin_name = if plugin.starts_with("ogham-gen-") {
        plugin.to_string()
    } else {
        format!("ogham-gen-{}", plugin)
    };

    eprintln!("running plugin: {}", bin_name);

    let mut child = Command::new(&bin_name)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("failed to start {}: {}", bin_name, e))?;

    // Send request via stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(request_bytes)
            .map_err(|e| format!("failed to write to plugin stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("failed to wait for plugin: {}", e))?;

    if !output.status.success() {
        return Err(format!("plugin {} exited with {}", bin_name, output.status));
    }

    // Decode response
    use prost::Message;
    let response = ogham_proto::ogham::compiler::OghamCompileResponse::decode(output.stdout.as_slice())
        .map_err(|e| format!("failed to decode plugin response: {}", e))?;

    // Report plugin errors
    for err in &response.errors {
        eprintln!("plugin {}: {}: {}", bin_name, err.severity, err.message);
    }

    // Write generated files
    for file in &response.files {
        let path = Path::new(&file.name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create dir {}: {}", parent.display(), e))?;
        }

        if file.append {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(path)
                .map_err(|e| format!("cannot open {}: {}", path.display(), e))?;
            f.write_all(&file.content)
                .map_err(|e| format!("cannot write {}: {}", path.display(), e))?;
        } else {
            std::fs::write(path, &file.content)
                .map_err(|e| format!("cannot write {}: {}", path.display(), e))?;
        }

        eprintln!("  wrote {}", file.name);
    }

    eprintln!("plugin {} generated {} file(s)", bin_name, response.files.len());
    Ok(())
}
