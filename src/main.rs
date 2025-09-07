use std::io::Stdout;

use anyhow::Result;
use clap::Parser;

use pkgs::cli::{Cli, Command};
use pkgs::config::Config;
use pkgs::logger::WriterOutput;
use pkgs::meta::TRACE_FILE;
use pkgs::trace::Trace;

type Runner = pkgs::runner::Runner<WriterOutput<Stdout>>;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let cwd = std::env::current_dir()?;
    let stdout = WriterOutput::new(std::io::stdout());
    let runner = Runner::new(&cwd, stdout);

    let config = runner.read_config()?;
    let available = config.packages.keys();

    match &cli.command {
        Command::Load { modules } => load(&config, modules.get(available)?, runner),
        Command::Unload { modules } => unload(modules.get(available)?, runner),
        Command::List { oneline } => {
            println!(
                "{}",
                available
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(if *oneline { "\n" } else { " " })
            );
            Ok(())
        }
    }
}

fn load(config: &Config, modules: Vec<String>, mut runner: Runner) -> Result<()> {
    let pkgs_dir = runner.create_pkgs_dir()?;

    let trace_file = pkgs_dir.join(TRACE_FILE);
    let mut trace = if trace_file.exists() {
        Trace::read_from_file(&trace_file)?
    } else {
        Trace::default()
    };

    for name in modules {
        let pkg_trace = trace.packages.get(&name);
        let package = config.get(&name)?;

        match runner.load_module(&package, pkg_trace) {
            Ok(pkg_trace) => {
                println!("Loaded package: {name}");
                trace.packages.insert(name.clone(), pkg_trace);
            }
            Err(e) => {
                eprintln!("{e}");
                runner.rollback()?;
            }
        }
    }

    trace.write_to_file(&trace_file)?;

    Ok(())
}

fn unload(modules: Vec<String>, mut runner: Runner) -> Result<()> {
    let pkgs_dir = runner.get_pkgs_dir()?;

    let trace_file = pkgs_dir.join(TRACE_FILE);
    let mut trace = if trace_file.exists() {
        Trace::read_from_file(&trace_file)?
    } else {
        Trace::default()
    };

    for name in modules {
        let Some(pkg_trace) = trace.packages.get(&name) else {
            eprintln!("Warning! Package '{name}' is not loaded.");
            continue;
        };

        match runner.unload_module(&name, pkg_trace) {
            Ok(()) => {
                println!("Unloaded package: {name}");
                trace.packages.remove(&name);
            }
            Err(e) => {
                eprintln!("{e}");
                runner.rollback()?;
            }
        }
    }

    trace.write_to_file(&trace_file)?;

    Ok(())
}
