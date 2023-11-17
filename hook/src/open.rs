/// Open files and directories.
use crate::utils::get_path;
use anyhow::Result;
use log::{info, warn};
use redhook::hook;
use std::ffi::c_char;

fn open_hook(ptr: *const c_char, oflag: i32, mode: u32) -> Result<i32> {
    let path = match get_path(ptr) {
        Ok(s) => s,
        Err(e) => {
            warn!("open_hook: invalid path ({})", e);
            return Err(e);
        }
    };
    info!("open: {}, {}, {}", path, oflag, mode);
    info!("{:?}", open!(path.as_str()));
    Ok(unsafe { redhook::real!(open)(ptr, oflag, mode) })
}

hook! {
    unsafe fn open(ptr: *const c_char, oflag: i32, mode: u32) -> i32 => my_open {
        match open_hook(ptr, oflag, mode) {
            Err(_) => redhook::real!(open)(ptr, oflag, mode),
            Ok(fd) => fd,
        }
    }
}

hook! {
    // TODO: when dirfd != AT_FDCWD
    unsafe fn openat(dirfd: i32, ptr: *const c_char, flags: i32, mode: u32) -> i32 => my_openat {
        match open_hook(ptr, flags, mode) {
            Err(_) => redhook::real!(openat)(dirfd, ptr, flags, mode),
            Ok(fd) => fd,
        }
    }
}

hook! {
    unsafe fn open64(ptr: *const c_char, oflag: i32, mode: u32) -> i32 => my_open64 {
        match open_hook(ptr, oflag, mode) {
            Err(_) => redhook::real!(open64)(ptr, oflag, mode),
            Ok(fd) => fd,
        }
    }
}

hook! {
    // TODO: when dirfd != AT_FDCWD
    unsafe fn openat64(dirfd: i32, ptr: *const c_char, flags: i32, mode: u32) -> i32 => my_openat64 {
        match open_hook(ptr, flags, mode) {
            Err(_) => redhook::real!(openat64)(dirfd, ptr, flags, mode),
            Ok(fd) => fd,
        }
    }
}

hook! {
    unsafe fn fdopendir(fd: i32) -> *mut libc::DIR => my_fdopendir {
        info!("fdopendir: {}", fd);
        redhook::real!(fdopendir)(fd)
    }
}
