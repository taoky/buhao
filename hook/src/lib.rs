#[ctor::ctor]
fn init_log() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}

mod manager;
mod dir;
mod open;
mod stat;
mod utils;
