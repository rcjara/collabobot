use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{routing::get, Router};
use collabobot::appstate::AppState;
use collabobot::prelude::*;
use collabobot::{db, projects};
use std::net::SocketAddr;
use std::sync::Arc;
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
    let subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::DEBUG)
        .finish();

    let () = tracing::subscriber::set_global_default(subscriber)?;
    let () = color_eyre::install()?;

    Ok(())
}

use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};

#[tokio::main]
async fn main() -> Result<()> {
    let () = setup_logging()?;

    let appstate = Arc::new(AppState::initialize().await?);
    let () = log_and_panic_if_error(db::apply_migrations(&appstate.db).await);

    let tracing_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().include_headers(true))
        .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
        .on_response(
            DefaultOnResponse::new()
                .level(tracing::Level::INFO)
                .latency_unit(LatencyUnit::Micros),
        );

    let app = Router::new()
        .route("/", get(projects::handlers::get_projects))
        .route(
            projects::routes::root,
            get(projects::handlers::get_projects),
        )
        .route(
            projects::routes::new,
            get(projects::handlers::get_new_project_form).post(projects::handlers::post_project),
        )
        .route(projects::routes::show, get(projects::handlers::get_project))
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
