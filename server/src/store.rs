use serde::Serialize;
use tokio_rusqlite::{params, Connection, Result};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
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

pub struct Authenticator {
    passkey: String,
    user_id: uuid::Uuid,
}

#[derive(Clone)]
pub struct Store {
    conn: Connection,
}

impl Store {
    pub async fn new(conn: Connection) -> Self {
        Self { conn }
    }

    pub async fn insert_user(&self, user: User) -> Result<()> {
        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO users (id, username) VALUES (?1, ?2)",
                    params![user.id, user.username],
                )?;
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
}
