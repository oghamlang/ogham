//! Plugin runner — reads OghamCompileRequest from stdin, calls the user
//! function, writes OghamCompileResponse to stdout.
//!
//! This is the entry point for all Ogham plugins. The plugin binary
//! is invoked by the `ogham generate` command with protobuf-encoded
//! request on stdin and expects protobuf-encoded response on stdout.

use crate::{CompileRequest, CompileResponse, CompileError, Severity};
use prost::Message;
use std::io::{self, Read, Write};

/// Run a plugin handler. Reads `CompileRequest` from stdin,
/// calls `handler`, writes `CompileResponse` to stdout.
///
/// If the handler returns an error, it's wrapped in a `CompileError`
/// and sent back as part of the response (not as a process exit code).
///
/// # Example
///
/// ```rust,no_run
/// oghamgen::run(|req| {
///     let mut resp = oghamgen::CompileResponse::default();
///     // ... generate code ...
///     Ok(resp)
/// });
/// ```
pub fn run<F>(handler: F)
where
    F: FnOnce(CompileRequest) -> Result<CompileResponse, String>,
{
    let result = run_inner(handler);

    match result {
        Ok(response) => {
            let mut buf = Vec::new();
            if let Err(e) = response.encode(&mut buf) {
                eprintln!("oghamgen: failed to encode response: {}", e);
                std::process::exit(1);
            }
            if let Err(e) = io::stdout().write_all(&buf) {
                eprintln!("oghamgen: failed to write response: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            // Fatal error — still try to send a response with the error
            let response = CompileResponse {
                files: Vec::new(),
                errors: vec![CompileError {
                    message: e,
                    severity: Severity::Error as i32,
                    source_type: String::new(),
                    source_field: String::new(),
                }],
            };
            let mut buf = Vec::new();
            if response.encode(&mut buf).is_ok() {
                let _ = io::stdout().write_all(&buf);
            }
            std::process::exit(1);
        }
    }
}

fn run_inner<F>(handler: F) -> Result<CompileResponse, String>
where
    F: FnOnce(CompileRequest) -> Result<CompileResponse, String>,
{
    // Read all of stdin
    let mut input = Vec::new();
    io::stdin()
        .read_to_end(&mut input)
        .map_err(|e| format!("failed to read stdin: {}", e))?;

    // Decode request
    let request = CompileRequest::decode(input.as_slice())
        .map_err(|e| format!("failed to decode request: {}", e))?;

    // Call handler
    handler(request)
}
