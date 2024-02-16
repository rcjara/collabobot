use axum::error_handling::HandleError;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::routing::{any, post};
use axum::Form;
use axum::{routing::get, Router};
use color_eyre::eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::future::IntoFuture;
use std::net::SocketAddr;
use std::sync::Arc;
use surrealdb::engine::any::{connect, Any};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Datetime;
use surrealdb::Surreal;
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
    id: surrealdb::sql::Thing,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectForm {
    project_name: String,
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
            password: "root",
        })
        .await
        .wrap_err("unable to sign in")?;

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
    //let error_handler = HandleError::new(handler, handle_eyre_error);

    let app = Router::new()
        .route("/", get(handler))
        .route(
            "/new-project",
            get(new_project_form).post(handle_new_project),
        )
        .route("/hi", get(hi_handler))
        .fallback(handler_404)
        .with_state(appstate);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let () = axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

async fn new_project_form(
    State(_appstate): State<Arc<AppState>>,
    request: Request,
) -> (StatusCode, Html<String>) {
    event!(Level::INFO, "GET new-message");
    (
        StatusCode::OK,
        format!(
            "<p>request: {:?}</p><form action=\"/new-project\" method=\"POST\" enctype=\"application/x-www-form-urlencoded\"><input type=text name=project_name></input></form>",

            request
        )
        .into(),
    )
}

async fn handle_eyre_error(err: eyre::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Something went wrong: {err}"),
    )
}

async fn hi_handler(State(_appstate): State<Arc<AppState>>) -> Html<&'static str> {
    Html("<h3>Hi!</h3>")
}

async fn handle_new_project(
    State(appstate): State<Arc<AppState>>,
    Form(project_form): Form<ProjectForm>,
) -> (StatusCode, Html<String>) {
    event!(Level::INFO, "Got a post request to handle a new project");
    let project: std::result::Result<Vec<Project>, _> =
        appstate.db.create("projects").content(project_form).await;
    match project {
        Ok(projects) => {
            event!(Level::INFO, "WTF are these projects: {:?}", projects);
            match projects.first() {
                Some(project) => (
                    StatusCode::CREATED,
                    format!(
                        "<h1> You created a project: {}</h1><p>created at: {}</p>",
                        project.project_name, project.created_at
                    )
                    .into(),
                ),
                None => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("<p>No project was created</p>").into(),
                ),
            }
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("<p>There was an error: {:?}</p>", error).into(),
        ),
    }
}

async fn handler(State(appstate): State<Arc<AppState>>) -> (StatusCode, Html<String>) {
    event!(Level::INFO, "Loading the root of the app");
    let projects = appstate
        .db
        .select::<Vec<Project>>("projects")
        .into_future()
        .await;
    match projects {
        Ok(projects) => {
            let string = projects
                .into_iter()
                .map(|project| format!("<li>{} &#8212 {}</li>", project.id, project.project_name))
                .collect::<Vec<_>>()
                .join("\n");
            (StatusCode::OK, format!("<ul>{string}</ul>").into())
        }
        Err(error) => {
            event!(Level::ERROR, "error {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("<p>There was an error: {:?}</p>", error).into(),
            )
        }
    }
}
