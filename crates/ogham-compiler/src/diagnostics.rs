//! Diagnostic accumulator for the compiler pipeline.
//!
//! Every pass receives `&mut Diagnostics` and appends errors/warnings
//! without stopping. The caller checks [`has_errors`] between phases.

/// Severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A single diagnostic message with location.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub file: String,
    pub span: std::ops::Range<usize>,
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

    pub fn error(&mut self, file: &str, span: std::ops::Range<usize>, msg: impl Into<String>) {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Error,
            message: msg.into(),
            file: file.to_string(),
            span,
        });
    }

    pub fn warning(&mut self, file: &str, span: std::ops::Range<usize>, msg: impl Into<String>) {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            message: msg.into(),
            file: file.to_string(),
            span,
        });
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
