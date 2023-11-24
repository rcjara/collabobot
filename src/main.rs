use axum::{routing::get, Router};
use std::error::Error;
use std::net::SocketAddr;
use surrealdb::engine::any::connect;
use surrealdb::opt::auth::Root;
use surrealdb_migrations::MigrationRunner;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

async fn apply_migrations() -> Result<()> {
    let db = connect("ws://localhost:8000").await?;

    // Signin as a namespace, database, or root user
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;

    // Select a specific namespace / database
    db.use_ns("namespace").use_db("database").await?;

    // Apply all migrations
    MigrationRunner::new(&db)
        .up()
        .await
        .expect("Failed to apply migrations");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
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
