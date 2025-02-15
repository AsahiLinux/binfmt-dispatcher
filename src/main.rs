mod config;
use crate::config::ConfigFile;

use libc::{sysconf, _SC_PAGESIZE};
use std::env;
use std::ffi::OsString;
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

    let mut emulator_id = &settings.defaults.emulator;
    for binary in settings.binaries.values() {
        if Path::new(&binary.path) == canonicalize(&args[0]).unwrap() {
            emulator_id = &binary.emulator;
            break;
        }
    }

    let emulator = settings.emulators.get(emulator_id).unwrap();
    let emulator_name = emulator.name.as_ref().unwrap();
    let emulator_path = &emulator.path;

    let mut use_muvm = emulator.use_muvm.unwrap();
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
        info!("Using {} with muvm", emulator_name);
        command = Command::new("/usr/bin/muvm");
        command.arg("--");
        command.arg(emulator_path);
    } else {
        info!("Using {}", emulator_name);
        command = Command::new(emulator_path);
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
