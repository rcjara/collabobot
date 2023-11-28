use axum::{routing::get, Router};
use eyre::{Result, WrapErr};
use std::net::SocketAddr;
use surrealdb::engine::any::connect;
use surrealdb::opt::auth::Root;
use surrealdb_migrations::MigrationRunner;
use tracing::{event, instrument, Level};

#[instrument]
async fn apply_migrations() -> Result<()> {
    event!(Level::DEBUG, "connecting to db");
    let db = connect("ws://localhost:8000").await?;
    event!(Level::DEBUG, "connected to db");

    event!(Level::DEBUG, "signing into db");
    // Signin as a namespace, database, or root user
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await
    .wrap_err("Failed to login to db")?;

    event!(Level::DEBUG, "signed into db");

    // Select a specific namespace / database
    db.use_ns("namespace").use_db("database").await?;

    event!(Level::DEBUG, "applying migrations");
    // Apply all migrations
    MigrationRunner::new(&db)
        .up()
        .await
        .wrap_err("Failed to apply migrations")?;
    event!(Level::INFO, "migrations applied");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt().finish();

    let () =
        tracing::subscriber::set_global_default(subscriber).wrap_err("Failed to setup logging")?;
    // Route all requests on "/" endpoint to anonymous handler.
    //
    // A handler is an async function which returns something that implements
    // `axum::response::IntoResponse`.

    // A closure or a function can be used as handler.
    //
    //
    let () = apply_migrations().await?;

    let app = Router::new().route("/", get(handler));

    // Address that server will bind to.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // Use `hyper::server::Server` which is re-exported through `axum::Server` to serve the app.
    let () = axum::Server::bind(&addr)
        // Hyper server takes a make service.
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn handler() -> &'static str {
    "Hello, world!"
}
