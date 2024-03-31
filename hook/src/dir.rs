use anyhow::Result;
use buhao_lib::{Contents, DirectoryItem, InodeType};
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
    let fd = open!(path.as_str(), 0, true)?;
    info!("using fake libc::DIR: {}", fd);
    Ok(fd as *mut libc::DIR)
}

fn readdir_get_dirent(dirp: u64) -> Result<(DirectoryItem, i64)> {
    let info: ShadowFd = match retrieve_fd!(dirp) {
        Some(info) => info,
        None => {
            warn!("readdir: invalid libc::DIR");
            set_errno_code(libc::EBADF);
            return Err(anyhow::anyhow!("invalid libc::DIR"));
        }
    };
    info!("readdir: {:?}", info.path);
    let state: DirState = match get_dirstate!(dirp) {
        Some(dirent) => dirent,
        None => {
            warn!("readdir: invalid libc::DIR");
            set_errno_code(libc::EBADF);
            return Err(anyhow::anyhow!("invalid libc::DIR"));
        }
    };
    let idx = state.idx;
    let dirent_item = match info.info.contents {
        Contents::Directory(ref v) => v.children.get(idx),
        _ => {
            warn!("readdir: not a directory");
            set_errno_code(libc::ENOTDIR);
            return Err(anyhow::anyhow!("not a directory"));
        }
    };
    let dirent = match dirent_item {
        Some(dirent) => dirent,
        None => {
            info!("readdir: end of directory");
            return Err(anyhow::anyhow!("end of directory"));
        }
    };
    Ok((dirent.clone(), idx as i64))
}

macro_rules! alloc_dirent {
    ($dirent: expr, $idx:expr, $typ:ty) => {{
        let name = $dirent.name;
        let inode = $dirent.inode;
        let itype = $dirent.itype;
        if name.len() >= 256 {
            warn!("readdir: name too long");
            set_errno_code(libc::ENAMETOOLONG);
            return null_mut();
        }
        let res = libc::malloc(std::mem::size_of::<$typ>()) as *mut $typ;
        (*res).d_ino = inode;
        (*res).d_off = $idx;
        (*res).d_reclen = std::mem::size_of::<$typ>() as u16;
        (*res).d_type = match itype {
            InodeType::Directory => libc::DT_DIR,
            InodeType::File => libc::DT_REG,
            InodeType::Symlink => libc::DT_LNK,
        };
        // SAFE: d_name is a 256-byte buffer
        let cname = std::ffi::CString::new(name.clone()).unwrap();
        std::ptr::copy_nonoverlapping(cname.as_ptr(), (*res).d_name.as_mut_ptr(), name.len() + 1);
        res
    }};
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
        if (dirp as u64) < crate::LOWER_DIRFD_BOUND {
            return redhook::real!(readdir)(dirp);
        }
        let (dirent, idx) = match readdir_get_dirent(dirp as u64) {
            Ok(dirent) => dirent,
            Err(_) => return null_mut(),
        };

        info!("readdir item: {}", dirent.name);
        let res = alloc_dirent!(dirent, idx, libc::dirent);
        set_dirstate!(dirp as u64, DirState { idx: idx as usize + 1 });
        res
    }
}

hook! {
    unsafe fn readdir64(dirp: *mut libc::DIR) -> *mut libc::dirent64 => my_readdir64 {
        if (dirp as u64) < crate::LOWER_DIRFD_BOUND {
            return redhook::real!(readdir64)(dirp);
        }
        let (dirent, idx) = match readdir_get_dirent(dirp as u64) {
            Ok(dirent) => dirent,
            Err(_) => return null_mut(),
        };

        info!("readdir64 item: {}", dirent.name);
        let res = alloc_dirent!(dirent, idx, libc::dirent64);
        set_dirstate!(dirp as u64, DirState { idx: idx as usize + 1 });
        res
    }
}

hook! {
    unsafe fn closedir(dirp: *mut libc::DIR) -> i32 => my_closedir {
        if (dirp as u64) < crate::LOWER_DIRFD_BOUND {
            return redhook::real!(closedir)(dirp);
        }
        let info: ShadowFd = match retrieve_fd!(dirp as u64) {
            Some(info) => info,
            None => {
                warn!("closedir: invalid libc::DIR");
                set_errno_code(libc::EBADF);
                return -1;
            }
        };
        info!("closedir: {:?}", info);
        close!(dirp as u64, true);
        0
    }
}