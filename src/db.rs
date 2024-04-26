use crate::prelude::*;
use surrealdb::{
    engine::any::{connect, Any},
    opt::auth::Root,
    Surreal,
};

const uri: &str = "ws://localhost:8000";
const username: &str = "root";
const password: &str = "root";
const namespace: &str = "namespace";
const database: &str = "database";

// #[instrument]
// pub async fn apply_migrations(db: &Surreal<Any>) -> Result<()> {
//     use surrealdb_migrations::MigrationRunner;
//     let mr = MigrationRunner::new(&db);
//     info!("we have mr");
//
//     // Apply all migrations
//     mr.up().await.context("failed to apply migrations")?;
//     info!("migrations applied");
//
//     Ok(())
// }

pub(crate) async fn logon_as_root() -> Result<Surreal<Any>> {
    debug!("connecting to db");
    let db = connect(uri).await?;
    debug!("connected to db");

    debug!("signing into db");
    // Signin as a namespace, database, or root user
    db.signin(Root { username, password })
        .await
        .wrap_err("unable to sign in")?;

    debug!("signed into db");

    // Select a specific namespace / database
    db.use_ns(namespace).use_db(database).await?;
    info!("connected to surrealdb {namespace}:{database} as {username}");
    Ok(db)
}
