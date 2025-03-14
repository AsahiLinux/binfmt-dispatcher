// SPDX-License-Identifier: MIT

use libc::{sysconf, _SC_PAGESIZE};
use std::os::raw::c_long;

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
