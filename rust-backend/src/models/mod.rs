use crate::services::{config::Config, docker::DockerService};
use std::sync::Arc;

pub struct AppState {
    pub config: Config,
    pub docker: Arc<DockerService>,
}
