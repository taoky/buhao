use std::cell::RefCell;
use std::collections::HashMap;
use std::os::unix::net::UnixStream;

use anyhow::Result;
use buhao_lib::syncframed::SyncFramed;
use buhao_lib::{
    BuhaoCodec, Contents, Inode, Item, RequestActionType, ResponseActionType, BUHAO_SOCK_PATH,
};
use serde_json::json;

use crate::{LOWER_DIRFD_BOUND, LOWER_FD_BOUND};

thread_local! {
    pub static MANAGER: RefCell<Manager> = RefCell::new(Manager::default());
}

#[derive(Debug, Clone)]
pub struct ShadowFd {
    pub path: String,
    real_fd: Option<i32>,
    pub info: Inode,
}

#[derive(Debug, Clone)]
pub struct DirState {
    pub idx: usize,
}

#[derive(Debug)]
pub struct Manager {
    framed: SyncFramed<UnixStream, BuhaoCodec, Item>,
    fd_map: HashMap<u64, ShadowFd>,
    next_fd: i32,
    next_dirfd: u64,
    dir_state: HashMap<u64, DirState>,
}

impl Default for Manager {
    fn default() -> Self {
        let stream = UnixStream::connect(BUHAO_SOCK_PATH).unwrap();
        let codec = BuhaoCodec;
        Self {
            framed: SyncFramed::new(stream, codec),
            fd_map: HashMap::new(),
            next_fd: LOWER_FD_BOUND,
            next_dirfd: LOWER_DIRFD_BOUND,
            dir_state: HashMap::new(),
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

    pub fn open(&mut self, path: &str, dir_op: bool) -> Result<u64> {
        let inode = self.get(path)?;
        if dir_op {
            if let Contents::Directory(_) = inode.contents {
            } else {
                return Err(
                    std::io::Error::new(std::io::ErrorKind::Other, "not a directory").into(),
                );
            }
        }
        let shadow_fd = ShadowFd {
            path: path.to_string(),
            real_fd: None,
            info: inode,
        };

        if dir_op {
            self.fd_map.insert(self.next_dirfd, shadow_fd);
            self.register_dir(self.next_dirfd);
            self.next_dirfd += 1;
            Ok(self.next_dirfd - 1)
        } else {
            self.fd_map.insert(self.next_fd as u64, shadow_fd);
            self.next_fd += 1;
            Ok(self.next_fd as u64 - 1)
        }
    }

    pub fn close(&mut self, fd: u64, dir_op: bool) {
        if dir_op {
            self.unregister_dir(fd);
        }
        self.fd_map.remove(&fd);
    }

    fn register_dir(&mut self, fd: u64) {
        self.dir_state.insert(fd, DirState { idx: 0 });
    }

    fn unregister_dir(&mut self, fd: u64) {
        self.dir_state.remove(&fd);
    }

    pub fn get_dirstate(&self, fd: u64) -> Option<&DirState> {
        self.dir_state.get(&fd)
    }

    pub fn set_dirstate(&mut self, fd: u64, state: DirState) {
        self.dir_state.insert(fd, state);
    }

    pub fn retrieve_fd(&self, fd: u64) -> Option<&ShadowFd> {
        self.fd_map.get(&fd)
    }
}
