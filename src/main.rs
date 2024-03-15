use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::{routing::get, Router};
use collabobot::appstate::AppState;
use collabobot::prelude::*;
use collabobot::{db, projects};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
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

    let (projects_route, projects_router) = projects::sub_router();

    let router = Router::new()
        .route("/", get(|| async { Redirect::permanent(projects_route) }))
        .nest(projects_route, projects_router)
        .layer(tracing_layer)
        .fallback(handler_404)
        .with_state(appstate);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let () = axum_server::bind(addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
