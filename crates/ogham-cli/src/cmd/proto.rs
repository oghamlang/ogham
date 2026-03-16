use crate::cli::ProtoExportArgs;

pub fn run_export(args: ProtoExportArgs) -> Result<(), String> {
    eprintln!("ogham proto export {}", args.output_dir.display());
    Err("not implemented yet".to_string())
}
