use std::process::exit;

use log::{error, info};
use serde_json::json;
use tokio::net::UnixStream;

use buhao_lib::{BuhaoCodec, ResponseActionType};
use futures::prelude::*;
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() {
    // init logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let stream = UnixStream::connect("/tmp/buhao.sock").await.unwrap();
    let (mut writer, mut reader) = Framed::new(stream, BuhaoCodec).split();

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        let (command, args) = input.split_once(' ').unwrap_or((input, ""));
        let sent_success = match command {
            "exit" => {
                exit(0);
            }
            "get" => {
                let path = args;
                let payload = json!({
                    "path": path,
                });
                if let Err(e) = writer.send((1, payload)).await {
                    error!("Failed to send payload: {}", e);
                    false
                } else {
                    true
                }
            }
            "refresh" => {
                unimplemented!("refresh")
            }
            _ => {
                error!("Unknown command: {}", command);
                false
            }
        };
        if sent_success {
            let response = match reader.next().await {
                Some(Err(e)) => {
                    error!("Failed to receive response: {}", e);
                    continue;
                }
                None => {
                    error!("Connection closed");
                    continue;
                }
                Some(Ok(response)) => response,
            };
            let typ = response.0;
            let payload = response.1;
            let typ: ResponseActionType = match typ.try_into() {
                Err(e) => {
                    error!("Unknown response type: {}", e);
                    continue;
                }
                Ok(typ) => typ,
            };
            info!("Response: {:?}, {:?}", typ, payload);
        }
    }
}
