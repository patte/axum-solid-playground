use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

// Models and queries
// Intentionally using rusqlite and not tokio_rusqlite
// the async wrapping is done where the queries are called.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    //pub id: String,
    pub id: Uuid,
    pub username: String,
}
impl User {
    pub fn new(username: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            username,
        }
    }
}

pub fn insert_user(conn: &Connection, user: User) -> Result<usize> {
    conn.execute(
        "insert into
        users (id, username)
        values (?1, ?2)",
        params![user.id, user.username],
    )
}

pub fn insert_authenticator(
    conn: &Connection,
    user_id: Uuid,
    passkey: Passkey,
    user_agent_short: &str,
) -> Result<usize> {
    conn.execute(
        "insert into
        authenticators (user_id, passkey, user_agent_short)
        values (?1, ?2, ?3)",
        params![
            user_id,
            serde_json::to_string(&passkey).unwrap(),
            user_agent_short
        ],
    )
}

pub fn insert_user_and_passkey(
    conn: &mut Connection,
    user: User,
    passkey: Passkey,
    user_agent_short: &str,
) -> Result<()> {
    let tx = conn.transaction()?;

    insert_user(&tx, user.clone())?;

    insert_authenticator(&tx, user.id, passkey, user_agent_short)?;

    tx.commit()?;
    Ok(())
}

pub fn check_username_exists(conn: &mut Connection, username: &str) -> Result<bool> {
    let mut stmt = conn.prepare(
        "
        select id
        from users
        where username = ?1
        ",
    )?;
    let mut rows = stmt.query(params![username])?;
    let exists = rows.next()?.is_some();
    Ok(exists)
}

pub fn get_passkey_for_user_and_passkey_id(
    conn: &Connection,
    user_id: Uuid,
    passkey_id: String,
) -> Result<Option<Passkey>> {
    let mut stmt = conn.prepare(
        "
        select passkey
        from authenticators
        where
            user_id = ?1 and
            json_extract(passkey, '$.cred.cred_id') = ?2",
    )?;
    let mut rows = stmt.query(params![user_id, passkey_id])?;
    let passkey = rows.next()?.map(|row| {
        let passkey: String = row.get(0).expect("Failed to get row");
        serde_json::from_str(&passkey).unwrap()
    });
    Ok(passkey)
}

pub fn update_passkey_for_user_and_passkey_id(
    conn: &Connection,
    user_id: Uuid,
    passkey_id: String,
    counter: u32,
    backup_state: bool,
    backup_eligible: bool,
) -> Result<usize> {
    let mut stmt = conn.prepare(
        "
        update authenticators
        set passkey = json_patch(
            passkey,
            ?3
        )
        where
            user_id = ?1 and
            json_extract(passkey, '$.cred.cred_id') = ?2",
    )?;
    stmt.execute(params![
        user_id,
        passkey_id,
        serde_json::json!({
            "cred": {
                "counter": counter,
                "backup_state": backup_state,
                "backup_eligible": backup_eligible
            }
        })
        .to_string()
    ])
}

pub fn get_user_by_id(conn: &Connection, id: Uuid) -> Result<User> {
    let mut stmt = conn.prepare(
        "
        select id, username
        from users
        where id = ?1",
    )?;
    let user = stmt.query_row(params![id], |row| {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
        })
    })?;
    Ok(user)
}

#[allow(dead_code)]
pub fn get_all_users(conn: &Connection) -> Result<Vec<User>> {
    let mut stmt = conn.prepare("SELECT id, username FROM users")?;
    let users = stmt
        .query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
            })
        })?
        .collect();
    users
}
