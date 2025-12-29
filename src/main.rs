use poem::{Route, Server, get, post, listener::TcpListener, middleware::AddData, EndpointExt};

pub mod types;
pub mod handlers;
pub mod db;
pub mod data;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Get database URL from environment or use default
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/authit".to_string());

    // Create database connection pool
    let pool = db::create_pool(&database_url)
        .expect("Failed to create database pool");

    // Initialize database tables
    db::init_db(&pool)
        .await
        .expect("Failed to initialize database");

    println!("Server starting on http://0.0.0.0:3000");
    println!("Database connected and initialized");

    // Get API key from environment or use default (warn if using default)
    let api_key = std::env::var("API_KEY")
        .unwrap_or_else(|_| {
            println!("WARNING: Using default API key. Set API_KEY environment variable for production!");
            "default-insecure-key".to_string()
        });
    println!("API Key configured: {}", if api_key == "default-insecure-key" { "DEFAULT (insecure)" } else { "CUSTOM" });

    let app = Route::new()
        .nest(
            "/api/v1",
            Route::new()
                .nest(
                    "/public",
                    Route::new()
                        .at("/health", get(crate::handlers::public::health))
                        .at("/auth", get(crate::handlers::public::auth))
                        .at("/product", get(crate::handlers::public::product_handler))
                )
                .nest(
                    "/private",
                    Route::new()
                        .nest(
                            "/license",
                            Route::new()
                                .at("/generator", post(crate::handlers::private::license::generator))
                        )
                )
        )
        .with(AddData::new(pool));

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}