use async_trait::async_trait;
use cookie::time::OffsetDateTime;
use rusqlite::OptionalExtension;
use tokio_rusqlite::{params, Connection, Result as SqlResult};
use tower_sessions::{
    session::{Id, Record},
    session_store::{self, ExpiredDeletion},
    SessionStore,
};

// A tokio-rusqlite session store.
// Based on SqlxStore
// https://github.com/maxcountryman/tower-sessions-stores/tree/main/sqlx-store

#[derive(Clone, Debug)]
pub struct RusqliteStore {
    conn: Connection,
    table_name: String,
}

/// An error type for SQLx stores.
#[derive(thiserror::Error, Debug)]
pub enum RusqliteStoreError {
    /// A variant to map `rusqlite` errors.
    #[error(transparent)]
    SqlError(#[from] tokio_rusqlite::Error),

    /// A variant to map `rmp_serde` encode errors.
    #[error(transparent)]
    Encode(#[from] rmp_serde::encode::Error),

    /// A variant to map `rmp_serde` decode errors.
    #[error(transparent)]
    Decode(#[from] rmp_serde::decode::Error),
}

impl From<RusqliteStoreError> for session_store::Error {
    fn from(err: RusqliteStoreError) -> Self {
        match err {
            RusqliteStoreError::SqlError(inner) => session_store::Error::Backend(inner.to_string()),
            RusqliteStoreError::Decode(inner) => session_store::Error::Decode(inner.to_string()),
            RusqliteStoreError::Encode(inner) => session_store::Error::Encode(inner.to_string()),
        }
    }
}

impl RusqliteStore {
    /// Create a new SQLite store with the provided connection.
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            table_name: "tower_sessions".into(),
        }
    }

    /// Set the session table name with the provided name.
    #[allow(dead_code)]
    pub fn with_table_name(mut self, table_name: impl AsRef<str>) -> Result<Self, String> {
        let table_name = table_name.as_ref();
        if !is_valid_table_name(table_name) {
            return Err(format!(
                "Invalid table name '{}'. Table names must be alphanumeric and may contain \
                 hyphens or underscores.",
                table_name
            ));
        }

        self.table_name = table_name.to_owned();
        Ok(self)
    }

    /// Migrate the session schema.
    pub async fn migrate(&self) -> SqlResult<()> {
        let conn = self.conn.clone();
        let query = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {}
            (
                id TEXT PRIMARY KEY NOT NULL,
                data BLOB NOT NULL,
                expiry_date INTEGER NOT NULL
            )
            "#,
            self.table_name
        );
        conn.call(
            move |conn| conn.execute(&query, []).map_err(|e| e.into()), // Convert to tokio_rusqlite::Error
        )
        .await
        .map(|_| ())
    }
}

#[async_trait]
impl ExpiredDeletion for RusqliteStore {
    async fn delete_expired(&self) -> session_store::Result<()> {
        let conn = self.conn.clone();
        let query = format!(
            r#"
                DELETE FROM {}
                WHERE expiry_date < ?1
            "#,
            self.table_name
        );
        conn.call(
            move |conn| {
                conn.execute(&query, [OffsetDateTime::now_utc().unix_timestamp()])
                    .map_err(|e| e.into())
            }, // Convert to tokio_rusqlite::Error
        )
        .await
        .map_err(|e| {
            error!("Error deleting session: {:?}", e);
            RusqliteStoreError::SqlError(e).into()
        })
        .map(|_| ())
    }
}

#[async_trait]
impl SessionStore for RusqliteStore {
    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let conn = self.conn.clone();
        let table_name = self.table_name.clone();
        let record_id = record.id.to_string();
        let record_data = rmp_serde::to_vec(record).map_err(RusqliteStoreError::Encode)?;
        let record_expiry = record.expiry_date;

        conn.call(move |conn| {
            let query = format!(
                r#"
                    INSERT INTO {}
                        (id, data, expiry_date) VALUES (?1, ?2, ?3)
                        ON CONFLICT(id) DO UPDATE SET
                            data = excluded.data,
                            expiry_date = excluded.expiry_date
                "#,
                table_name
            );
            conn.execute(
                &query,
                params![record_id, record_data, record_expiry.unix_timestamp()],
            )
            .map_err(|e| e.into()) // Convert to tokio_rusqlite::Error
        })
        .await
        .map_err(|e| {
            error!("Error saving session: {:?}", e);
            RusqliteStoreError::SqlError(e).into()
        })
        .map(|_| {
            //info!("Session saved: {:?}", record);
            ()
        })
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let conn = self.conn.clone();
        let table_name = self.table_name.clone();
        let session_id_str = session_id.to_string();

        let data = conn
            .call(move |conn| {
                let query = format!(
                    r#"
                        SELECT data FROM {}
                        WHERE id = ?1 AND expiry_date > ?2
                    "#,
                    table_name
                );
                let mut stmt = conn.prepare(&query)?;
                stmt.query_row(
                    params![session_id_str, OffsetDateTime::now_utc().unix_timestamp()],
                    |row| {
                        let data: Vec<u8> = row.get(0)?;
                        Ok(data)
                    },
                )
                .optional()
                .map_err(|e| e.into()) // Convert to tokio_rusqlite::Error
            })
            .await
            .map_err(|e| {
                error!("Error loading session: {:?}", e);
                RusqliteStoreError::SqlError(e)
            })?;

        match data {
            Some(data) => {
                let record: Record =
                    rmp_serde::from_slice(&data).map_err(RusqliteStoreError::Decode)?;
                //info!("Session loaded: {:?}", record);
                Ok(Some(record))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let conn = self.conn.clone();
        let table_name = self.table_name.clone();
        let session_id_str = session_id.to_string();

        conn.call(move |conn| {
            let query = format!(
                r#"
                    DELETE FROM {}
                    WHERE id = ?1
                "#,
                table_name
            );
            conn.execute(&query, params![session_id_str])
                .map_err(|e| e.into()) // Convert to tokio_rusqlite::Error
        })
        .await
        .map_err(|e| {
            error!("Error deleting session: {:?}", e);
            RusqliteStoreError::SqlError(e).into()
        })
        .map(|_| {
            //info!("Session deleted: {:?}", v);
            ()
        })
    }
}

fn is_valid_table_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}
