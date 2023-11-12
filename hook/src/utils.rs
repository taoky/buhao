use anyhow::Result;
use std::ffi::{c_char, CStr};

pub fn get_path(path: *const c_char) -> Result<String> {
    let path = unsafe { CStr::from_ptr(path) };
    Ok(path.to_str()?.to_owned())
}
