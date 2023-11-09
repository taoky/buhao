use buhao_library::{BuhaoCodec, RequestActionType, ResponseActionType};
use tokio_util::codec::Framed;
use std::{path::Path, sync::{Arc, Mutex}};

use log::warn;
use tokio::net::UnixListener;
use tokio_stream::StreamExt;
use futures::sink::SinkExt;

mod fs;
use fs::Filesystem;


#[tokio::main]
async fn main() {
    // init logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    // unlink before bind if possible
    std::fs::remove_file("/tmp/buhao.sock").unwrap_or(());
    let listener = UnixListener::bind("/tmp/buhao.sock").unwrap();
    let filesystem = Arc::new(Mutex::new(Filesystem::load_from_fs(Path::new("/tmp/buhao"))));

    loop {
        match listener.accept().await {
            Ok((socket, _addr)) => {
                let filesystem = filesystem.clone();
                tokio::spawn(async move {
                    let mut framed = Framed::new(socket, BuhaoCodec);

                    while let Some(message) = framed.next().await {
                        match message {
                            Ok((message_type, payload)) => {
                                let action_type = RequestActionType::try_from(message_type);
                                match action_type {
                                    Ok(RequestActionType::Refresh) => {
                                        unimplemented!("filesystem refresh")
                                    }
                                    Ok(RequestActionType::Get) => {
                                        let path = payload["path"].as_str().unwrap();
                                        let path = Path::new(path);
                                        let result = {
                                            let filesystem = filesystem.lock().unwrap();
                                            filesystem.open(path).cloned()
                                        };
                                        let result = match result {
                                            Err(e) => {
                                                (ResponseActionType::Error, format!("{}", e).as_bytes().to_vec())
                                            },
                                            Ok(inode) => {
                                                match inode.serialize_metadata() {
                                                    Err(e) => {
                                                        (ResponseActionType::Error, format!("{}", e).as_bytes().to_vec())
                                                    },
                                                    Ok(metadata) => {
                                                        (ResponseActionType::Ok, metadata.as_slice().to_vec())
                                                    }
                                                }
                                            }
                                        };
                                        let result: (u8, Vec<u8>) = (result.0.into(), result.1);
                                        if let Err(e) = framed.send(result).await {
                                            warn!("Error sending message: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Error decoding message: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Error decoding message: {}", e);
                            }
                        }
                        
                    }
                });
            }
            Err(e) => {
                warn!("Error accepting connection: {}", e);
            }
        }
    }
}
