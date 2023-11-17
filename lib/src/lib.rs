use anyhow::Result;
use serde::Serialize;
use std::{io, os::unix::prelude::MetadataExt};

use serde_json::{json, Value};
use tokio_util::{
    bytes::{Buf, BufMut},
    codec::{Decoder, Encoder},
};

pub mod syncframed;

pub const BUHAO_SOCK_PATH: &str = "/tmp/buhao.sock";

pub type InodeId = u64;
pub const INVALID_PARENT: InodeId = u64::MAX;

#[derive(Debug, Serialize, Clone)]
pub struct DirectoryItem {
    pub name: String,
    pub inode: InodeId,
}

#[derive(Debug, Serialize, Clone)]
pub struct DirectoryContents {
    pub parent: InodeId,
    pub children: Vec<DirectoryItem>,
}

#[derive(Debug, Serialize, Clone)]
pub enum Contents {
    File,
    Symlink(String),
    Directory(DirectoryContents),
}

#[derive(Debug, Serialize, Clone)]
pub struct Inode {
    pub id: InodeId,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub nlink: u64,
    pub atime: i64,
    pub mtime: i64,
    pub ctime: i64,
    pub contents: Contents,
}

impl Inode {
    pub fn new(metadata: std::fs::Metadata, contents: Contents) -> Self {
        Self {
            id: metadata.ino(),
            mode: metadata.mode(),
            uid: metadata.uid(),
            gid: metadata.gid(),
            nlink: metadata.nlink(),
            atime: metadata.atime(),
            mtime: metadata.mtime(),
            ctime: metadata.ctime(),
            contents,
        }
    }

    pub fn serialize_metadata(&self) -> Result<Value> {
        Ok(json!(&self))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RequestActionType {
    Refresh,
    Get,
}

impl TryFrom<u8> for RequestActionType {
    type Error = io::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RequestActionType::Refresh),
            1 => Ok(RequestActionType::Get),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid message type",
            )),
        }
    }
}

impl From<RequestActionType> for u8 {
    fn from(value: RequestActionType) -> Self {
        match value {
            RequestActionType::Refresh => 0,
            RequestActionType::Get => 1,
        }
    }
}

impl PartialEq<RequestActionType> for u8 {
    fn eq(&self, other: &RequestActionType) -> bool {
        *self == Into::<u8>::into(*other)
    }
}

pub fn convert_request_tuple(t: (RequestActionType, Value)) -> Item {
    let (action_type, payload) = t;
    (action_type.into(), payload)
}

#[derive(Debug, Clone, Copy)]
pub enum ResponseActionType {
    Ok,
    Error,
}

impl TryFrom<u8> for ResponseActionType {
    type Error = io::Error;

    fn try_from(value: u8) -> Result<Self, io::Error> {
        match value {
            0 => Ok(ResponseActionType::Ok),
            1 => Ok(ResponseActionType::Error),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid message type",
            )),
        }
    }
}

impl From<ResponseActionType> for u8 {
    fn from(value: ResponseActionType) -> Self {
        match value {
            ResponseActionType::Ok => 0,
            ResponseActionType::Error => 1,
        }
    }
}

impl PartialEq<ResponseActionType> for u8 {
    fn eq(&self, other: &ResponseActionType) -> bool {
        *self == Into::<u8>::into(*other)
    }
}

pub fn convert_response_tuple(t: (ResponseActionType, Value)) -> Item {
    let (action_type, payload) = t;
    (action_type.into(), payload)
}

/// Packet design for requests and responses (which works like <https://i3wm.org/docs/ipc.html>):
/// "buhao"<json payload len (u32)><message type (u8)><json payload>
pub struct BuhaoCodec;
pub type Item = (u8, Value);

impl Decoder for BuhaoCodec {
    type Item = crate::Item;
    type Error = io::Error;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 10 {
            // Data not enough to use
            return Ok(None);
        }
        // Check magic
        if src[0..5] != *b"buhao" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic"));
        }

        let payload_len = u32::from_be_bytes(src[5..9].try_into().unwrap());
        if src.len() < 10 + payload_len as usize {
            // Data not enough to use
            return Ok(None);
        }

        src.advance(5 + 4);

        let message_type = src.get_u8();
        let payload = src.split_to(payload_len as usize);
        let payload = serde_json::from_slice(&payload)?;

        Ok(Some((message_type, payload)))
    }
}

impl Encoder<Item> for BuhaoCodec {
    type Error = io::Error;

    fn encode(
        &mut self,
        item: Item,
        dst: &mut tokio_util::bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        let (message_type, payload) = item;
        let json_bytes = serde_json::to_vec(&payload)?;
        dst.reserve(10 + json_bytes.len());
        dst.put_slice(b"buhao");
        dst.put_u32(json_bytes.len() as u32);
        dst.put_u8(message_type);
        dst.extend_from_slice(&json_bytes);
        Ok(())
    }
}
