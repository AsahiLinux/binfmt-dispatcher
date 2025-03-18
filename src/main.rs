// SPDX-License-Identifier: MIT

mod config;
use crate::config::ConfigFile;

mod util;
use crate::util::get_page_size;
use crate::util::{error_dialog, info_dialog};

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{canonicalize, read_link};
use std::io::{stdin, IsTerminal};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

fn abort(msg: &str) {
    error!("{}", msg);
    if !stdin().is_terminal() {
        error_dialog(msg);
    }
    exit(1)
}

fn main() {
    // Parse config
    let settings: ConfigFile = config::parse_config().unwrap();

    // Setup logging
    let logger_env = env_logger::Env::default()
        .filter_or("BINFMT_DISPATCHER_LOG_LEVEL", &settings.defaults.log_level)
        .write_style_or("BINFMT_DISPATCHER_LOG_STYLE", "auto");

    env_logger::Builder::from_env(logger_env)
        .format_level(false)
        .format_timestamp(None)
        .init();
    trace!("Configuration:\n{:#?}", settings);

    // Collect command line arguments to pass through
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    trace!("Args:\n{:#?}", args);
    if args.is_empty() {
        abort("No arguments passed, re-run with --help to learn more.");
    } else {
        match args[0].to_str().unwrap() {
            "--help" => {
                println!("This program is meant to be run as a binfmt_misc handler and has no interactive options. See {} for more information.", env!("CARGO_PKG_HOMEPAGE"));
                exit(0);
            }
            "--version" => {
                println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
                exit(1);
            }
            _ => trace!("Assuming arguments are coming from binfmt_misc"),
        }
    }

    // File descriptor 3 is where binfmt_misc typically passes the executable
    let binary: PathBuf = read_link("/proc/self/fd/3").unwrap_or_else(|e| {
        error!("Failed to read the executable from fd#3: {}", e);
        exit(1);
    });
    trace!("Binary: {:#?}", binary);

    let mut interpreter_id = &settings.defaults.interpreter;
    for binary in settings.binaries.values() {
        if Path::new(&binary.path) == canonicalize(&args[0]).unwrap() {
            interpreter_id = &binary.interpreter;
            break;
        }
    }

    let interpreter = settings.interpreters.get(interpreter_id).unwrap();
    let interpreter_name = interpreter.name.as_ref().unwrap();
    let interpreter_path = &interpreter.path;
    let mut interpreter_missing_paths = vec![];
    for path in interpreter.required_paths.as_ref().unwrap() {
        let p = Path::new(OsStr::new(path));
        if !p.exists() {
            interpreter_missing_paths.push(p);
        }
    }
    if !interpreter_missing_paths.is_empty() {
        warn!(
            "Will attempt to install missing requirements for {}",
            interpreter_name
        );
        if !stdin().is_terminal() {
            info_dialog(&format!(
                "To run this program, {} needs to be installed.",
                interpreter_name
            ));
        }

        let mut dnf_command;
        if stdin().is_terminal() {
            trace!("Running in a terminal");
            dnf_command = Command::new("pkexec");
        } else {
            trace!("Not running in a terminal");
            dnf_command = Command::new("xdg-terminal-exec");
            dnf_command.arg("--");
            dnf_command.arg("pkexec");
        }

        dnf_command.arg("/usr/bin/dnf");
        dnf_command.arg("install");
        dnf_command.args(&interpreter_missing_paths);
        debug!("Running:\n{:#?}", dnf_command);
        match dnf_command.spawn() {
            Ok(mut child) => {
                let status = child.wait().expect("Failed to wait on dnf process");
                if !status.success() {
                    debug!(
                        "Failed to install missing requirements: dnf returned {:?}",
                        status
                    );
                    abort("The installation failed. Please try again.");
                }
            }
            Err(e) => {
                debug!("Failed to execute dnf: {}", e);
                abort("The package manager failed to start. Please try again.");
            }
        }
    }

    let mut use_muvm = interpreter.use_muvm.unwrap();
    if use_muvm {
        if let Some(size) = get_page_size() {
            debug!("Page size: {} bytes", size);
            // Use muvm if the page-size is not 4k
            use_muvm = size != 4096;
        } else {
            debug!("Failed to get page size");
            if !stdin().is_terminal() {
                error_dialog("We were unable to detect the system page size. The program might not execute correctly.");
            }
            use_muvm = false;
        }
    }

    let mut command;
    if use_muvm {
        info!("Using {} with muvm", interpreter_name);
        command = Command::new("/usr/bin/muvm");
        command.arg("--");
        command.arg(interpreter_path);
    } else {
        info!("Using {}", interpreter_name);
        command = Command::new(interpreter_path);
    }

    command.arg(binary);

    // Pass through all the arguments
    command.args(&args);

    // Execute the command and replace the current process
    // Use a panic instead of expecting a return value
    debug!("Running:\n{:#?}", command);
    let _ = command.exec();

    // If exec fails, it will not return; however, we include this to handle the case.
    abort("The program failed to execute.");
}
