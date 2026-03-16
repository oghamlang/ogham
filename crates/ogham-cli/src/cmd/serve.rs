use crate::cli::ServeArgs;

pub fn run(args: ServeArgs) -> Result<(), String> {
    eprintln!("ogham serve --plugin {} --address {}", args.plugin, args.address);
    Err("not implemented yet".to_string())
}
