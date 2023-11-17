use std::ffi::c_char;
use anyhow::Result;
use log::{info, warn};
use redhook::hook;

use crate::utils::get_path;

fn opendir_hook(dirptr: *const c_char) -> Result<*mut libc::DIR> {
    let path = match get_path(dirptr) {
        Ok(s) => s,
        Err(e) => {
            warn!("opendir_hook: invalid path ({})", e);
            return Err(e);
        }
    };
    info!("opendir: {}", path);
    info!("{:?}", open!(path.as_str()));
    Ok(unsafe { redhook::real!(opendir)(dirptr) })
}

hook! {
    unsafe fn opendir(dirptr: *const c_char) -> *mut libc::DIR => my_opendir {
        match opendir_hook(dirptr) {
            Err(_) => redhook::real!(opendir)(dirptr),
            Ok(fd) => fd,
        }
    }
}

hook! {
    unsafe fn readdir(dirp: *mut libc::DIR) -> *mut libc::dirent => my_readdir {
        let entry = redhook::real!(readdir)(dirp);
        if entry.is_null() {
            return entry;
        }
        let name = std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()).to_str().unwrap();
        info!("readdir: {}", name);
        entry
    }
}

hook! {
    unsafe fn readdir64(dirp: *mut libc::DIR) -> *mut libc::dirent64 => my_readdir64 {
        let entry = redhook::real!(readdir64)(dirp);
        if entry.is_null() {
            return entry;
        }
        let name = std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()).to_str().unwrap();
        info!("readdir64: {}", name);
        entry
    }
}
