use crate::cli::BreakingArgs;

pub fn run_breaking(args: BreakingArgs) -> Result<(), String> {
    eprintln!(
        "ogham check breaking --against {} (allow={}, force={})",
        args.against, args.allow, args.force
    );
    Err("not implemented yet".to_string())
}
