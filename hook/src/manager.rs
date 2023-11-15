use std::os::unix::net::UnixStream;

use buhao_lib::{BuhaoCodec, Item};
use buhao_lib::syncframed::SyncFramed;

thread_local! {
    static MANAGER: Manager = Manager::default();
}

#[derive(Default)]
pub struct Manager {
    framed: Option<SyncFramed<UnixStream, BuhaoCodec, Item>>,
}

impl Manager {
    fn init(&mut self) {
        if self.framed.is_none() {
            let stream = UnixStream::connect("/tmp/buhao.sock").unwrap();
            let codec = BuhaoCodec;
            self.framed = Some(SyncFramed::new(stream, codec));
        }
    }
}