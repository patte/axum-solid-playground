use serde::{Deserialize, Serialize};
use tokio_rusqlite::{params, Connection, Result};
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

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

#[derive(Clone)]
pub struct Store {
    conn: Connection,
}

impl Store {
    pub async fn new(conn: Connection) -> Self {
        Self { conn }
    }

    pub async fn insert_user_and_passkey(&self, user: User, passkey: Passkey) -> Result<()> {
        self.conn
            .call(move |conn| {
                let tx = conn.transaction()?;

                tx.execute(
                    "INSERT INTO users (id, username) VALUES (?1, ?2)",
                    params![user.id, user.username],
                )?;

                tx.execute(
                    "INSERT INTO authenticators (user_id, passkey) VALUES (?1, ?2)",
                    params![user.id, serde_json::to_string(&passkey).unwrap()],
                )?;

                tx.commit().expect("failed to save user and authenticator");
                Ok(())
            })
            .await
    }

    pub async fn get_passkey_for_user_and_passkey_id(
        &self,
        user_id: Uuid,
        passkey_id: String,
    ) -> Result<Option<Passkey>> {
        let passkey = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "select passkey
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
            })
            .await;
        passkey
    }

    pub async fn update_passkey_for_user_and_passkey_id(
        &self,
        user_id: Uuid,
        passkey_id: String,
        counter: u32,
        backup_state: bool,
        backup_eligible: bool,
    ) -> Result<()> {
        self.conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "update authenticators
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
                ])?;
                Ok(())
            })
            .await
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>> {
        let users = self
            .conn
            .call(|conn| {
                let mut stmt = conn.prepare("SELECT id, username FROM users")?;
                let people = stmt
                    .query_map([], |row| {
                        Ok(User {
                            id: row.get(0)?,
                            username: row.get(1)?,
                        })
                    })?
                    .collect::<std::result::Result<Vec<User>, rusqlite::Error>>()?;

                Ok(people)
            })
            .await;
        users
    }

    pub async fn check_username_exists(&self, username: String) -> Result<bool> {
        let exists = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare("SELECT id FROM users WHERE username = ?1")?;
                let mut rows = stmt.query(params![username])?;
                let exists = rows.next()?.is_some();
                Ok(exists)
            })
            .await;
        exists
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<User> {
        let user = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare("SELECT id, username FROM users WHERE id = ?1")?;
                let user = stmt
                    .query_row(params![id], |row| {
                        Ok(User {
                            id: row.get(0)?,
                            username: row.get(1)?,
                        })
                    })
                    .expect("Failed to get user");
                Ok(user)
            })
            .await;
        user
    }
}
