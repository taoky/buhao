use std::cell::RefCell;
use std::os::unix::net::UnixStream;

use buhao_lib::syncframed::SyncFramed;
use buhao_lib::{BuhaoCodec, Item, RequestActionType, BUHAO_SOCK_PATH};
use serde_json::json;

thread_local! {
    pub static MANAGER: RefCell<Manager> = RefCell::new(Manager::default());
}

pub struct Manager {
    framed: SyncFramed<UnixStream, BuhaoCodec, Item>,
}

impl Default for Manager {
    fn default() -> Self {
        let stream = UnixStream::connect(BUHAO_SOCK_PATH).unwrap();
        let codec = BuhaoCodec;
        Self {
            framed: SyncFramed::new(stream, codec),
        }
    }
}

impl Manager {
    pub fn interact(&mut self, item: Item) -> Item {
        self.framed.send(item).unwrap();
        self.framed.recv().unwrap()
    }

    pub fn open(&mut self, path: &str) -> Item {
        let item = (RequestActionType::Get.into(), json!({"path": path}));
        self.interact(item)
    }
}
