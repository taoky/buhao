use anyhow::{anyhow, Result};
use path_clean::PathClean;
use std::ffi::{c_char, CStr};

fn get_path_internal(path: &str) -> Result<String> {
    let path = if path.starts_with("/") {
        path.to_owned()
    } else {
        let cwd = std::env::current_dir()?;
        cwd.join(path)
            .clean()
            .to_str()
            .ok_or(anyhow!("invalid path"))?
            .to_owned()
    };
    Ok(path)
}

pub fn get_path(path: *const c_char) -> Result<String> {
    let path = unsafe { CStr::from_ptr(path) }.to_str()?;
    // convert to absolute path if it is not
    get_path_internal(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_path() {
        let cwd_before = std::env::current_dir().unwrap();
        std::fs::create_dir_all("/tmp/buhao").unwrap();
        std::env::set_current_dir("/tmp/buhao").unwrap();
        let path = "/tmp";
        assert_eq!(get_path_internal(path).unwrap(), path);
        let path = ".";
        assert_eq!(get_path_internal(path).unwrap(), "/tmp/buhao/");

        std::env::set_current_dir(cwd_before).unwrap();
    }
}
