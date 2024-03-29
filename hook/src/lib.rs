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

macro_rules! get {
    ($path: expr) => {{
        let path = &$crate::construct_absoulte_path($path)?;
        $crate::manager::MANAGER.with(|m| m.borrow_mut().get(path))
    }};
}

mod dir;
mod manager;
mod open;
mod stat;
mod utils;
