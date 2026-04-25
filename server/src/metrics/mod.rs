use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use std::future::ready;
use axum::{routing::get, Router};
use tracing::info;

pub mod middleware;

pub fn init_prometheus() -> metrics_exporter_prometheus::PrometheusHandle {
    PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder")
}

pub async fn start_metrics_server(addr: SocketAddr, handle: metrics_exporter_prometheus::PrometheusHandle) {
    let app = Router::new().route("/metrics", get(move || ready(handle.render())));
    
    info!("Metrics server listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Record a worker task execution
pub fn record_worker_task(task_name: &'static str, success: bool, duration: std::time::Duration) {
    let status = if success { "success" } else { "error" };
    metrics::counter!("worker_tasks_total", "task" => task_name, "status" => status).increment(1);
    metrics::histogram!("worker_task_duration_seconds", "task" => task_name).record(duration.as_secs_f64());
    
    if !success {
        metrics::counter!("worker_errors_total", "task" => task_name).increment(1);
    }
}

/// Record a user login
pub fn record_login() {
    metrics::counter!("user_logins_total").increment(1);
}
