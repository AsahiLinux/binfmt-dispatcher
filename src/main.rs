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
    // Collect command line arguments to pass through
    let args: Vec<OsString> = env::args_os().skip(1).collect();

    // File descriptor 3 is where binfmt_misc typically passes the executable
    let binary = read_link("/proc/self/fd/3").unwrap();

    let settings: ConfigFile = config::parse_config().unwrap();

    let mut emulator_id = &settings.defaults.emulator;
    for binary in settings.binaries.values() {
        if Path::new(&binary.path) == canonicalize(&args[0]).unwrap() {
            emulator_id = &binary.emulator;
            break;
        }
    }

    let emulator = settings.emulators.get(emulator_id).unwrap();
    let emulator_path = &emulator.path;

    let mut use_muvm = emulator.use_muvm.unwrap();
    if use_muvm {
        if let Some(size) = get_page_size() {
            println!("Page size: {} bytes", size);
            // Use muvm if the page-size is not 4k
            use_muvm = size != 4096;
        } else {
            eprintln!("Failed to get page size");
            use_muvm = false;
        }
    }

    let mut command;
    if use_muvm {
        println!("Using muvm");
        command = Command::new("/usr/bin/muvm");
        command.arg("--");
        command.arg(emulator_path);
    } else {
        command = Command::new(emulator_path);
    }

    command.arg(binary);

    // Pass through all the arguments
    command.args(&args);

    // Execute the command and replace the current process
    // Use a panic instead of expecting a return value
    let _ = command.exec();

    // If exec fails, it will not return; however, we include this to handle the case.
    eprintln!("Failed to execute binary");
    std::process::exit(1);
}
