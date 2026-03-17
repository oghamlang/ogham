//! Code generation utilities — string builders, indent management,
//! import tracking, and file emission helpers.

use crate::GeneratedFile;

/// A code writer with automatic indentation.
///
/// # Example
///
/// ```
/// use oghamgen::CodeWriter;
///
/// let mut w = CodeWriter::new();
/// w.line("package main");
/// w.newline();
/// w.line("func main() {");
/// w.indent();
/// w.line("fmt.Println(\"hello\")");
/// w.dedent();
/// w.line("}");
///
/// let file = w.to_file("main.go");
/// assert!(file.name == "main.go");
/// ```
pub struct CodeWriter {
    buf: String,
    indent_level: usize,
    indent_str: String,
    imports: Vec<String>,
}

impl CodeWriter {
    /// Create a new writer with tab indentation.
    pub fn new() -> Self {
        Self {
            buf: String::new(),
            indent_level: 0,
            indent_str: "\t".to_string(),
            imports: Vec::new(),
        }
    }

    /// Create a new writer with custom indentation (e.g., 4 spaces).
    pub fn with_indent(indent: &str) -> Self {
        Self {
            buf: String::new(),
            indent_level: 0,
            indent_str: indent.to_string(),
            imports: Vec::new(),
        }
    }

    /// Write a line with current indentation.
    pub fn line(&mut self, text: &str) {
        for _ in 0..self.indent_level {
            self.buf.push_str(&self.indent_str);
        }
        self.buf.push_str(text);
        self.buf.push('\n');
    }

    /// Write a line without indentation.
    pub fn raw(&mut self, text: &str) {
        self.buf.push_str(text);
        self.buf.push('\n');
    }

    /// Write text without newline.
    pub fn write(&mut self, text: &str) {
        self.buf.push_str(text);
    }

    /// Write an empty line.
    pub fn newline(&mut self) {
        self.buf.push('\n');
    }

    /// Increase indentation level.
    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decrease indentation level.
    pub fn dedent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
    }

    /// Open a block: write line and indent.
    /// E.g., `w.open("func main() {")` → writes line + indents.
    pub fn open(&mut self, text: &str) {
        self.line(text);
        self.indent();
    }

    /// Close a block: dedent and write line.
    /// E.g., `w.close("}")` → dedents + writes line.
    pub fn close(&mut self, text: &str) {
        self.dedent();
        self.line(text);
    }

    /// Add a comment line.
    pub fn comment(&mut self, prefix: &str, text: &str) {
        self.line(&format!("{} {}", prefix, text));
    }

    /// Track an import path (for languages that need import blocks).
    pub fn add_import(&mut self, path: &str) {
        if !self.imports.contains(&path.to_string()) {
            self.imports.push(path.to_string());
        }
    }

    /// Get tracked imports.
    pub fn imports(&self) -> &[String] {
        &self.imports
    }

    /// Check if an import was added.
    pub fn has_import(&self, path: &str) -> bool {
        self.imports.iter().any(|i| i == path)
    }

    /// Get the generated code as a string.
    pub fn finish(&self) -> String {
        self.buf.clone()
    }

    /// Get the generated code as bytes.
    pub fn finish_bytes(&self) -> Vec<u8> {
        self.buf.as_bytes().to_vec()
    }

    /// Convert to a GeneratedFile.
    pub fn to_file(&self, name: &str) -> GeneratedFile {
        GeneratedFile {
            name: name.to_string(),
            content: self.finish_bytes(),
            append: false,
        }
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Current indentation level.
    pub fn level(&self) -> usize {
        self.indent_level
    }
}

impl Default for CodeWriter {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helper functions for common patterns ───────────────────────────────

/// Convert a name to PascalCase.
pub fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// Convert a name to snake_case.
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.extend(c.to_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert a name to camelCase.
pub fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    let mut chars = pascal.chars();
    match chars.next() {
        Some(c) => c.to_lowercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

/// Convert a name to SCREAMING_SNAKE_CASE.
pub fn to_screaming_snake_case(s: &str) -> String {
    to_snake_case(s).to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writer_basic() {
        let mut w = CodeWriter::new();
        w.line("hello");
        w.line("world");
        assert_eq!(w.finish(), "hello\nworld\n");
    }

    #[test]
    fn writer_indent() {
        let mut w = CodeWriter::with_indent("  ");
        w.line("func main() {");
        w.indent();
        w.line("fmt.Println(\"hello\")");
        w.dedent();
        w.line("}");
        assert_eq!(
            w.finish(),
            "func main() {\n  fmt.Println(\"hello\")\n}\n"
        );
    }

    #[test]
    fn writer_open_close() {
        let mut w = CodeWriter::with_indent("    ");
        w.open("if true {");
        w.line("do_thing()");
        w.close("}");
        assert_eq!(w.finish(), "if true {\n    do_thing()\n}\n");
    }

    #[test]
    fn writer_imports() {
        let mut w = CodeWriter::new();
        w.add_import("fmt");
        w.add_import("os");
        w.add_import("fmt"); // duplicate
        assert_eq!(w.imports().len(), 2);
        assert!(w.has_import("fmt"));
    }

    #[test]
    fn writer_to_file() {
        let mut w = CodeWriter::new();
        w.line("hello");
        let file = w.to_file("test.go");
        assert_eq!(file.name, "test.go");
        assert_eq!(file.content, b"hello\n");
        assert!(!file.append);
    }

    #[test]
    fn pascal_case() {
        assert_eq!(to_pascal_case("user_name"), "UserName");
        assert_eq!(to_pascal_case("id"), "Id");
        assert_eq!(to_pascal_case("created_at"), "CreatedAt");
    }

    #[test]
    fn snake_case() {
        assert_eq!(to_snake_case("UserName"), "user_name");
        assert_eq!(to_snake_case("ID"), "i_d");
        assert_eq!(to_snake_case("createdAt"), "created_at");
    }

    #[test]
    fn camel_case() {
        assert_eq!(to_camel_case("user_name"), "userName");
        assert_eq!(to_camel_case("created_at"), "createdAt");
    }

    #[test]
    fn screaming_snake() {
        assert_eq!(to_screaming_snake_case("OrderStatus"), "ORDER_STATUS");
        assert_eq!(to_screaming_snake_case("active"), "ACTIVE");
    }
}
