use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::os::unix::net::UnixStream;

use anyhow::Result;
use buhao_lib::syncframed::SyncFramed;
use buhao_lib::{
    BuhaoCodec, DirectoryContents, Inode, Item, RequestActionType, ResponseActionType,
    BUHAO_SOCK_PATH,
};
use serde_json::json;

thread_local! {
    pub static MANAGER: RefCell<Manager> = RefCell::new(Manager::default());
}

#[derive(Debug)]
struct ShadowFd {
    path: String,
    real_fd: Option<i32>,
    info: Inode,
}

pub struct Manager {
    framed: SyncFramed<UnixStream, BuhaoCodec, Item>,
    fd_map: HashMap<i32, ShadowFd>,
    next_fd: i32,
}

impl Default for Manager {
    fn default() -> Self {
        let stream = UnixStream::connect(BUHAO_SOCK_PATH).unwrap();
        let codec = BuhaoCodec;
        Self {
            framed: SyncFramed::new(stream, codec),
            fd_map: HashMap::new(),
            next_fd: -1,
        }
    }
}

macro_rules! check_managed {
    ($self:ident, $path:ident) => {
        if !$self.is_managed($path) {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "not managed path").into());
        }
    };
}

/// Assuming the path is absolute
impl Manager {
    pub fn interact(&mut self, item: Item) -> Item {
        self.framed.send(item).unwrap();
        self.framed.recv().unwrap()
    }

    pub fn is_managed(&self, path: &str) -> bool {
        // TODO: get managed path from server
        path.starts_with("/tmp/buhao/") || path == "/tmp/buhao"
    }

    /// Get file info from remote server
    pub fn get(&mut self, path: &str) -> Result<Inode> {
        check_managed!(self, path);
        let item = (RequestActionType::Get.into(), json!({"path": path}));
        let resp = self.interact(item);
        if resp.0 == <ResponseActionType as Into<u8>>::into(ResponseActionType::Ok) {
            Ok(serde_json::from_value(resp.1)?)
        } else {
            Err(anyhow::anyhow!("{}", resp.1))
        }
    }

    pub fn open(&mut self, path: &str) -> Result<i32> {
        unimplemented!()
    }
}
