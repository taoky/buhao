/// Open files and directories.
use crate::{get_path, manager::ShadowFd};
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
    let fd = open!(path.as_str(), oflag, false)?;
    info!("using fake fd: {}", fd);
    Ok(fd as i32)
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
    // TODO: stub
    unsafe fn fdopendir(fd: i32) -> *mut libc::DIR => my_fdopendir {
        info!("fdopendir (stub): {}", fd);
        redhook::real!(fdopendir)(fd)
    }
}

hook! {
    unsafe fn close(fd: i32) -> i32 => my_close {
        if fd < crate::LOWER_FD_BOUND {
            return redhook::real!(close)(fd);
        }
        let fd = fd as u64;
        let info: ShadowFd = match retrieve_fd!(fd) {
            Some(info) => info,
            None => {
                warn!("close: invalid fd");
                return -1;
            }
        };
        info!("close: {:?}", info);
        close!(fd, false);
        0
    }
}
