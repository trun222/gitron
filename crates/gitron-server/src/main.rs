mod auth;
mod file_store;
mod routes;
mod sse;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::Router;
use clap::Parser;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use gitron_core::cache::repo_state::RepoStateCache;
use gitron_core::watcher::manager::WatcherManager;

use crate::file_store::FileCredentialStore;
use crate::sse::SseBroadcaster;

/// Gitron web server — self-hosted Git GUI
#[derive(Parser, Debug)]
#[command(name = "gitron-server", version, about)]
struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value = "9417")]
    port: u16,

    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Authentication token (required if host != 127.0.0.1/localhost)
    #[arg(long)]
    token: Option<String>,

    /// Path to built frontend files
    #[arg(long)]
    frontend_dir: Option<PathBuf>,

    /// Auto-open a repository on startup
    #[arg(long)]
    repo: Option<String>,
}

/// Shared server state
pub struct ServerState {
    pub cache: RepoStateCache,
    pub broadcaster: Arc<SseBroadcaster>,
    pub watcher: Mutex<Option<WatcherManager>>,
    pub repo_path: Mutex<Option<PathBuf>>,
    pub poll_interval_ms: Mutex<u64>,
    pub auth_token: Option<String>,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    let cli = Cli::parse();

    // Warn if binding to non-localhost without a token
    if cli.host != "127.0.0.1" && cli.host != "localhost" && cli.token.is_none() {
        log::warn!("Binding to {} without --token. API is unauthenticated!", cli.host);
    }

    // Initialize credential store
    let cred_store = Arc::new(FileCredentialStore::new());
    gitron_core::credential::init(cred_store);

    let broadcaster = Arc::new(SseBroadcaster::new());

    let state = Arc::new(ServerState {
        cache: RepoStateCache::new(),
        broadcaster,
        watcher: Mutex::new(None),
        repo_path: Mutex::new(None),
        poll_interval_ms: Mutex::new(0),
        auth_token: cli.token.clone(),
    });

    // Build API routes
    let api_router = routes::api_router(state.clone());

    // Apply auth middleware if token is set
    let api_router = if cli.token.is_some() {
        api_router.layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ))
    } else {
        api_router
    };

    let mut app = Router::new()
        .nest("/api", api_router)
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any))
        .layer(TraceLayer::new_for_http());

    // Serve static frontend files if provided
    if let Some(frontend_dir) = &cli.frontend_dir {
        if frontend_dir.exists() {
            let index_path = frontend_dir.join("index.html");
            let serve_dir = ServeDir::new(frontend_dir)
                .not_found_service(ServeFile::new(&index_path));
            app = app.fallback_service(serve_dir);
            log::info!("Serving frontend from {}", frontend_dir.display());
        } else {
            log::warn!("Frontend directory {} does not exist", frontend_dir.display());
        }
    }

    let addr: SocketAddr = format!("{}:{}", cli.host, cli.port)
        .parse()
        .expect("Invalid host:port");

    log::info!("Gitron server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
