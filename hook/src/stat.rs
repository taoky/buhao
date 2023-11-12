use anyhow::Result;
use log::{info, warn};
use redhook::hook;
use std::ffi::c_char;

use crate::utils::get_path;

unsafe fn stat_hook(ptr: *const c_char, buf: *mut libc::stat, use_lstat: bool) -> Result<i32> {
    let path = match get_path(ptr) {
        Ok(s) => s,
        Err(e) => {
            warn!("stat_hook: invalid path ({})", e);
            return Err(e);
        }
    };
    info!("stat: {}", path);
    if !use_lstat {
        Ok(redhook::real!(stat)(ptr, buf))
    } else {
        Ok(redhook::real!(lstat)(ptr, buf))
    }
}

unsafe fn stat64_hook(ptr: *const c_char, buf: *mut libc::stat64, use_lstat: bool) -> Result<i32> {
    let path = match get_path(ptr) {
        Ok(s) => s,
        Err(e) => {
            warn!("stat_hook: invalid path ({})", e);
            return Err(e);
        }
    };
    info!("stat: {}", path);
    if !use_lstat {
        Ok(redhook::real!(stat64)(ptr, buf))
    } else {
        Ok(redhook::real!(lstat64)(ptr, buf))
    }
}

hook! {
    unsafe fn stat(path: *const c_char, buf: *mut libc::stat) -> i32 => my_stat {
        match stat_hook(path, buf, false) {
            Err(_) => redhook::real!(stat)(path, buf),
            Ok(fd) => fd,
        }
    }
}

hook! {
    unsafe fn stat64(path: *const c_char, buf: *mut libc::stat64) -> i32 => my_stat64 {
        match stat64_hook(path, buf, false) {
            Err(_) => redhook::real!(stat64)(path, buf),
            Ok(fd) => fd,
        }
    }
}

hook! {
    unsafe fn fstat(fd: i32, buf: *mut libc::stat) -> i32 => my_fstat {
        info!("fstat: {}", fd);
        redhook::real!(fstat)(fd, buf)
    }
}

hook! {
    unsafe fn fstat64(fd: i32, buf: *mut libc::stat64) -> i32 => my_fstat64 {
        info!("fstat64: {}", fd);
        redhook::real!(fstat64)(fd, buf)
    }
}

hook! {
    // TODO: when dirfd != AT_FDCWD, and flags
    unsafe fn fstatat(dirfd: i32, path: *const c_char, buf: *mut libc::stat, flags: i32) -> i32 => my_fstatat {
        match stat_hook(path, buf, false) {
            Err(_) => redhook::real!(fstatat)(dirfd, path, buf, flags),
            Ok(fd) => fd,
        }
    }
}

hook! {
    // TODO: handling symlink
    unsafe fn lstat(path: *const c_char, buf: *mut libc::stat) -> i32 => my_lstat {
        match stat_hook(path, buf, true) {
            Err(_) => redhook::real!(lstat)(path, buf),
            Ok(fd) => fd,
        }
    }
}

hook! {
    // TODO: handling symlink
    unsafe fn lstat64(path: *const c_char, buf: *mut libc::stat64) -> i32 => my_lstat64 {
        match stat64_hook(path, buf, true) {
            Err(_) => redhook::real!(lstat64)(path, buf),
            Ok(fd) => fd,
        }
    }
}
