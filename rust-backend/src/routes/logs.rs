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
    /// Frontend uses "limit", backend used "lines" - accept both
    pub lines: Option<usize>,
    pub limit: Option<usize>,
    pub since: Option<String>,
}

/// Log entry matching frontend's LogEntry interface
#[derive(Serialize, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub source: String,
}

/// Parse a raw log line into a structured LogEntry
fn parse_log_line(line: &str, source: &str) -> LogEntry {
    // Try to parse common log formats
    // Format: [2026-02-22 12:05:54] message
    // Format: 2026/02/22 12:05:54 message
    let (timestamp, level, message) = if line.starts_with('[') {
        if let Some(end) = line.find(']') {
            let ts = &line[1..end];
            let rest = line[end + 1..].trim();
            let (lvl, msg) = if rest.starts_with("ERROR") {
                ("error", rest.trim_start_matches("ERROR").trim_start_matches(':').trim())
            } else if rest.starts_with("WARN") {
                ("warn", rest.trim_start_matches("WARN").trim_start_matches(':').trim())
            } else {
                ("info", rest)
            };
            (ts.to_string(), lvl.to_string(), msg.to_string())
        } else {
            (chrono::Utc::now().to_rfc3339(), "info".to_string(), line.to_string())
        }
    } else if line.len() > 19 && line.chars().nth(4) == Some('/') {
        // 2026/02/22 12:05:54 message
        let ts = &line[..19];
        let rest = line[19..].trim();
        let (lvl, msg) = if rest.contains("ERROR") || rest.contains("error") {
            ("error", rest)
        } else if rest.contains("WARN") || rest.contains("warn") {
            ("warn", rest)
        } else {
            ("info", rest)
        };
        (ts.to_string(), lvl.to_string(), msg.to_string())
    } else {
        (chrono::Utc::now().to_rfc3339(), "info".to_string(), line.to_string())
    };

    LogEntry {
        timestamp,
        level,
        message,
        source: source.to_string(),
    }
}

pub async fn get_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LogsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let container = query.container.unwrap_or_else(|| crate::services::docker::CONTAINER_NAME.to_string());
    let lines = query.limit.or(query.lines).unwrap_or(100);

    info!("Fetching logs for container: {}, lines: {}", container, lines);

    let raw_logs = state.docker.get_container_logs(&container, lines).await?;

    let entries: Vec<LogEntry> = raw_logs
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| parse_log_line(l, &container))
        .collect();

    // Frontend expects LogEntry[] directly (not wrapped in ApiResponse)
    Ok(Json(entries))
}

pub async fn stream_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LogsQuery>,
) -> Sse<impl Stream<Item = Result<Event, ApiError>>> {
    let container = query.container.unwrap_or_else(|| crate::services::docker::CONTAINER_NAME.to_string());

    info!("Starting log stream for container: {}", container);

    let stream = LogStream::new(state.clone(), container);

    Sse::new(stream)
}

struct LogStream {
    state: Arc<AppState>,
    container: String,
    interval: tokio::time::Interval,
}

impl LogStream {
    fn new(state: Arc<AppState>, container: String) -> Self {
        Self {
            state,
            container,
            interval: interval(Duration::from_secs(1)),
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
                    let entries: Vec<LogEntry> = logs
                        .iter()
                        .filter(|l| !l.trim().is_empty())
                        .map(|l| parse_log_line(l, &self.container))
                        .collect();
                    let event = Event::default()
                        .data(serde_json::to_string(&entries).unwrap_or_default());
                    return Poll::Ready(Some(Ok(event)));
                }

                Poll::Pending
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
