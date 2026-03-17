//! Rich diagnostic system inspired by cargo/rustc.
//!
//! Every pass accumulates diagnostics without stopping. The CLI renders
//! them with source context, labeled spans, and suggestions.

use codespan_reporting::diagnostic as cs;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

/// Severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Note,
}

/// A labeled source span.
#[derive(Debug, Clone)]
pub struct Label {
    pub file: String,
    pub span: std::ops::Range<usize>,
    pub message: String,
}

/// A code suggestion (fix).
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub file: String,
    pub span: std::ops::Range<usize>,
    pub replacement: String,
    pub message: String,
}

/// A single diagnostic message with rich context.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<String>,
    pub message: String,
    pub primary: Option<Label>,
    pub secondary: Vec<Label>,
    pub suggestions: Vec<Suggestion>,
    pub notes: Vec<String>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn file(&self) -> &str {
        self.primary.as_ref().map(|l| l.file.as_str()).unwrap_or("")
    }

    pub fn span(&self) -> std::ops::Range<usize> {
        self.primary.as_ref().map(|l| l.span.clone()).unwrap_or(0..0)
    }
}

/// Accumulates diagnostics across all compiler passes.
#[derive(Debug, Default)]
pub struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Simple error (backwards compatible).
    pub fn error(&mut self, file: &str, span: std::ops::Range<usize>, msg: impl Into<String>) {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: None,
            message: msg.into(),
            primary: Some(Label {
                file: file.to_string(),
                span,
                message: String::new(),
            }),
            secondary: Vec::new(),
            suggestions: Vec::new(),
            notes: Vec::new(),
            help: None,
        });
    }

    /// Simple warning (backwards compatible).
    pub fn warning(&mut self, file: &str, span: std::ops::Range<usize>, msg: impl Into<String>) {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: None,
            message: msg.into(),
            primary: Some(Label {
                file: file.to_string(),
                span,
                message: String::new(),
            }),
            secondary: Vec::new(),
            suggestions: Vec::new(),
            notes: Vec::new(),
            help: None,
        });
    }

    /// Builder for rich diagnostics.
    pub fn build(&mut self, severity: Severity, msg: impl Into<String>) -> DiagnosticBuilder<'_> {
        DiagnosticBuilder {
            diagnostics: self,
            diag: Diagnostic {
                severity,
                code: None,
                message: msg.into(),
                primary: None,
                secondary: Vec::new(),
                suggestions: Vec::new(),
                notes: Vec::new(),
                help: None,
            },
        }
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Error)
    }

    pub fn all(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Builder pattern for constructing rich diagnostics.
pub struct DiagnosticBuilder<'a> {
    diagnostics: &'a mut Diagnostics,
    diag: Diagnostic,
}

impl<'a> DiagnosticBuilder<'a> {
    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.diag.code = Some(code.into());
        self
    }

    pub fn primary(mut self, file: &str, span: std::ops::Range<usize>, label: impl Into<String>) -> Self {
        self.diag.primary = Some(Label {
            file: file.to_string(),
            span,
            message: label.into(),
        });
        self
    }

    pub fn secondary(mut self, file: &str, span: std::ops::Range<usize>, label: impl Into<String>) -> Self {
        self.diag.secondary.push(Label {
            file: file.to_string(),
            span,
            message: label.into(),
        });
        self
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.diag.notes.push(note.into());
        self
    }

    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.diag.help = Some(help.into());
        self
    }

    pub fn suggestion(mut self, file: &str, span: std::ops::Range<usize>, replacement: impl Into<String>, msg: impl Into<String>) -> Self {
        self.diag.suggestions.push(Suggestion {
            file: file.to_string(),
            span,
            replacement: replacement.into(),
            message: msg.into(),
        });
        self
    }

    pub fn emit(self) {
        self.diagnostics.diagnostics.push(self.diag);
    }
}

// ── Rendering ──────────────────────────────────────────────────────────

/// Render diagnostics to stderr with source context.
pub fn render_diagnostics(
    diagnostics: &Diagnostics,
    sources: &[(String, String)], // (filename, content)
) {
    let mut files = SimpleFiles::new();
    let mut file_ids = std::collections::HashMap::new();

    for (name, content) in sources {
        let id = files.add(name.clone(), content.clone());
        file_ids.insert(name.clone(), id);
    }

    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = term::Config::default();

    for diag in diagnostics.all() {
        let cs_diag = to_codespan(diag, &file_ids);
        let _ = term::emit_to_write_style(&mut writer.lock(), &config, &files, &cs_diag);
    }
}

/// Summary line: "error: compilation failed with N error(s) and M warning(s)"
pub fn render_summary(diagnostics: &Diagnostics) {
    let errors = diagnostics.all().iter().filter(|d| d.severity == Severity::Error).count();
    let warnings = diagnostics.all().iter().filter(|d| d.severity == Severity::Warning).count();

    if errors > 0 || warnings > 0 {
        eprint!("  ");
        if errors > 0 {
            eprint!("{} error(s)", errors);
        }
        if errors > 0 && warnings > 0 {
            eprint!(" and ");
        }
        if warnings > 0 {
            eprint!("{} warning(s)", warnings);
        }
        eprintln!(" emitted");
    }
}

fn to_codespan(
    diag: &Diagnostic,
    file_ids: &std::collections::HashMap<String, usize>,
) -> cs::Diagnostic<usize> {
    let severity = match diag.severity {
        Severity::Error => cs::Severity::Error,
        Severity::Warning => cs::Severity::Warning,
        Severity::Info | Severity::Note => cs::Severity::Note,
    };

    let mut cs_diag = cs::Diagnostic::new(severity).with_message(&diag.message);

    if let Some(ref code) = diag.code {
        cs_diag = cs_diag.with_code(code);
    }

    let mut labels = Vec::new();

    if let Some(ref primary) = diag.primary {
        if let Some(&file_id) = file_ids.get(&primary.file) {
            let span = clamp_span(&primary.span);
            let label = cs::Label::primary(file_id, span);
            let label = if primary.message.is_empty() {
                label
            } else {
                label.with_message(&primary.message)
            };
            labels.push(label);
        }
    }

    for sec in &diag.secondary {
        if let Some(&file_id) = file_ids.get(&sec.file) {
            let span = clamp_span(&sec.span);
            labels.push(
                cs::Label::secondary(file_id, span).with_message(&sec.message),
            );
        }
    }

    cs_diag = cs_diag.with_labels(labels);

    let mut notes = Vec::new();
    for note in &diag.notes {
        notes.push(format!("note: {}", note));
    }
    if let Some(ref help) = diag.help {
        notes.push(format!("help: {}", help));
    }
    for sug in &diag.suggestions {
        notes.push(format!("suggestion: {} → `{}`", sug.message, sug.replacement));
    }
    if !notes.is_empty() {
        cs_diag = cs_diag.with_notes(notes);
    }

    cs_diag
}

/// Ensure span start < end and both > 0 for codespan (it panics on empty ranges in some cases).
fn clamp_span(span: &std::ops::Range<usize>) -> std::ops::Range<usize> {
    let start = span.start;
    let end = span.end.max(start + 1);
    start..end
}
