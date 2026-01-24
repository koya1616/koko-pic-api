use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, get_service},
    Router,
};
use async_graphql::{
    Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::response::Response;
use std::sync::Arc;
use tokio::signal;

// Define a simple Query object for GraphQL
struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self, ctx: &Context<'_>) -> String {
        "Hello, World!".to_string()
    }
}

// Define a simple schema type
type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create the GraphQL schema
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    // Create the Axum app
    let app = Router::new()
        // GraphQL endpoint
        .route("/graphql", get(graphql_handler).post(graphql_handler))
        // GraphQL playground
        .route("/playground", get(playground_handler))
        // Health check endpoint
        .route("/health", get(health_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .unwrap();
    
    println!("Server running on http://0.0.0.0:8000");
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    Ok(())
}

// GraphQL handler
async fn graphql_handler(
    State(schema): State<AppSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

// GraphQL playground handler
async fn playground_handler() -> Html<String> {
    Html(async_graphql::http::playground_source(
        async_graphql::http::PlaygroundConfig::new("/graphql"),
    ))
}

// Health check handler
async fn health_handler() -> StatusCode {
    StatusCode::OK
}

// Graceful shutdown signal
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("Received termination signal, shutting down gracefully...");
}