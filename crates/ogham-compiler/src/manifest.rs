//! Parsing of ogham.mod.yaml and ogham.gen.yaml project files.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

// ── ogham.mod.yaml ─────────────────────────────────────────────────────

/// Module manifest — who the module is, dependencies, and plugin build info.
#[derive(Debug, Deserialize, Default)]
pub struct ModFile {
    pub module: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub ogham: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub require: HashMap<String, RequireEntry>,
    #[serde(default)]
    pub replace: HashMap<String, ReplaceEntry>,
    #[serde(default)]
    pub plugin: Option<PluginSection>,
    #[serde(default)]
    pub breaking: Option<BreakingSection>,
}

/// Breaking change detection configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct BreakingSection {
    /// Reference to compare against: "git:main", "git:v1.0.0", "./path/"
    pub against: String,
    /// Policy: "off", "warn", "error"
    #[serde(default = "default_breaking_policy")]
    pub policy: String,
}

fn default_breaking_policy() -> String {
    "warn".to_string()
}

/// A dependency entry — can be a version string or a detailed spec.
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum RequireEntry {
    /// Simple version range: `^1.0.0`
    Version(String),
    /// Git dependency
    Git {
        git: String,
        #[serde(default)]
        tag: Option<String>,
        #[serde(default)]
        branch: Option<String>,
        #[serde(default)]
        rev: Option<String>,
    },
    /// Local path dependency
    Path {
        path: String,
    },
}

/// Override a dependency with a local path or git fork.
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ReplaceEntry {
    Path {
        path: String,
    },
    Git {
        git: String,
        #[serde(default)]
        branch: Option<String>,
    },
}

/// Plugin build configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct PluginSection {
    pub build: String,
}

// ── ogham.gen.yaml ─────────────────────────────────────────────────────

/// Generation config — which plugins to run, where to put output.
#[derive(Debug, Deserialize, Default)]
pub struct GenFile {
    #[serde(default)]
    pub generate: GenerateSection,
}

#[derive(Debug, Deserialize, Default)]
pub struct GenerateSection {
    #[serde(default)]
    pub plugins: Vec<PluginEntry>,
}

/// A single plugin invocation.
#[derive(Debug, Deserialize, Clone)]
pub struct PluginEntry {
    /// Module path from require (e.g., `github.com/org/go`)
    #[serde(default)]
    pub name: Option<String>,
    /// External binary path
    #[serde(default)]
    pub path: Option<String>,
    /// gRPC address (overrides stdio for name plugins)
    #[serde(default)]
    pub grpc: Option<String>,
    /// Output directory for generated files
    pub out: String,
    /// Key-value options passed to the plugin
    #[serde(default)]
    pub opts: HashMap<String, String>,
}

// ── Loading ────────────────────────────────────────────────────────────

/// Load ogham.mod.yaml from a directory.
pub fn load_mod_file(dir: &Path) -> Result<ModFile, String> {
    let path = dir.join("ogham.mod.yaml");
    if !path.exists() {
        return Err(format!("ogham.mod.yaml not found in {}", dir.display()));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    serde_yaml::from_str(&content)
        .map_err(|e| format!("invalid ogham.mod.yaml: {}", e))
}

/// Load ogham.gen.yaml from a directory.
pub fn load_gen_file(dir: &Path) -> Result<GenFile, String> {
    let path = dir.join("ogham.gen.yaml");
    if !path.exists() {
        return Err(format!("ogham.gen.yaml not found in {}", dir.display()));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    serde_yaml::from_str(&content)
        .map_err(|e| format!("invalid ogham.gen.yaml: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mod_file_simple() {
        let yaml = r#"
module: github.com/myteam/myproject
version: 0.1.0
ogham: ">= 0.1.0"
description: E-commerce schema definitions
license: MIT
require:
  github.com/oghamlang/std: ^1.0.0
  github.com/org/database: ^2.0.0
"#;
        let m: ModFile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(m.module, "github.com/myteam/myproject");
        assert_eq!(m.version, "0.1.0");
        assert_eq!(m.require.len(), 2);
    }

    #[test]
    fn parse_mod_file_with_replace() {
        let yaml = r#"
module: github.com/myteam/myproject
version: 0.1.0
require:
  github.com/org/database: ^2.0.0
replace:
  github.com/org/database:
    path: ../database-fork
"#;
        let m: ModFile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(m.replace.len(), 1);
    }

    #[test]
    fn parse_mod_file_git_dep() {
        let yaml = r#"
module: github.com/myteam/myproject
version: 0.1.0
require:
  github.com/org/timestamps:
    git: https://github.com/org/timestamps.git
    tag: v1.0.0
"#;
        let m: ModFile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(m.require.len(), 1);
        match m.require.get("github.com/org/timestamps").unwrap() {
            RequireEntry::Git { git, tag, .. } => {
                assert_eq!(git, "https://github.com/org/timestamps.git");
                assert_eq!(tag.as_deref(), Some("v1.0.0"));
            }
            _ => panic!("expected git dep"),
        }
    }

    #[test]
    fn parse_mod_file_path_dep() {
        let yaml = r#"
module: github.com/myteam/myproject
version: 0.1.0
require:
  my-plugin:
    path: ../plugins/my-plugin
"#;
        let m: ModFile = serde_yaml::from_str(yaml).unwrap();
        match m.require.get("my-plugin").unwrap() {
            RequireEntry::Path { path } => assert_eq!(path, "../plugins/my-plugin"),
            _ => panic!("expected path dep"),
        }
    }

    #[test]
    fn parse_mod_file_plugin() {
        let yaml = r#"
module: github.com/org/database
version: 2.0.0
plugin:
  build: go build -o ogham-gen-database ./cmd
"#;
        let m: ModFile = serde_yaml::from_str(yaml).unwrap();
        assert!(m.plugin.is_some());
        assert_eq!(m.plugin.unwrap().build, "go build -o ogham-gen-database ./cmd");
    }

    #[test]
    fn parse_gen_file() {
        let yaml = r#"
generate:
  plugins:
    - name: github.com/org/database
      out: internal/db/gen/
      opts:
        orm: sqlx
    - name: github.com/org/go
      out: internal/models/
    - path: ./tools/my-custom-plugin
      out: gen/
"#;
        let g: GenFile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(g.generate.plugins.len(), 3);
        assert_eq!(g.generate.plugins[0].name.as_deref(), Some("github.com/org/database"));
        assert_eq!(g.generate.plugins[0].out, "internal/db/gen/");
        assert_eq!(g.generate.plugins[0].opts.get("orm").unwrap(), "sqlx");
        assert_eq!(g.generate.plugins[2].path.as_deref(), Some("./tools/my-custom-plugin"));
    }

    #[test]
    fn parse_gen_file_with_grpc() {
        let yaml = r#"
generate:
  plugins:
    - name: github.com/org/go-pgx
      grpc: localhost:50051
      out: internal/db/gen/
"#;
        let g: GenFile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(g.generate.plugins[0].grpc.as_deref(), Some("localhost:50051"));
    }

    #[test]
    fn parse_mod_file_with_breaking() {
        let yaml = r#"
module: github.com/myteam/myproject
version: 0.1.0
breaking:
  against: git:main
  policy: error
"#;
        let m: ModFile = serde_yaml::from_str(yaml).unwrap();
        let b = m.breaking.unwrap();
        assert_eq!(b.against, "git:main");
        assert_eq!(b.policy, "error");
    }

    #[test]
    fn parse_mod_file_breaking_default_policy() {
        let yaml = r#"
module: github.com/myteam/myproject
version: 0.1.0
breaking:
  against: git:v1.0.0
"#;
        let m: ModFile = serde_yaml::from_str(yaml).unwrap();
        let b = m.breaking.unwrap();
        assert_eq!(b.against, "git:v1.0.0");
        assert_eq!(b.policy, "warn");
    }
}
