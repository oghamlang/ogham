//! Ogham CLI — compiler, package manager, and code generation toolchain.

mod cli;
mod cmd;

use clap::Parser;

fn main() {
    let args = cli::Cli::parse();

    let result = match args.command {
        cli::Commands::Generate(args) => cmd::generate::run(args),
        cli::Commands::Check { command } => match command {
            cli::CheckCommands::Breaking(args) => cmd::check::run_breaking(args),
        },
        cli::Commands::Proto { command } => match command {
            cli::ProtoCommands::Export(args) => cmd::proto::run_export(args),
        },
        cli::Commands::Get(args) => cmd::get::run(args),
        cli::Commands::Install => cmd::install::run(),
        cli::Commands::Update => cmd::update::run(),
        cli::Commands::Vendor => cmd::vendor::run(),
        cli::Commands::Init(args) => cmd::init::run(args),
        cli::Commands::Serve(args) => cmd::serve::run(args),
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
