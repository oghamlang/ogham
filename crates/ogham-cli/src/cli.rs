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

    /// Check schemas for issues
    Check {
        #[command(subcommand)]
        command: CheckCommands,
    },

    /// Protocol buffer operations
    Proto {
        #[command(subcommand)]
        command: ProtoCommands,
    },

    /// Add a dependency to ogham.mod.yaml
    Get(GetArgs),

    /// Install all dependencies for the current project
    Install,

    /// Update dependency versions
    Update,

    /// Copy dependencies into vendor/
    Vendor,

    /// Initialize a new Ogham project or plugin
    Init(InitArgs),

    /// Serve a plugin as gRPC server
    Serve(ServeArgs),
}

#[derive(Subcommand)]
pub enum CheckCommands {
    /// Check for breaking changes against a reference
    Breaking(BreakingArgs),
}

#[derive(Subcommand)]
pub enum ProtoCommands {
    /// Export .proto files from .ogham schemas
    Export(ProtoExportArgs),
}

// ── Command args ───────────────────────────────────────────────────────

#[derive(Args)]
pub struct GenerateArgs {
    /// Run only a specific plugin
    #[arg(long)]
    pub plugin: Option<String>,

    /// Project root directory (default: current directory)
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,
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
pub struct ProtoExportArgs {
    /// Output directory for .proto files
    pub output_dir: PathBuf,

    /// Project root directory
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,
}

#[derive(Args)]
pub struct GetArgs {
    /// Dependency to add (e.g., github.com/org/database or github.com/org/database@2.1.0)
    pub dependency: String,
}

#[derive(Args)]
pub struct InitArgs {
    /// Initialize as a plugin project
    #[arg(long)]
    pub plugin: Option<String>,

    /// Project directory (default: current directory)
    #[arg(default_value = ".")]
    pub dir: PathBuf,
}

#[derive(Args)]
pub struct ServeArgs {
    /// Plugin to serve
    #[arg(long)]
    pub plugin: String,

    /// Address to listen on
    #[arg(long, default_value = ":50051")]
    pub address: String,
}
