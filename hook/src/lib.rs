use std::path::Path;

#[ctor::ctor]
fn init_log() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}

pub fn construct_absoulte_path(path: &str) -> Result<String, std::io::Error> {
    let path = Path::new(path);
    if path.is_absolute() {
        Ok(path.to_str().unwrap().to_string())
    } else {
        let cwd = std::env::current_dir()?;
        Ok(cwd.join(path).to_str().unwrap().to_string())
    }
}

macro_rules! open {
    ($path: expr) => {
        {
            let path = &$crate::construct_absoulte_path($path)?;
            // TODO: get managed path from server
            if !path.starts_with("/tmp") {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "not managed path").into());
            }
            MANAGER.with(|m| m.borrow_mut().open(path))
        }
    };
}

mod dir;
mod manager;
mod open;
mod stat;
mod utils;
