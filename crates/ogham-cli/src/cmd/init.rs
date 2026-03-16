use crate::cli::InitArgs;

pub fn run(args: InitArgs) -> Result<(), String> {
    if let Some(plugin) = &args.plugin {
        eprintln!("ogham init --plugin {} in {}", plugin, args.dir.display());
    } else {
        eprintln!("ogham init in {}", args.dir.display());
    }
    Err("not implemented yet".to_string())
}
