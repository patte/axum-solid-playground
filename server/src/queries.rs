use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

// Models and queries
// Intentionally using rusqlite and not tokio_rusqlite
// the async wrapping is done where the queries are called.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
}
impl User {
    pub fn new(username: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            username,
            created_at: Utc::now(),
        }
    }
}

pub fn insert_user(conn: &Connection, user: User) -> Result<usize> {
    conn.execute(
        "insert into
        users (id, username, created_at)
        values (?1, ?2, ?3)",
        params![user.id, user.username, user.created_at.to_rfc3339()],
    )
}

pub fn insert_authenticator(
    conn: &Connection,
    user_id: Uuid,
    passkey: Passkey,
    created_at: DateTime<Utc>,
    user_agent_short: &str,
) -> Result<usize> {
    conn.execute(
        "insert into
        authenticators (user_id, passkey, created_at, user_agent_short)
        values (?1, ?2, ?3, ?4)",
        params![
            user_id,
            serde_json::to_string(&passkey).unwrap(),
            created_at.to_rfc3339(),
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

    insert_authenticator(&tx, user.id, passkey, user.created_at, user_agent_short)?;

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
        select id, username, created_at
        from users
        where id = ?1",
    )?;
    let user = stmt.query_row(params![id], |row| {
        let created_at_string: String = row.get(2)?;
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            created_at: DateTime::parse_from_rfc3339(&created_at_string)
                .unwrap()
                .to_utc(),
        })
    })?;
    Ok(user)
}

#[allow(dead_code)]
pub fn get_all_users(conn: &Connection) -> Result<Vec<User>> {
    let mut stmt = conn.prepare("SELECT id, username, created_at FROM users")?;
    let users = stmt
        .query_map([], |row| {
            let created_at_string: String = row.get(2)?;
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                created_at: DateTime::parse_from_rfc3339(&created_at_string)
                    .unwrap()
                    .to_utc(),
            })
        })?
        .collect();
    users
}

#[derive(Debug, Clone, Serialize)]
pub struct Authenticator {
    pub user_id: Uuid,
    pub passkey: Passkey,
    pub user_agent_short: String,
    pub created_at: DateTime<Utc>,
}

pub fn get_authenticators_for_user_id(
    conn: &Connection,
    user_id: Uuid,
) -> Result<Vec<Authenticator>> {
    let mut stmt = conn.prepare(
        "
        select user_id, passkey, user_agent_short, created_at
        from authenticators
        where user_id = ?1",
    )?;
    let authenticators = stmt
        .query_map(params![user_id], |row| {
            let passkey_string: String = row.get(1)?;
            let created_at_string: String = row.get(3)?;
            Ok(Authenticator {
                user_id: row.get(0)?,
                passkey: serde_json::from_str(&passkey_string).unwrap(),
                user_agent_short: row.get(2)?,
                created_at: DateTime::parse_from_rfc3339(&created_at_string)
                    .unwrap()
                    .to_utc(),
            })
        })?
        .collect();
    authenticators
}
