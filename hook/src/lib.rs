use anyhow::Result;
use path_clean::PathClean;
use std::ffi::{c_char, CStr};

#[ctor::ctor]
fn init_log() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}

pub fn construct_absoulte_path(path: &str) -> Result<String, std::io::Error> {
    let path = if path.starts_with('/') {
        path.to_owned()
    } else {
        let cwd = std::env::current_dir()?;
        cwd.join(path)
            .clean()
            .to_str()
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Path is not valid UTF-8",
            ))?
            .to_owned()
    };
    Ok(path)
}

pub(crate) fn get_path(path: *const c_char) -> Result<String> {
    let path = unsafe { CStr::from_ptr(path) }.to_str()?;
    // convert to absolute path if it is not
    Ok(construct_absoulte_path(path)?)
}

pub(crate) fn set_errno_code(code: i32) {
    unsafe {
        *libc::__errno_location() = code;
    }
}

macro_rules! get {
    ($path: expr) => {{
        let path = &$crate::construct_absoulte_path($path)?;
        $crate::manager::MANAGER.with(|m| m.borrow_mut().get(path))
    }};
}

macro_rules! open {
    ($path: expr, $dirop: expr) => {{
        let path = &$crate::construct_absoulte_path($path)?;
        $crate::manager::MANAGER.with(|m| m.borrow_mut().open(path, $dirop))
    }};
}

macro_rules! get_dirstate {
    ($fd: expr) => {
        $crate::manager::MANAGER.with(|m| m.borrow().get_dirstate($fd).cloned())
    };
}

macro_rules! set_dirstate {
    ($fd: expr, $state: expr) => {
        $crate::manager::MANAGER.with(|m| m.borrow_mut().set_dirstate($fd, $state))
    };
}

macro_rules! retrieve_fd {
    ($fd: expr) => {
        $crate::manager::MANAGER.with(|m| m.borrow().retrieve_fd($fd).cloned())
    };
}

macro_rules! close {
    ($fd: expr, $dirop: expr) => {
        $crate::manager::MANAGER.with(|m| m.borrow_mut().close($fd, $dirop))
    };
}

const LOWER_DIRFD_BOUND: u64 = 0x0000800000000000;
const LOWER_FD_BOUND: i32 = 0x00800000;

mod dir;
mod manager;
mod open;
mod stat;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_path() {
        let cwd_before = std::env::current_dir().unwrap();
        std::fs::create_dir_all("/tmp/buhao").unwrap();
        std::env::set_current_dir("/tmp/buhao").unwrap();
        let path = "/tmp";
        assert_eq!(construct_absoulte_path(path).unwrap(), path);
        let path = ".";
        assert_eq!(construct_absoulte_path(path).unwrap(), "/tmp/buhao/");

        std::env::set_current_dir(cwd_before).unwrap();
    }
}
