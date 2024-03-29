use anyhow::Result;
use buhao_lib::{Contents, Inode};
use libc::AT_FDCWD;
use log::{info, warn};
use redhook::hook;
use std::ffi::c_char;

use crate::utils::get_path;

const RECURSIVE_LIMIT: usize = 10;

macro_rules! inode_to_stat {
    ($inode: ident, $buf: ident) => {
        unsafe {
            (*$buf).st_dev = 0;
            (*$buf).st_ino = $inode.id;
            match $inode.contents {
                Contents::File => {
                    (*$buf).st_mode = libc::S_IFREG;
                }
                Contents::Directory(_) => {
                    (*$buf).st_mode = libc::S_IFDIR;
                }
                Contents::Symlink(_) => {
                    (*$buf).st_mode = libc::S_IFLNK;
                }
            }
            (*$buf).st_nlink = $inode.nlink;
            (*$buf).st_mode = $inode.mode;
            (*$buf).st_uid = $inode.uid;
            (*$buf).st_gid = $inode.gid;
            (*$buf).st_rdev = 0;
            (*$buf).st_size = $inode.size;
            (*$buf).st_blksize = 4096;
            (*$buf).st_blocks = 8;
            (*$buf).st_atime = $inode.atime;
            (*$buf).st_mtime = $inode.mtime;
            (*$buf).st_ctime = $inode.ctime;
        }
    };
}

fn stat_hook(
    ptr: *const c_char,
    buf: *mut libc::stat,
    use_lstat: bool,
    recursive: usize,
) -> Result<i32> {
    if recursive > RECURSIVE_LIMIT {
        warn!("stat_hook: recursive limit reached");
        return Err(anyhow::anyhow!("recursive limit reached"));
    }
    let path = match get_path(ptr) {
        Ok(s) => s,
        Err(e) => {
            warn!("stat_hook: invalid path ({})", e);
            return Err(e);
        }
    };
    info!("stat: {}", path);
    let resp: Inode = get!(path.as_str())?;
    info!("{:?}", resp);
    let is_symlink = match resp.contents {
        Contents::Symlink(_) => true,
        _ => false,
    };
    if !use_lstat || !is_symlink {
        inode_to_stat!(resp, buf);
        Ok(0)
    } else {
        // is symlink
        let new_path = match resp.contents {
            Contents::Symlink(s) => s,
            _ => {
                unreachable!("stat_hook: invalid symlink logic");
            }
        };
        let ptr = new_path.as_ptr() as *const c_char;
        stat_hook(ptr, buf, false, recursive + 1)
    }
}

// TODO
fn stat64_hook(
    ptr: *const c_char,
    buf: *mut libc::stat64,
    use_lstat: bool,
    recursive: usize,
) -> Result<i32> {
    if recursive > RECURSIVE_LIMIT {
        warn!("stat_hook: recursive limit reached");
        return Err(anyhow::anyhow!("recursive limit reached"));
    }
    let path = match get_path(ptr) {
        Ok(s) => s,
        Err(e) => {
            warn!("stat_hook: invalid path ({})", e);
            return Err(e);
        }
    };
    info!("stat64: {}", path);
    let resp: Inode = get!(path.as_str())?;
    info!("{:?}", resp);
    let is_symlink = match resp.contents {
        Contents::Symlink(_) => true,
        _ => false,
    };
    if !use_lstat || !is_symlink {
        inode_to_stat!(resp, buf);
        Ok(0)
    } else {
        // is symlink
        let new_path = match resp.contents {
            Contents::Symlink(s) => s,
            _ => {
                unreachable!("stat_hook: invalid symlink logic");
            }
        };
        let ptr = new_path.as_ptr() as *const c_char;
        stat64_hook(ptr, buf, false, recursive + 1)
    }
}

hook! {
    unsafe fn stat(path: *const c_char, buf: *mut libc::stat) -> i32 => my_stat {
        match stat_hook(path, buf, false, 0) {
            Err(_) => redhook::real!(stat)(path, buf),
            Ok(fd) => fd,
        }
    }
}

hook! {
    unsafe fn stat64(path: *const c_char, buf: *mut libc::stat64) -> i32 => my_stat64 {
        match stat64_hook(path, buf, false, 0) {
            Err(_) => redhook::real!(stat64)(path, buf),
            Ok(fd) => fd,
        }
    }
}

hook! {
    // TODO: stub
    unsafe fn fstat(fd: i32, buf: *mut libc::stat) -> i32 => my_fstat {
        info!("fstat (stub): {}", fd);
        redhook::real!(fstat)(fd, buf)
    }
}

hook! {
    // TODO: stub
    unsafe fn fstat64(fd: i32, buf: *mut libc::stat64) -> i32 => my_fstat64 {
        info!("fstat64 (stub): {}", fd);
        redhook::real!(fstat64)(fd, buf)
    }
}

hook! {
    // TODO: when dirfd != AT_FDCWD, and flags
    unsafe fn fstatat(dirfd: i32, path: *const c_char, buf: *mut libc::stat, flags: i32) -> i32 => my_fstatat {
        if dirfd != AT_FDCWD {
            warn!("fstatat: dirfd != AT_FDCWD (fallback)");
            return redhook::real!(fstatat)(dirfd, path, buf, flags);
        }
        match stat_hook(path, buf, false, 0) {
            Err(_) => redhook::real!(fstatat)(dirfd, path, buf, flags),
            Ok(fd) => fd,
        }
    }
}

hook! {
    unsafe fn lstat(path: *const c_char, buf: *mut libc::stat) -> i32 => my_lstat {
        match stat_hook(path, buf, true, 0) {
            Err(_) => redhook::real!(lstat)(path, buf),
            Ok(fd) => fd,
        }
    }
}

hook! {
    unsafe fn lstat64(path: *const c_char, buf: *mut libc::stat64) -> i32 => my_lstat64 {
        match stat64_hook(path, buf, true, 0) {
            Err(_) => redhook::real!(lstat64)(path, buf),
            Ok(fd) => fd,
        }
    }
}
