use std::collections::HashMap;
use std::fs::read_dir;

use anyhow::Result;
use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Defaults {
    pub interpreter: String,
    pub log_level: String,
}

#[derive(Debug, Deserialize)]
pub struct Interpreter {
    pub name: Option<String>,
    pub path: String,
    pub use_muvm: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct Binaries {
    pub path: String,
    pub interpreter: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub defaults: Defaults,
    pub interpreters: HashMap<String, Interpreter>,
    pub binaries: HashMap<String, Binaries>,
}

pub fn parse_config() -> Result<ConfigFile> {
    let mut builder = Config::builder();

    // Load main config files
    let drop_in_dir = "/usr/lib/binfmt-dispatcher.d";
    if let Ok(entries) = read_dir(drop_in_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
                builder = builder.add_source(File::from(path).required(false));
            }
        }
    }

    // Load local config from /etc
    builder = builder.add_source(File::with_name("/etc/binfmt-dispatcher.toml").required(false));

    // Load user config
    let xdg_dirs = xdg::BaseDirectories::with_prefix("binfmt-dispatcher")?;
    let xdg_config = xdg_dirs.get_config_file("binfmt-dispatcher.toml");
    builder = builder.add_source(File::from(xdg_config).required(false));

    // Build config
    let config = builder.build()?;

    let mut settings: ConfigFile = config.try_deserialize()?;

    for (key, interpreter) in settings.interpreters.iter_mut() {
        // Default to the interpreter id as name
        if interpreter.name.is_none() {
            interpreter.name = Some(key.clone());
        }
        // Default to not using muvm
        if interpreter.use_muvm.is_none() {
            interpreter.use_muvm = Some(false);
        }
    }

    Ok(settings)
}
