use axum::extract::State;
use axum::{routing::get, Router};
use color_eyre::eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Root;
use surrealdb::opt::IntoResource;
use surrealdb::sql::Datetime;
use surrealdb::{engine::any::connect, Surreal};
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
async fn apply_migrations(db: &Surreal<Any>) -> Result<()> {
    use surrealdb_migrations::MigrationRunner;

    // Apply all migrations
    MigrationRunner::new(&db)
        .up()
        .await
        .context("failed to apply migrations")?;
    event!(Level::INFO, "migrations applied");

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Project {
    project_name: String,
    created_at: Datetime,
}

#[derive(Clone)]
struct AppState {
    pub db: Surreal<Any>,
}

impl AppState {
    async fn initialize() -> Result<Self> {
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
        Ok(Self { db })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let () = setup_logging()?;

    let appstate = Arc::new(AppState::initialize().await?);
    let () = log_and_panic_if_error(apply_migrations(&appstate.db).await);

    let app = Router::new().route("/", get(handler)).with_state(appstate);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let () = axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn handler(State(appstate): State<Arc<AppState>>) -> &'static str {
    let projects = appstate.db.select::<Vec<Project>>("projects");

    "Hello, world!"
}
