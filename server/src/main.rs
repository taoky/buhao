use anyhow::anyhow;
use std::path::Path;

use log::warn;
use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;

mod fs;
use fs::Filesystem;

enum ActionType {
    Refresh,
    Get,
}

impl TryFrom<u8> for ActionType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ActionType::Refresh),
            1 => Ok(ActionType::Get),
            _ => Err(anyhow!("Invalid action type: {}", value)),
        }
    }
}

#[tokio::main]
async fn main() {
    // init logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    // unlink before bind if possible
    std::fs::remove_file("/tmp/buhao.sock").unwrap_or(());
    let listener = UnixListener::bind("/tmp/buhao.sock").unwrap();
    let filesystem = Filesystem::load_from_fs(Path::new("/tmp/buhao"));

    // Packet design (bytes)
    // Request packet:
    // Action type: 1 byte
    // Action refresh: follows NULL-terminated string
    // Action get: follows NULL-terminated string
    //
    // Respond packet:
    // Action refresh: NULL-terminated string
    // Action get: a JSON string of the opened inode

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();

        // read action type
        let type_ch: ActionType = match socket.read_u8().await.unwrap().try_into() {
            Ok(t) => t,
            Err(e) => {
                warn!("Failed to read action type: {}", e);
                continue;
            }
        };

        match type_ch {
            ActionType::Refresh => {
                warn!("not implemented");
            }
            ActionType::Get => {
                // read until NULL
                
            }
        }
    }
}
