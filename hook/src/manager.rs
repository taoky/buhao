use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::os::unix::net::UnixStream;

use anyhow::Result;
use buhao_lib::syncframed::SyncFramed;
use buhao_lib::{
    BuhaoCodec, DirectoryContents, Inode, Item, RequestActionType, ResponseActionType, BUHAO_SOCK_PATH
};
use serde_json::json;

thread_local! {
    pub static MANAGER: RefCell<Manager> = RefCell::new(Manager::default());
}

pub struct HookDir {
    path: String,
    contents: DirectoryContents,
}

pub struct Manager {
    framed: SyncFramed<UnixStream, BuhaoCodec, Item>,
    allocated_dir: HashSet<u64>,
    dir_map: HashMap<u64, HookDir>,
}

impl Default for Manager {
    fn default() -> Self {
        let stream = UnixStream::connect(BUHAO_SOCK_PATH).unwrap();
        let codec = BuhaoCodec;
        Self {
            framed: SyncFramed::new(stream, codec),
            allocated_dir: HashSet::new(),
            dir_map: HashMap::new(),
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
        Ok(serde_json::from_value(self.interact(item).1)?)
    }

    // pub fn opendir(&mut self, path: &str) -> Result<Item> {
    //     check_managed!(self, path);
    //     let item = (RequestActionType::Get.into(), json!({"path": path}));
    //     let item = self.interact(item);
    //     if item.0 == ResponseActionType::Ok {
    //         let dir_id = item.1["dir_id"].as_u64().unwrap();
    //         self.allocated_dir.insert(dir_id);
    //         // WIP
    //         // self.dir_map.insert(dir_id, HookDir {
    //         //     path: path.to_string(),
    //         //     contents: serde_json::from_value(item.1["contents"].clone()).unwrap()
    //         // });
    //     }

    //     unimplemented!()
    // }
}
