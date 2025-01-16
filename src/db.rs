use anyhow::Result;
use crate::command::{
    CheckMsgArgs,
    SendMsgArgs,
};

#[derive(Debug, sqlx::FromRow)]
pub struct MsgDB {
    pub from_user: String,
    pub to_user: String,
    pub content: String,
    pub uuid: i64,
    pub connected_msg_uuid: i64,
    pub timestamp: i64,
}

const CREATE_TABLE: &str = "CREATE TABLE IF NOT EXISTS msg (
    from_user TEXT,
    to_user TEXT,
    content TEXT,
    uuid INTEGER PRIMARY KEY AUTOINCREMENT,
    connected_msg_uuid INTEGER,
    timestamp TIMESTAMP
)";

pub async fn create_table(pool: &sqlx::SqlitePool) -> Result<()> {
    sqlx::query(CREATE_TABLE).execute(pool).await?;
    Ok(())
}

pub async fn insert_msg(pool: &sqlx::SqlitePool, from_user: &str, args: &SendMsgArgs, timestamp: i64) -> Result<()> {
    let mut query_text = "INSERT INTO msg (".to_string();
    let mut values = "VALUES (".to_string();
    let mut params = vec![];

    if !from_user.is_empty() {
        query_text.push_str("from_user,");
        values.push_str("?,");
        params.push(from_user);
    }

    if !args.to.is_empty() {
        query_text.push_str("to_user,");
        values.push_str("?,");
        params.push(&args.to);
    }

    if !args.content.is_empty() {
        query_text.push_str("content,");
        values.push_str("?,");
        params.push(&args.content);
    }

    query_text.push_str("connected_msg_uuid,");
    values.push_str("?,");

    query_text.push_str("timestamp)");
    values.push_str("?)");

    let full_query = format!("{} {}", query_text, values);
    let mut query = sqlx::query(&full_query);

    for param in params {
        query = query.bind(param);
    }

    query = query.bind(args.connected_id);
    query = query.bind(timestamp);

    query.execute(pool).await?;
    Ok(())
}

// -1の場合は無視
pub async fn check_msg(pool: &sqlx::SqlitePool, args: &CheckMsgArgs) -> Result<Vec<MsgDB>> {
    let mut query_text = "SELECT * FROM msg WHERE 1 = 1".to_string();

    if !args.from_user_name.is_empty() {
        query_text.push_str(" AND from_user = ?");
    }
    if args.since != -1 {
        query_text.push_str(" AND timestamp >= ?");
    }
    if args.until != -1 {
        query_text.push_str(" AND timestamp <= ?");
    }
    if args.max_msg != -1 {
        query_text.push_str(" LIMIT ?");
    }

    let mut query = sqlx::query_as::<_, MsgDB>(&query_text);

    if !args.from_user_name.is_empty() {
        query = query.bind(&args.from_user_name);
    }
    if args.since != -1 {
        query = query.bind(args.since);
    }
    if args.until != -1 {
        query = query.bind(args.until);
    }
    if args.max_msg != -1 {
        query = query.bind(args.max_msg);
    }

    let messages = query.fetch_all(pool).await?;
    Ok(messages)
}