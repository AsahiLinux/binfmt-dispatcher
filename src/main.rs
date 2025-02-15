mod config;
use crate::config::ConfigFile;

use libc::{sysconf, _SC_PAGESIZE};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{canonicalize, read_link};
use std::os::raw::c_long;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

fn get_page_size() -> Option<usize> {
    unsafe {
        let page_size: c_long = sysconf(_SC_PAGESIZE);
        if page_size == -1 {
            None // Error retrieving page size
        } else {
            Some(page_size as usize)
        }
    }
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

    // File descriptor 3 is where binfmt_misc typically passes the executable
    let binary = read_link("/proc/self/fd/3").unwrap();
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
        let mut dnf_command = Command::new("sudo");
        dnf_command.arg("dnf");
        dnf_command.arg("install");
        dnf_command.args(&interpreter_missing_paths);
        debug!("Running:\n{:#?}", dnf_command);
        match dnf_command.spawn() {
            Ok(mut child) => {
                let status = child.wait().expect("Failed to wait on dnf process");
                if !status.success() {
                    error!(
                        "Failed to install missing requirements: dnf returned {:?}",
                        status
                    );
                    std::process::exit(1);
                }
            }
            Err(e) => {
                error!("Failed to execute dnf: {}", e);
                std::process::exit(1);
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
            error!("Failed to get page size");
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
    error!("Failed to execute binary");
    std::process::exit(1);
}
