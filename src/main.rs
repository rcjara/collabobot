use axum::body::Body;
use axum::error_handling::HandleError;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{any, post};
use axum::Form;
use axum::{routing::get, Router};
use collabobot::appstate::AppState;
use collabobot::db;
use collabobot::internal_service_error::AppError;
use collabobot::prelude::*;
use serde::{Deserialize, Serialize};
use std::future::IntoFuture;
use std::net::SocketAddr;
use std::sync::Arc;
use surrealdb::engine::any::{connect, Any};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Datetime;
use surrealdb::Surreal;
use tower_http::LatencyUnit;

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
        //.json()
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

use tower::{Service, ServiceBuilder, ServiceExt};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
fn tracing_layer() -> impl tower::Layer<()> {}

#[tokio::main]
async fn main() -> Result<()> {
    let () = setup_logging()?;

    let appstate = Arc::new(AppState::initialize().await?);
    let () = log_and_panic_if_error(db::apply_migrations(&appstate.db).await);
    //let error_handler = HandleError::new(handler, handle_eyre_error);
    //
    let tracing_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().include_headers(true))
        .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
        .on_response(
            DefaultOnResponse::new()
                .level(tracing::Level::INFO)
                .latency_unit(LatencyUnit::Micros),
        );

    let app = Router::new()
        .route("/", get(handler))
        .route(
            "/new-project",
            get(new_project_form).post(handle_new_project),
        )
        .route("/hi", get(hi_handler))
        .route("/projects/:id", get(project))
        .layer(tracing_layer)
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
    // info!("GET new-message");
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

async fn project(
    State(appstate): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Html<String>)> {
    let project: Option<Project> = appstate.db.select(("projects", id.clone())).await?;
    let response = match project {
        Some(project) => (
            StatusCode::OK,
            format!("<h1>{}</h1>", project.project_name).into(),
        ),
        None => (
            StatusCode::NOT_FOUND,
            format!("no projects matching \"{}\"", id).into(),
        ),
    };
    Ok(response)
}

async fn handle_new_project(
    State(appstate): State<Arc<AppState>>,
    Form(project_form): Form<ProjectForm>,
) -> Result<Response<Body>> {
    // info!("Got a post request to handle a new project");
    let projects: Vec<Project> = appstate.db.create("projects").content(project_form).await?;
    info!("Trying to create a project");
    let response = match projects.first() {
        Some(project) => {
            info!("Created a project with id: '{}'", &project.id);
            Redirect::to(&format!("/projects/{}", project.id.id.to_raw())).into_response()
        }
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("<p>No project was created</p>"),
        )
            .into_response(),
    };
    Ok(response)
}

async fn handler(State(appstate): State<Arc<AppState>>) -> Result<(StatusCode, Html<String>)> {
    // info!("Loading the root of the app");
    let projects = appstate
        .db
        .select::<Vec<Project>>("projects")
        .into_future()
        .await?;
    let inner_html = projects
        .into_iter()
        .map(|project| {
            let id = project.id.id.to_raw();
            format!(
                "<li><a href=\"projects/{}\">{}</a></li>",
                id, project.project_name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok((StatusCode::OK, format!("<ul>{inner_html}</ul>").into()))
}
