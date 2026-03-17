//! `ogham generate` — compile .ogham files and run plugins.

use crate::cli::GenerateArgs;
use ogham_compiler::ast::AstNode;
use ogham_compiler::lower;
use ogham_compiler::manifest;
use ogham_compiler::pipeline::{self, SourceFile};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn run(args: GenerateArgs) -> Result<(), String> {
    let dir = &args.dir;

    // Try to load ogham.mod.yaml (optional — we can compile without it)
    let mod_file = manifest::load_mod_file(dir).ok();
    if let Some(ref m) = mod_file {
        eprintln!("module: {} v{}", m.module, m.version);
    }

    // Discover .ogham files — look in schemas/ subdir or project root
    let schemas_dir = dir.join("schemas");
    let search_dir = if schemas_dir.is_dir() { &schemas_dir } else { dir };

    let sources = discover_ogham_files(search_dir)?;
    if sources.is_empty() {
        return Err(format!("no .ogham files found in {}", search_dir.display()));
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
    let request_bytes = serialize_request(&module, &args)?;

    eprintln!("compiled successfully ({} bytes IR)", request_bytes.len());

    // Determine which plugins to run
    let plugins = resolve_plugins(dir, &args)?;

    if plugins.is_empty() {
        eprintln!("no plugins configured (use --plugin <name> or create ogham.gen.yaml)");
        return Ok(());
    }

    // Run each plugin in order
    for plugin in &plugins {
        let out_dir = dir.join(&plugin.out);

        // Encode request with plugin-specific output dir and options
        let module_clone = lower::inflate(&result.interner, &result.arenas, &result.symbols, &package);
        let req = lower::build_request(
            module_clone,
            env!("CARGO_PKG_VERSION"),
            plugin.opts.clone(),
            &out_dir.to_string_lossy(),
        );
        let request_with_out = {
            use prost::Message;
            let mut buf = Vec::new();
            req.encode(&mut buf).map_err(|e| format!("encode error: {}", e))?;
            buf
        };

        let plugin_name = plugin.name.as_deref()
            .or(plugin.path.as_deref())
            .unwrap_or("unknown");

        run_plugin(plugin_name, plugin.path.as_deref(), &request_with_out)?;
    }

    Ok(())
}

fn serialize_request(
    module: &ogham_proto::ogham::ir::Module,
    _args: &GenerateArgs,
) -> Result<Vec<u8>, String> {
    use prost::Message;
    let request = lower::build_request(
        module.clone(),
        env!("CARGO_PKG_VERSION"),
        Default::default(),
        ".",
    );
    let mut buf = Vec::new();
    request.encode(&mut buf).map_err(|e| format!("failed to encode request: {}", e))?;
    Ok(buf)
}

struct ResolvedPlugin {
    name: Option<String>,
    path: Option<String>,
    out: String,
    opts: std::collections::HashMap<String, String>,
}

fn resolve_plugins(dir: &Path, args: &GenerateArgs) -> Result<Vec<ResolvedPlugin>, String> {
    // --plugin flag takes priority
    if let Some(ref plugin_name) = args.plugin {
        return Ok(vec![ResolvedPlugin {
            name: Some(plugin_name.clone()),
            path: None,
            out: ".".to_string(),
            opts: Default::default(),
        }]);
    }

    // Try ogham.gen.yaml
    match manifest::load_gen_file(dir) {
        Ok(gen) => {
            Ok(gen.generate.plugins.into_iter().map(|p| ResolvedPlugin {
                name: p.name,
                path: p.path,
                out: p.out,
                opts: p.opts,
            }).collect())
        }
        Err(_) => Ok(Vec::new()),
    }
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
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with('.') || name == "vendor" || name == "node_modules" || name == "target" || name == "gen" {
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

fn run_plugin(name: &str, explicit_path: Option<&str>, request_bytes: &[u8]) -> Result<(), String> {
    let bin_name = if let Some(path) = explicit_path {
        path.to_string()
    } else {
        // Extract last segment: github.com/org/ogham-gen-go → ogham-gen-go
        let short = name.rsplit('/').next().unwrap_or(name);
        if short.starts_with("ogham-gen-") {
            short.to_string()
        } else {
            format!("ogham-gen-{}", short)
        }
    };

    eprintln!("running plugin: {}", bin_name);

    let mut child = Command::new(&bin_name)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("failed to start {}: {} (is it installed?)", bin_name, e))?;

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

    use prost::Message;
    let response = ogham_proto::ogham::compiler::OghamCompileResponse::decode(output.stdout.as_slice())
        .map_err(|e| format!("failed to decode plugin response: {}", e))?;

    for err in &response.errors {
        eprintln!("plugin {}: {}: {}", bin_name, err.severity, err.message);
    }

    for file in &response.files {
        let path = Path::new(&file.name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create dir {}: {}", parent.display(), e))?;
        }

        if file.append {
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
