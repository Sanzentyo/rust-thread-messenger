use serde::{Deserialize, Serialize};
use tokio::io::{self, AsyncReadExt, BufReader};

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseStatus {
    Ok,
    Failed,
    Invalid,
}

pub async fn read_json<'a, T>(reader: &mut BufReader<tokio::net::tcp::ReadHalf<'a>>) -> io::Result<T>
where
    T: for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    let mut buffer = Vec::new();
    let mut temp_buf = [0; 4096];  // より大きなバッファサイズ

    loop {
        let n = reader.read(&mut temp_buf).await?;
        if n == 0 {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"))?;
        }
        buffer.extend_from_slice(&temp_buf[..n]);
        
        // JSONの完全性をチェック
        match serde_json::from_slice::<T>(&buffer) {
            Ok(json) => return Ok(json),
            Err(_) => continue,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendCommand {
    pub command: String,
    pub user_name: String,
    pub timestamp: i64,
    pub args: Args,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Args {
    SendMsg(SendMsgArgs), // "send_msg"コマンド
    CheckMsg(CheckMsgArgs), // "check_msg"コマンド
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendMsgArgs {
    pub to: String,
    pub content: String,
    pub connected_id: i64, // ない場合は-1
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendMsgResponse {
    pub status: ResponseStatus,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckMsgArgs {
    pub max_msg: i64, // デフォルトは-1で無制限
    pub recursive: i64, // デフォルトは-1で無制限
    pub from_user_name: String, // 特定の相手からメッセージだけ表示、空文字なら全員表示
    pub since: i64, // タイムスタンプと同じ形式で、-1がデフォルトで指定なし
    pub until: i64, // タイムスタンプと同じ形式で、-1がデフォルトで指定なし
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckMsgResponse {
    pub status: ResponseStatus,
    pub timestamp: i64,
    pub msg: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub content: String,
    pub timestamp: i64,
    pub uuid: i64,
    pub children_msg: Vec<Message>,
}