use std::path::Path;
use std::process;

use pkgs::config::Config;
use pkgs::meta::TOML_CONFIG_FILE;

fn main() {
    let config = Config::read(Path::new(TOML_CONFIG_FILE)).unwrap_or_else(|err| {
        eprintln!("Error reading config: {err}");
        process::exit(1);
    });

    println!("Loaded config with {} packages", config.packages.len());
}
