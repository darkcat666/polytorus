use std::io::prelude::*;
use std::net::TcpStream;
use std::error::Error;
use crate::infrastructure::protocol::messages::{bytes_to_cmd, Message};
use crate::infrastructure::network::server::Server;

pub fn handle_connection(server: &Server, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer)?;

    // 受信データからコマンドと残りのデータに分割する
    let (cmd, data) = bytes_to_cmd(&buffer)?;
    match cmd.as_str() {
        "addr" => {
            // 例: アドレスメッセージの処理
            let nodes: Vec<String> = bincode::deserialize(data)?;
            server.handle_addr(nodes)?;
        },
        "block" => {
            let block_msg: crate::infrastructure::protocol::messages::Blockmsg = bincode::deserialize(data)?;
            server.handle_block(block_msg)?;
        },
        "inv" => {
            let inv_msg: crate::infrastructure::protocol::messages::Invmsg = bincode::deserialize(data)?;
            server.handle_inv(inv_msg)?;
        },
        "getblocks" => {
            let getblocks_msg: crate::infrastructure::protocol::messages::GetBlocksmsg = bincode::deserialize(data)?;
            server.handle_get_blocks(getblocks_msg)?;
        },
        "getdata" => {
            let getdata_msg: crate::infrastructure::protocol::messages::GetDatamsg = bincode::deserialize(data)?;
            server.handle_get_data(getdata_msg)?;
        },
        "tx" => {
            let tx_msg: crate::infrastructure::protocol::messages::Txmsg = bincode::deserialize(data)?;
            server.handle_tx(tx_msg)?;
        },
        "version" => {
            let version_msg: crate::infrastructure::protocol::messages::Versionmsg = bincode::deserialize(data)?;
            server.handle_version(version_msg)?;
        },
        _ => {
            eprintln!("Unknown command: {}", cmd);
        }
    }
    Ok(())
}