//! CLI argument definitions using clap derive.

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "ogham",
    version,
    about = "Ogham schema language compiler and toolchain"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate code from Ogham schemas using plugins from ogham.gen.yaml
    Generate(GenerateArgs),

    /// Validate schemas — compile without running plugins
    Check(CheckArgs),

    /// Detect breaking changes against a reference
    Breaking(BreakingArgs),

    /// Dump compiled IR as JSON (debug)
    Dump(DumpArgs),

    /// Add a dependency to ogham.mod.yaml
    Get(GetArgs),

    /// Install all dependencies for the current project
    Install,

    /// Update dependency versions
    Update,

    /// Copy dependencies into vendor/
    Vendor,

}

// ── Command args ───────────────────────────────────────────────────────

#[derive(Args)]
pub struct CheckArgs {
    /// Project root directory
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,
}

#[derive(Args)]
pub struct GenerateArgs {
    /// Run only a specific plugin
    #[arg(long)]
    pub plugin: Option<String>,

    /// Project root directory (default: current directory)
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,

    /// Skip breaking change check even if configured in ogham.mod.yaml
    #[arg(long)]
    pub skip_breaking: bool,
}

#[derive(Args)]
pub struct BreakingArgs {
    /// Reference to compare against (git:main, git:v1.0.0, path, or module@version)
    #[arg(long)]
    pub against: String,

    /// Only ERROR blocks; WARNING and INFO are logged but don't block
    #[arg(long, conflicts_with = "force")]
    pub allow: bool,

    /// Nothing blocks; everything is logged
    #[arg(long, conflicts_with = "allow")]
    pub force: bool,
}

#[derive(Args)]
pub struct DumpArgs {
    /// Project root directory
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,

    /// Output file (default: stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

#[derive(Args)]
pub struct GetArgs {
    /// Dependency to add (e.g., github.com/org/database or github.com/org/database@2.1.0)
    pub dependency: String,
}

