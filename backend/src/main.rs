use neo4rs::*;
use std::net::SocketAddr;
use std::sync::Arc;
mod models;
mod handlers;
mod error;
mod schema_generator;
mod config;
mod db;

use handlers::{get_persons, create_person};
use config::Settings;
use db::neo4j::repository::Neo4jRepository;
use axum::{
    Router,
    routing::{get, post},
    response::{Html, IntoResponse, Response},
    http::Uri,
};
use tower_http::cors::{CorsLayer, Any};
use std::error::Error as StdError;
async fn serve_frontend() -> Html<String> {
    Html(include_str!("../../static/index.html").to_string())
}

async fn serve_static(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches("/css/");
    let content = tokio::fs::read_to_string(format!("css/{}", path))
        .await
        .unwrap_or_else(|_| String::from("/* File not found */"));

    Response::builder()
        .header("Content-Type", "text/css")
        .body(content)
        .unwrap()
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn StdError + Send + Sync + 'static>> {
    let matches = clap::Command::new("Genealogy DB")
        .arg(clap::Arg::new("import")
            .long("import")
            .value_name("FILE")
            .help("Import data from file"))
        .get_matches();

    if let Some(import_file) = matches.get_one::<String>("import") {
        let repo = Neo4jRepository::new("bolt://localhost:7687".to_string())
            .await
            .map_err(|e| Box::<dyn StdError + Send + Sync + 'static>::from(format!("{}", e)))?;
        repo.import_from_file(import_file)
            .await
            .map_err(|e| Box::<dyn StdError + Send + Sync + 'static>::from(format!("{}", e)))?;
        println!("Import completed successfully!");
        return Ok(());
    }

    dotenv::dotenv().ok();
    let settings = Settings::new()
        .map_err(|e| Box::<dyn StdError + Send + Sync + 'static>::from(format!("{}", e)))?;

    let config = ConfigBuilder::default()
        .uri("bolt://localhost:7687")
        .user("neo4j")
        .password("password")
        .db("neo4j")
        .build()
        .map_err(|e| Box::<dyn StdError + Send + Sync + 'static>::from(format!("{}", e)))?;

    let graph = Arc::new(
        Graph::connect(config)
            .await
            .map_err(|e| Box::<dyn StdError + Send + Sync + 'static>::from(format!("{}", e)))?
    );

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(serve_frontend))
        .route("/api/persons", get(get_persons))
        .route("/api/persons", post(create_person))
        .route("/css/*file", get(serve_static))
        .with_state(graph)
        .layer(cors);

    println!("Server running on http://localhost:3000");

    let addr: SocketAddr = "0.0.0.0:3000".parse()
        .map_err(|e| Box::<dyn StdError + Send + Sync + 'static>::from(format!("{}", e)))?;

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| Box::<dyn StdError + Send + Sync + 'static>::from(format!("{}", e)))?;

    Ok(())
}