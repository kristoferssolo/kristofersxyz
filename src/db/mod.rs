pub type DbPool = sqlx::SqlitePool;

pub async fn connect(database_url: &str) -> Result<DbPool, sqlx::Error> {
    todo!()
}
