use axum::{
    extract::{Query, State},
    response::{sse::{Event, Sse}, IntoResponse, Json},
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::interval;
use tracing::{error, info};
use crate::{
    error::ApiError,
    models::AppState,
};

#[derive(Deserialize)]
pub struct LogsQuery {
    pub container: Option<String>,
    pub lines: Option<usize>,
}

#[derive(Serialize)]
pub struct LogsResponse {
    pub container: String,
    pub logs: Vec<String>,
}

pub async fn get_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LogsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let container = query.container.unwrap_or_else(|| "proton-vpn".to_string());
    let lines = query.lines.unwrap_or(100);
    
    info!("Fetching logs for container: {}", container);
    
    let logs = state.docker.get_container_logs(&container, lines).await?;
    
    let response = LogsResponse {
        container,
        logs,
    };
    
    Ok(Json(response))
}

pub async fn stream_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LogsQuery>,
) -> Sse<impl Stream<Item = Result<Event, ApiError>>> {
    let container = query.container.unwrap_or_else(|| "proton-vpn".to_string());
    
    info!("Starting log stream for container: {}", container);
    
    let stream = LogStream::new(state.clone(), container);
    
    Sse::new(stream)
}

struct LogStream {
    state: Arc<AppState>,
    container: String,
    interval: tokio::time::Interval,
    current_logs: Vec<String>,
    log_index: usize,
}

impl LogStream {
    fn new(state: Arc<AppState>, container: String) -> Self {
        Self {
            state,
            container,
            interval: interval(Duration::from_secs(1)),
            current_logs: Vec::new(),
            log_index: 0,
        }
    }
}

impl Stream for LogStream {
    type Item = Result<Event, ApiError>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.interval.poll_tick(cx) {
            Poll::Ready(_) => {
                let logs = match tokio::runtime::Handle::current()
                    .block_on(self.state.docker.get_container_logs(&self.container, 50)) {
                    Ok(logs) => logs,
                    Err(e) => {
                        error!("Failed to fetch logs: {}", e);
                        return Poll::Ready(Some(Err(e)));
                    }
                };
                
                if !logs.is_empty() {
                    let event = Event::default()
                        .data(serde_json::to_string(&logs).unwrap_or_default());
                    return Poll::Ready(Some(Ok(event)));
                }
                
                Poll::Pending
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
