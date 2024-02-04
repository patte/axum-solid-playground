use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct DB {
    pub pool: SqlitePool,
}

#[derive(sqlx::FromRow, Debug)]
pub struct User {
    //pub id: sqlx::types::Uuid,
    //pub id: uuid::adapter::Hyphenated,
    pub id: String,
    pub username: String,
}
impl User {
    pub fn new(username: String) -> Self {
        Self {
            //id: uuid::Uuid::new_v4(),
            id: uuid::Uuid::new_v4().to_string(),
            username,
        }
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct Authenticator {
    passkey: String,
    user_id: uuid::Uuid,
}

impl DB {
    pub async fn new() -> Self {
        let db_url = std::env::var("DATABASE_URL").unwrap();

        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            println!("Creating database {}", db_url);
            match Sqlite::create_database(&db_url).await {
                Ok(_) => println!("Database created successfully"),
                Err(error) => panic!("Error creating database: {}", error),
            }
        }

        let pool = SqlitePool::connect(&db_url).await.unwrap();

        let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let migrations = std::path::Path::new(&crate_dir).join("./migrations");

        info!("Running migrations from {:?}...", migrations);
        let migration_results = sqlx::migrate::Migrator::new(migrations)
            .await
            .unwrap()
            .run(&pool)
            .await;

        match migration_results {
            Ok(_) => println!("Migrations ran successfully"),
            Err(error) => {
                panic!("Error running migrations: {}", error);
            }
        }
        Self { pool }
    }

    // TODO use query_as!

    pub async fn get_users(&self) -> Result<Vec<User>, sqlx::Error> {
        Ok(
            sqlx::query_as::<_, User>(r#"SELECT id, username FROM users;"#)
                //sqlx::query_as!(
                //    User,
                //    r#"SELECT id AS "id: uuid::Uuid", username FROM users;"#
                //)
                .fetch_all(&self.pool)
                .await?,
        )
    }
}
