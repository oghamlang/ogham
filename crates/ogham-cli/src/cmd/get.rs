use crate::cli::GetArgs;

pub fn run(args: GetArgs) -> Result<(), String> {
    eprintln!("ogham get {}", args.dependency);
    Err("not implemented yet".to_string())
}
