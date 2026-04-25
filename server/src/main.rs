mod api;
mod controller;
mod dto;
mod entities;
mod service;
mod worker;

use axum::http;
#[cfg(feature = "dotenv")]
use dotenv::dotenv;

use async_nats::jetstream;
use clap::{Parser, Subcommand};
use migration::{Migrator, MigratorTrait};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::process::exit;
use std::{net::SocketAddr, time::Duration};
use tokio::time::interval;
use tower_http::cors::{Any, CorsLayer};
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer, cookie};
use tracing::{Level, info, warn};
use tracing_subscriber::EnvFilter;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

use crate::scrape_article_worker::scrape_article_worker;
use crate::service::worker_service;
use crate::worker::{fetch_feeds_worker::fetch_feeds_task, scrape_article_worker};

use crate::service::auth_service::AuthService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub auth_service: Arc<AuthService>,
    pub frontend_url: String,
}

#[derive(Parser)]
#[command(name = "redy")]
#[command(about = "rss-redy", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    RunServer,
    RerunMl {
        #[arg(long)]
        missing_only: bool,
    },
    RunMlUuid {
        #[arg(long)]
        uuid: Uuid,
    },
    CalculateSentimentalAnalysisForUuid {
        #[arg(long)]
        missing_only: bool,
    },
    RecalculateSentimentalAnalysis {
        #[arg(long)]
        missing_only: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // LogTracer::init()?;

    tracing_subscriber::fmt()
        .with_file(false)
        .with_line_number(true)
        .with_env_filter(EnvFilter::new("html5ever=off,info"))
        .with_max_level(Level::INFO)
        .init();

    #[cfg(feature = "dotenv")]
    {
        warn!("Using dotenv, not recommended in production");
        dotenv().ok();
    }

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or("postgres://user:password@localhost:5432/my_app_db".to_string());
    let mut opt = ConnectOptions::new(db_url);

    opt.sqlx_slow_statements_logging_settings(
        tracing_log::log::LevelFilter::Warn,
        Duration::from_secs(2),
    );
    opt.sqlx_logging_level(tracing_log::log::LevelFilter::Debug);

    let db = Database::connect(opt).await?;
    let nats_url = std::env::var("NATS_URL").unwrap_or("nats://localhost:4222".to_string());
    let client = async_nats::connect(nats_url).await?;
    let js = jetstream::new(client);

    let cli = Cli::parse();

    Migrator::up(&db, None).await?;

    match cli.command {
        Commands::RunServer => {
            info!("starting server...")
        }
        Commands::RunMlUuid { uuid } => {
            warn!("reruning ml for: {}", uuid);
            worker_service::run_ml_for_uuid(&js, uuid).await;
            exit(0)
        }
        Commands::RecalculateSentimentalAnalysis { missing_only } => {
            warn!(
                "recalculating sentimental analysis, missing only: {}",
                missing_only
            );
            worker_service::calculate_sentimental_analysis(&db, &js, missing_only).await;
            exit(0)
        }
        Commands::RerunMl { missing_only } => {
            worker_service::run_ml(&db, &js, missing_only).await;
            exit(0);
        }
        Commands::CalculateSentimentalAnalysisForUuid { missing_only } => {
            worker_service::calculate_sentimental_analysis(&db, &js, missing_only).await;
            exit(0);
        }
    }

    let auth_service = AuthService::new(
        &std::env::var("OIDC_ISSUER").unwrap_or("https://accounts.google.com".to_string()),
        &std::env::var("OIDC_CLIENT_ID").unwrap_or("client_id".to_string()),
        &std::env::var("OIDC_CLIENT_SECRET").unwrap_or("client_secret".to_string()),
        &std::env::var("OIDC_REDIRECT_URL")
            .unwrap_or("http://localhost:8080/auth/callback".to_string()),
        db.clone(),
    )
    .await?;

    let (_router, _api) = controller::create_controller().split_for_parts();

    let cors = CorsLayer::new()
        .allow_origin([std::env::var("FRONTEND_URL")
            .unwrap_or("http://localhost:3000".to_owned())
            .parse()
            .unwrap()])
        .allow_credentials(true)
        .allow_methods([
            http::Method::CONNECT,
            http::Method::DELETE,
            http::Method::GET,
            http::Method::HEAD,
            http::Method::OPTIONS,
            http::Method::PATCH,
            http::Method::POST,
            http::Method::PUT,
            http::Method::TRACE,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]);

    let db_for_worker = db.clone();
    let js_worker = js.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(60));
        info!("Fetching feed articles every 1 minute...");
        loop {
            ticker.tick().await;
            if let Err(e) = fetch_feeds_task(&db_for_worker, &js_worker).await {
                eprintln!("Fetcher error: {:?}", e);
            }
        }
    });

    let db_for_worker2 = db.clone();
    tokio::spawn(async move {
        scrape_article_worker(&js, &db_for_worker2).await;
    });

    let state = AppState {
        db,
        auth_service: Arc::new(auth_service),
        frontend_url: std::env::var("FRONTEND_URL").unwrap_or("http://localhost:3000".to_string()),
    };

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(cookie::SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(cookie::time::Duration::seconds(120)));
    let api_router = controller::create_controller();

    let full_router = OpenApiRouter::new()
        .merge(api_router)
        .layer(session_layer)
        .layer(cors);

    let (router, api) = full_router.split_for_parts();

    let app = router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
