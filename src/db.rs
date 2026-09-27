use std::env;

static MIGRATIONS: toasty::migration::MigrationSet = toasty::embed_migrations!();

/// The default on-disk database, used when `DATABASE_URL` is not set.
const DEFAULT_DATABASE_URL: &str = "sqlite:f4y.db";

/// The connection string this process should use.
///
/// Reads `DATABASE_URL` so a different database can be pointed at without
/// touching the code, which is what the migration CLI and tests rely on.
pub fn database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_owned())
}

/// Open the database and apply pending schema migrations.
///
/// Cloning the returned handle is cheap: clones share one connection pool, so
/// the application registers a single value in the router and borrows it from
/// each request.
pub async fn connect() -> toasty::Db {
    let db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&database_url())
        .await
        .expect("failed to connect to the database");

    MIGRATIONS
        .apply(&db)
        .await
        .expect("failed to apply database migrations");

    db
}
