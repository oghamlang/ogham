//! Ogham CLI — compiler, package manager, and code generation toolchain.

mod cli;
mod cmd;

use clap::Parser;

fn main() {
    let args = cli::Cli::parse();

    let result = match args.command {
        cli::Commands::Generate(args) => cmd::generate::run(args),
        cli::Commands::Check(args) => cmd::check::run(args),
        cli::Commands::Breaking(args) => cmd::breaking::run(args),
        cli::Commands::Dump(args) => cmd::dump::run(args),
        cli::Commands::Get(args) => cmd::get::run(args),
        cli::Commands::Install => cmd::install::run(),
        cli::Commands::Update => cmd::update::run(),
        cli::Commands::Vendor => cmd::vendor::run(),
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
