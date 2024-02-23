use serde::{Deserialize, Serialize};
use surrealdb::sql::Datetime;

pub mod routes {
    pub const root: &str = "projects";
    pub const new: &str = "projects/new";
    pub const show: &str = "projects/:id";
}

pub mod handlers {
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

    use super::{Project, ProjectForm};

    pub async fn get_new_project_form(
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
        format!("{}/{}", routes::root, self.raw_id())
    }

    pub fn as_list_item(&self) -> String {
        format!(
            "<li><a href=\"{}\">{}</a></li>",
            self.route_to(),
            self.project_name
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectForm {
    project_name: String,
}
