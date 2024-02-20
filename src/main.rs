use axum::error_handling::HandleError;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::routing::{any, post};
use axum::Form;
use axum::{routing::get, Router};
use collabobot::appstate::AppState;
use collabobot::db;
use collabobot::prelude::*;
use serde::{Deserialize, Serialize};
use std::future::IntoFuture;
use std::net::SocketAddr;
use std::sync::Arc;
use surrealdb::engine::any::{connect, Any};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Datetime;
use surrealdb::Surreal;

fn log_and_panic_if_error(result: Result<()>) -> () {
    match result {
        Ok(()) => (),
        Err(err) => {
            error!("{:?}", err);
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
        .with_max_level(tracing::Level::DEBUG)
        .finish();

    let () = tracing::subscriber::set_global_default(subscriber)?;
    let () = color_eyre::install()?;

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

/*
 use tower::{ServiceBuilder, ServiceExt, Service};
 use tower_http::trace::TraceLayer;
 ServiceBuilder::new()
    .layer(
        TraceLayer::new_for_http()
            .make_span_with(
                DefaultMakeSpan::new().include_headers(true)
            )
            .on_request(
                DefaultOnRequest::new().level(Level::INFO)
            )
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .latency_unit(LatencyUnit::Micros)
            )
            // on so on for `on_eos`, `on_body_chunk`, and `on_failure`
    )
*/

#[tokio::main]
async fn main() -> Result<()> {
    let () = setup_logging()?;

    let appstate = Arc::new(AppState::initialize().await?);
    let () = log_and_panic_if_error(db::apply_migrations(&appstate.db).await);
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
    info!("GET new-message");
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
    info!("Got a post request to handle a new project");
    let project: std::result::Result<Vec<Project>, _> =
        appstate.db.create("projects").content(project_form).await;
    match project {
        Ok(projects) => {
            info!("WTF are these projects: {:?}", projects);
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
    info!("Loading the root of the app");
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
            error!("error {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("<p>There was an error: {:?}</p>", error).into(),
            )
        }
    }
}
