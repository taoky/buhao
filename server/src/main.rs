use buhao_lib::{BuhaoCodec, RequestActionType, ResponseActionType, convert_response_tuple};
use serde_json::json;
use tokio_util::codec::Framed;
use std::{path::Path, sync::{Arc, Mutex}};

use log::{warn, debug};
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
    let filesystem = Arc::new(Mutex::new(Filesystem::load_from_fs(Path::new("/tmp/"))));

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
                                        debug!("Get request: {}", payload);
                                        let path = payload["path"].as_str().unwrap();
                                        let path = Path::new(path);
                                        let result = {
                                            let filesystem = filesystem.lock().unwrap();
                                            filesystem.open(path).cloned()
                                        };
                                        let result = match result {
                                            Err(e) => {
                                                (ResponseActionType::Error, json!(format!("{}", e)))
                                            },
                                            Ok(inode) => {
                                                match inode.serialize_metadata() {
                                                    Err(e) => {
                                                        (ResponseActionType::Error, json!(format!("{}", e)))
                                                    },
                                                    Ok(metadata) => {
                                                        (ResponseActionType::Ok, metadata)
                                                    }
                                                }
                                            }
                                        };
                                        if let Err(e) = framed.send(convert_response_tuple(result)).await {
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
