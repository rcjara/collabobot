use axum::{routing::get, Router};
use color_eyre::eyre::{ErrReport, Result, WrapErr};
use std::{error::Error, net::SocketAddr};
use surrealdb::engine::any::connect;
use surrealdb::opt::auth::Root;
use tracing::{event, instrument, Level};

fn log_and_panic_if_error(result: Result<()>) -> () {
    match result {
        Ok(()) => (),
        Err(err) => {
            event!(Level::ERROR, "{:?}", err);
            panic!("{:?}", err);
        }
    }
}

fn setup_logging() -> Result<()> {
    //use std::fs::File;
    //let now = time::OffsetDateTime::now_utc();
    //let filename =
    //    time_fmt::format::format_offset_date_time("collabobot_%Y-%m-%d_%H:%M:%S.json", now)?;
    //let log_file = File::create(filename)?;

    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_writer(std::io::stderr)
        .with_max_level(Level::DEBUG)
        .finish();

    let () = tracing::subscriber::set_global_default(subscriber)?;
    let () = color_eyre::install()?;

    Ok(())
}

#[instrument]
async fn apply_migrations() -> Result<()> {
    use surrealdb_migrations::MigrationRunner;
    event!(Level::DEBUG, "connecting to db");
    let db = connect("ws://localhost:8000").await?;
    event!(Level::DEBUG, "connected to db");

    event!(Level::DEBUG, "signing into db");
    // Signin as a namespace, database, or root user
    db.signin(Root {
        username: "root",
        password: "rot",
    })
    .await
    .wrap_err("unable to sign in for migrations")?;

    event!(Level::DEBUG, "signed into db");

    // Select a specific namespace / database
    db.use_ns("namespace").use_db("database").await?;

    // Apply all migrations
    MigrationRunner::new(&db)
        .up()
        .await
        .context("failed to apply migrations")?;
    event!(Level::INFO, "migrations applied");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let () = setup_logging()?;
    let () = log_and_panic_if_error(apply_migrations().await);

    // Route all requests on "/" endpoint to anonymous handler.
    //
    // A handler is an async function which returns something that implements
    // `axum::response::IntoResponse`.

    // A closure or a function can be used as handler.
    //
    //

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
