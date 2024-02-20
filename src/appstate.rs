use surrealdb::engine::any::Any;
use surrealdb::Surreal;

use crate::db;
use crate::prelude::*;

#[derive(Clone)]
pub struct AppState {
    pub db: Surreal<Any>,
}

impl AppState {
    pub async fn initialize() -> Result<Self> {
        let db = db::logon_as_root().await?;
        Ok(Self { db })
    }
}
