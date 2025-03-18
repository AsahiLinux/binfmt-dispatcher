// SPDX-License-Identifier: MIT

use libc::{sysconf, _SC_PAGESIZE};
use std::os::raw::c_long;

use zenity_dialog::dialog;
use zenity_dialog::ZenityDialog;
use zenity_dialog::ZenityOutput;

use log::warn;

pub fn get_page_size() -> Option<usize> {
    unsafe {
        let page_size: c_long = sysconf(_SC_PAGESIZE);
        if page_size == -1 {
            None // Error retrieving page size
        } else {
            Some(page_size as usize)
        }
    }
}

pub fn info_dialog(msg: &str) -> bool {
    let result = ZenityDialog::new(dialog::Info::new().with_text(msg))
        .with_title(env!("CARGO_PKG_NAME"))
        .show();

    if result.is_err() {
        warn!("Failed to create dialog, is zenity installed?");
        return false;
    }

    match result.unwrap() {
        ZenityOutput::Affirmed { .. } => return true,
        _ => return false,
    }
}

pub fn error_dialog(msg: &str) -> bool {
    let result = ZenityDialog::new(dialog::Error::new().with_text(msg))
        .with_title(env!("CARGO_PKG_NAME"))
        .show();

    if result.is_err() {
        warn!("Failed to create dialog, is zenity installed?");
        return false;
    }

    match result.unwrap() {
        ZenityOutput::Affirmed { .. } => return true,
        _ => return false,
    }
}
