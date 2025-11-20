use std::{env, fs};

use anyhow::{anyhow, Context, Result};
use researchbase::bases::{
    config_file_path, ensure_workspace_structure, load_or_default, save, CompilerBinary,
};

fn main() -> Result<()> {
    ensure_workspace_structure()?;
    let args = CliArgs::parse()?;
    let config_path = config_file_path()?;
    let mut config = load_or_default()?;
    let mut changed = config_missing_writing_section(&config_path);

    changed |= apply_compiler_override(
        &mut config.writing.primary_compiler,
        args.primary.as_deref(),
        "tectonic",
    );
    changed |= apply_compiler_override(
        &mut config.writing.fallback_compiler,
        args.fallback.as_deref(),
        "pdflatex",
    );

    if changed {
        save(&config)?;
        println!(
            "Writing compiler settings recorded at {}",
            config_path.display()
        );
    } else {
        println!("Writing compiler settings already configured.");
    }

    Ok(())
}

struct CliArgs {
    primary: Option<String>,
    fallback: Option<String>,
}

impl CliArgs {
    fn parse() -> Result<Self> {
        let mut args = env::args().skip(1);
        let mut primary = None;
        let mut fallback = None;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--tectonic" | "--primary" => {
                    let value = args
                        .next()
                        .context("Expected a compiler command after --tectonic/--primary")?;
                    primary = Some(value);
                }
                "--fallback" => {
                    let value = args
                        .next()
                        .context("Expected a compiler command after --fallback")?;
                    fallback = Some(value);
                }
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                other => {
                    return Err(anyhow!(
                        "Unknown argument '{other}'. Run with --help for usage instructions."
                    ));
                }
            }
        }
        Ok(Self { primary, fallback })
    }
}

fn print_usage() {
    println!("ResearchBase setup (writing assistant)");
    println!("Ensures compiler commands are recorded in config.toml.");
    println!("Usage: cargo run --bin setup -- [options]");
    println!("Options:");
    println!("  --tectonic <command>   Override the primary compiler command (default: tectonic)");
    println!("  --fallback <command>   Override the fallback compiler command (default: pdflatex)");
}

fn apply_compiler_override(
    slot: &mut CompilerBinary,
    override_value: Option<&str>,
    default_command: &str,
) -> bool {
    if let Some(command) = override_value {
        if slot.command != command {
            slot.command = command.to_string();
            slot.args.clear();
            return true;
        }
        return false;
    }

    if slot.command.is_empty() {
        slot.command = default_command.to_string();
        slot.args.clear();
        return true;
    }

    false
}

fn config_missing_writing_section(path: &std::path::Path) -> bool {
    if !path.exists() {
        return true;
    }
    match fs::read_to_string(path) {
        Ok(contents) => !contents.contains("writing"),
        Err(_) => true,
    }
}
