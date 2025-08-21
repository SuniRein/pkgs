use std::path::Path;
use std::process;
use std::fs;

use pkgs::config::Config;
use pkgs::core::NamedPackage;
use pkgs::trace::Trace;
use pkgs::core::load;
use pkgs::meta::{TOML_CONFIG_FILE, PKGS_DIR, TRACE_FILE};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::read(Path::new(TOML_CONFIG_FILE))?;

    let pkgs_dir = Path::new(PKGS_DIR);
    if !pkgs_dir.exists() {
        fs::create_dir_all(pkgs_dir)?;
    }

    let trace_file = pkgs_dir.join(TRACE_FILE);
    let mut trace = if trace_file.exists() {
        Trace::read_from_file(&trace_file)?
    } else {
        Trace::default()
    };

    let root = std::env::current_dir()?;

    for (name, package) in config.packages {
        let pkg_trace = trace.packages.get(&name);
        let named_package = NamedPackage::new(&name, package);

        match load(&root, &named_package, pkg_trace) {
            Ok(pkg_trace) => {
                println!("Loaded package: {name}");
                trace.packages.insert(name.clone(), pkg_trace);
            }
            Err(err) => {
                eprintln!("Error loading package '{name}': {err}");
                process::exit(1);
            }
        }
    }

    trace.write_to_file(&trace_file)?;

    Ok(())
}
