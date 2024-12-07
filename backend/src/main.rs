//! This module handles database connections, API routes, and core functionality.

// External dependencies
use neo4rs::*;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

// Internal modules
/// Employee data model and related functionality
mod models;
use models::Employee;

/// HTTP request handlers for API endpoints
mod handlers;
use handlers::{get_employees, create_employee};

/// Custom error handling
mod error;

/// Database schema generation utilities
mod schema_generator;
use schema_generator::generate_schema;

/// Application configuration management
mod config;
use config::Settings;

// Web framework dependencies
use axum::{
    Router,
    routing::{get, post},
    response::{Html, IntoResponse, Response},
    http::Uri,
};

// CORS middleware
use tower_http::cors::{CorsLayer, Any};

/// Serves the frontend HTML file
async fn serve_frontend() -> Html<String> {
    Html(include_str!("../../static/index.html").to_string())
}

/// Serves static CSS files
///
/// # Arguments
/// * `uri` - The request URI containing the path to the CSS file
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

/// Application entry point
#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();
    let settings = Settings::new().expect("Failed to load settings");

    // Configure and establish Neo4j database connection
    let config = ConfigBuilder::default()
        .uri(&settings.database.uri)
        .user(&settings.database.username)
        .password(&settings.database.password)
        .db("neo4j")
        .build()
        .expect("Failed to build Neo4j config");

    // Create thread-safe database connection
    let graph = Arc::new(
        Graph::connect(config)
            .await
            .map_err(|e| {
                println!("Failed to connect to database: {}", e);
                e
            })
            .expect("Failed to connect to Neo4j after multiple attempts")
    );
    let graph_clone = Arc::clone(&graph);

    // Initialize database with sample data
    initialize_database(graph_clone).await
        .expect("Failed to initialize database");

    // Configure CORS middleware
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Set up API routes and middleware
    let app = Router::new()
        .route("/", get(serve_frontend))
        .route("/api/employees", get(get_employees))
        .route("/api/employees", post(create_employee))
        .route("/css/*file", get(serve_static))
        .with_state(graph)
        .layer(cors);

    println!("Server running on http://localhost:3000");

    // Start the server
    let addr: SocketAddr = "0.0.0.0:3000".parse()
        .expect("Failed to parse address");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to start server");
}

/// Initializes the database with test data
///
/// # Arguments
/// * `graph` - Thread-safe reference to the Neo4j graph database
///
/// # Returns
/// * `Result<()>` - Success or error status
async fn initialize_database(graph: Arc<Graph>) -> Result<()> {
    // Clear existing data
    graph.run(query("MATCH (e:Employee) DELETE e")).await?;

    // Define sample employee data
    let employees = vec![
        Employee { employee_id: 1, name: String::from("Andy"),  salary: 25_000.0, region: String::from("South Wales") },
        Employee { employee_id: 2, name: String::from("Jayne"), salary: 35_000.0, region: String::from("South Wales") },
        Employee { employee_id: 3, name: String::from("Emily"), salary: 45_000.0, region: String::from("Scotland") },
        Employee { employee_id: 4, name: String::from("Tom"),   salary: 55_000.0, region: String::from("London") }
    ];

    // Insert each employee into the database
    for employee in &employees {
        let query = query(
            "CREATE (e:Employee {employee_id: $id, name: $name, salary: $salary, region: $region})"
        )
            .param("id", employee.employee_id)
            .param("name", &*employee.name)
            .param("salary", employee.salary)
            .param("region", &*employee.region);

        graph.run(query).await?;
        employee.print();
    }

    // Generate and save database schema diagram
    generate_schema(&graph).await?;

    Ok(())
}