use std::sync::Arc;

use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Datetime;

use crate::appstate::AppState;

mod routes {
    pub const SUB_DOMAIN: &str = "/projects";
    pub const ROOT: &str = "/";
    pub const NEW: &str = "/new";
    pub const SHOW: &str = "/:id";
}

mod handlers {
    use crate::prelude::*;
    use std::{future::IntoFuture, sync::Arc};

    use axum::{
        body::Body,
        extract::{Path, Request, State},
        http::StatusCode,
        response::{Html, IntoResponse, Redirect, Response},
        Form,
    };

    use crate::appstate::AppState;

    use super::{routes, Project, ProjectForm};

    pub async fn get_new_project_form(
        State(_appstate): State<Arc<AppState>>,
        request: Request,
    ) -> (StatusCode, Html<String>) {
        // info!("GET new-message");
        (
        StatusCode::OK,
        format!(
            " <form action=\"{}/new\" method=\"POST\" enctype=\"application/x-www-form-urlencoded\"><label for=\"project_name\">enter your project name</label><input type=text name=project_name></input></form>",
            routes::SUB_DOMAIN
        )
        .into(),
    )
    }

    pub async fn get_project(
        State(appstate): State<Arc<AppState>>,
        Path(id): Path<String>,
    ) -> AppResult<(StatusCode, Html<String>)> {
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

    pub async fn post_project(
        State(appstate): State<Arc<AppState>>,
        Form(project_form): Form<ProjectForm>,
    ) -> AppResult<Response<Body>> {
        // info!("Got a post request to handle a new project");
        let projects: Vec<Project> = appstate.db.create("projects").content(project_form).await?;
        info!("Trying to create a project");
        let response = match projects.first() {
            Some(project) => {
                info!("Created a project with id: '{}'", &project.id);
                Redirect::to(&project.route_to()).into_response()
            }
            None => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("<p>No project was created</p>"),
            )
                .into_response(),
        };
        Ok(response)
    }

    pub async fn get_projects(
        State(appstate): State<Arc<AppState>>,
    ) -> AppResult<(StatusCode, Html<String>)> {
        let projects = appstate
            .db
            .select::<Vec<Project>>("projects")
            .into_future()
            .await?;
        let inner_html = projects
            .into_iter()
            .map(|project| project.as_list_item())
            .collect::<Vec<_>>()
            .join("\n");
        Ok((StatusCode::OK, format!("<ul>{inner_html}</ul>").into()))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub project_name: String,
    pub created_at: Datetime,
    pub id: surrealdb::sql::Thing,
}

impl Project {
    pub fn raw_id(&self) -> String {
        self.id.id.to_raw()
    }
    pub fn route_to(&self) -> String {
        format!("{}/{}", routes::SUB_DOMAIN, self.raw_id())
    }

    pub fn as_list_item(&self) -> String {
        format!(
            "<li><a href=\"{}\">{}</a></li>",
            self.route_to(),
            self.project_name
        )
    }
}

pub fn sub_router() -> (&'static str, Router<Arc<AppState>>) {
    let router = Router::new()
        .route(routes::ROOT, get(handlers::get_projects))
        .route(
            routes::NEW,
            get(handlers::get_new_project_form).post(handlers::post_project),
        )
        .route(routes::SHOW, get(handlers::get_project));
    (routes::SUB_DOMAIN, router)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectForm {
    project_name: String,
}
