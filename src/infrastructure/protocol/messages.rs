use serde::{Deserialize, Serialize};

pub const CMD_LEN: usize = 12;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Addr(Vec<String>),
    Version(Versionmsg),
    Tx(Txmsg),
    GetData(GetDatamsg),
    GetBlock(GetBlockmsg),
    Inv(Invmsg),
    Block(Blockmsg),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blockmsg {
    pub addr_from: String,
    pub block: crate::domain::block::Block,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetBlockmsg {
    pub addr_from: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetDatamsg {
    pub addr_from: String,
    pub kind: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Invmsg {
    pub addr_from: String,
    pub kind: String,
    pub items: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Txmsg {
    pub addr_from: String,
    pub transaction: crate::domain::transaction::Transaction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Versionmsg {
    pub addr_from: String,
    pub version: i32,
    pub best_height: i32,
}

pub fn cmd_to_bytes(cmd: &str) -> [u8; CMD_LEN] {
    let mut bytes = [0; CMD_LEN];
    for (i, &b) in cmd.as_bytes().iter().enumerate() {
        bytes[i] = b;
    }
    bytes
}

pub fn bytes_to_cmd(bytes: &[u8]) -> Result<(String, &[u8]), Box<dyn std::error::Error>> {
    if bytes.len() < CMD_LEN {
        return Err("Invalid bytes length".into());
    }

    let cmd_bytes = &bytes[..CMD_LEN];
    let cmd = String::from_utf8(cmd_bytes.iter().cloned().filter(|&b| b != 0).collect())?;
    
    Ok((cmd, &bytes[CMD_LEN..]))
}