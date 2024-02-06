use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;
use rusqlite_migration::AsyncMigrations;
use tokio_rusqlite::Connection;

//use crate::store::Store;

#[derive(Clone)]
pub struct DB {
    pub conn: Connection,
    //pub store: Store,
}

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

// Define migrations. These are applied atomically.
lazy_static! {
    static ref MIGRATIONS: AsyncMigrations =
        AsyncMigrations::from_directory(&MIGRATIONS_DIR).unwrap();
}

impl DB {
    pub async fn new() -> Self {
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let db_path = db_url.split("://").collect::<Vec<&str>>()[1];

        let mut conn = Connection::open(&db_path).await.unwrap();

        conn.call(move |conn| {
            conn.execute_batch(
                "
                PRAGMA foreign_keys = ON;
                PRAGMA journal_mode = WAL;
                PRAGMA synchronous = NORMAL;
                ",
            )
            .map_err(|e| e.into())
        })
        .await
        .unwrap();

        // Update the database schema, atomically
        info!("Applying migrations...");
        MIGRATIONS
            .to_latest(&mut conn)
            .await
            .expect("Failed to apply migrations");

        info!("DB ready");

        //let store = Store::new(conn.clone()).await;
        Self { conn } //, store }
    }
}
