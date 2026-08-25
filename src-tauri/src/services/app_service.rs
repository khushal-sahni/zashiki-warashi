use std::sync::Arc;

use crate::domain::AppStatus;
use crate::error::AppError;
use crate::repositories::Database;

pub struct AppService {
    database: Arc<Database>,
    app_data_dir: String,
}

impl AppService {
    pub fn new(database: Arc<Database>, app_data_dir: String) -> Self {
        Self {
            database,
            app_data_dir,
        }
    }

    pub fn get_status(&self) -> Result<AppStatus, AppError> {
        self.database.ping()?;
        let schema_version = self.database.schema_version()?;

        Ok(AppStatus {
            name: "Zashiki Warashi".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database_ready: true,
            app_data_dir: self.app_data_dir.clone(),
            schema_version,
        })
    }
}
