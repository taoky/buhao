use anyhow::Result;
use buhao_lib::{Contents, InodeType};
use log::{info, warn};
use redhook::hook;
use std::{ffi::c_char, ptr::null_mut};

use crate::{get_path, manager::{DirState, ShadowFd}, set_errno_code};

// Well, libc::DIR is opaque, so we can't really do anything with it
// But we could assume it shall always be a valid pointer if not provided by us
// Here we set LOWER_FD_BOUND to the lower bound of kernel space, so we can use it as an indicator

fn opendir_hook(dirptr: *const c_char) -> Result<*mut libc::DIR> {
    let path = match get_path(dirptr) {
        Ok(s) => s,
        Err(e) => {
            warn!("opendir_hook: invalid path ({})", e);
            return Err(e);
        }
    };
    info!("opendir: {}", path);
    let fd = open!(path.as_str(), true)?;
    info!("using fake libc::DIR: {}", fd);
    Ok(fd as *mut libc::DIR)
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
    // TODO: stub
    unsafe fn readdir(dirp: *mut libc::DIR) -> *mut libc::dirent => my_readdir {
        if (dirp as u64) < crate::LOWER_FD_BOUND {
            return redhook::real!(readdir)(dirp);
        }
        let info: ShadowFd = match retrieve_fd!(dirp as u64) {
            Some(info) => info,
            None => {
                warn!("readdir: invalid libc::DIR");
                set_errno_code(libc::EBADF);
                return null_mut();
            }
        };
        info!("readdir: {:?}", info);
        let state: DirState = match get_dirstate!(dirp as u64) {
            Some(dirent) => dirent,
            None => {
                warn!("readdir: invalid libc::DIR");
                set_errno_code(libc::EBADF);
                return null_mut();
            }
        };
        let idx = state.idx;
        let dirent_item = match info.info.contents {
            Contents::Directory(ref v) => v.children.get(idx),
            _ => {
                warn!("readdir: not a directory");
                set_errno_code(libc::ENOTDIR);
                return null_mut();
            }
        };
        let dirent = match dirent_item {
            Some(dirent) => dirent,
            None => {
                info!("readdir: end of directory");
                return null_mut();
            }
        };
        // let dirent_item = info.info.contents.get(dirent_idx);
        // sorry but I don't know how not to let it leak
        let res = libc::malloc(std::mem::size_of::<libc::dirent>()) as *mut libc::dirent;
        (*res).d_ino = dirent.inode;
        (*res).d_off = idx as i64;
        (*res).d_reclen = std::mem::size_of::<libc::dirent>() as u16;
        (*res).d_type = match dirent.itype {
            InodeType::Directory => libc::DT_DIR,
            InodeType::File => libc::DT_REG,
            InodeType::Symlink => libc::DT_LNK,
        };
        set_dirstate!(dirp as u64, DirState { idx: idx + 1 });
        res
    }
}

hook! {
    // TODO: stub
    unsafe fn readdir64(dirp: *mut libc::DIR) -> *mut libc::dirent64 => my_readdir64 {
        if (dirp as u64) < crate::LOWER_FD_BOUND {
            return redhook::real!(readdir64)(dirp);
        }
        let entry = redhook::real!(readdir64)(dirp);
        if entry.is_null() {
            return entry;
        }
        let name = std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()).to_str().unwrap();
        info!("readdir64 (stub): {}", name);
        entry
    }
}
