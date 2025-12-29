use poem::{Route, Server, get, post, put, delete, listener::TcpListener, middleware::AddData, EndpointExt};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod types;
pub mod handlers;
pub mod db;
pub mod data;
pub mod logging;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Initialize logging system
    let log_store = logging::init_log_store();

    // Set up tracing subscriber with both console output and in-memory storage
    let in_memory_layer = logging::InMemoryLayer::new(log_store);
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(in_memory_layer)
        .with(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("Starting Authit License Server");

    // Get database URL from environment or use default
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/authit".to_string());

    // Create database connection pool
    let pool = db::create_pool(&database_url)
        .expect("Failed to create database pool");

    tracing::info!("Database connection pool created");

    // Initialize database tables
    db::init_db(&pool)
        .await
        .expect("Failed to initialize database");

    tracing::info!("Database initialized successfully");

    // Get API key from environment or use default (warn if using default)
    let api_key = std::env::var("API_KEY")
        .unwrap_or_else(|_| {
            tracing::warn!("Using default API key - NOT SECURE! Set API_KEY environment variable for production!");
            "default-insecure-key".to_string()
        });

    if api_key == "default-insecure-key" {
        tracing::warn!("API Key: DEFAULT (insecure)");
    } else {
        tracing::info!("API Key: CUSTOM (configured)");
    }

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
                                .at("/add-product", put(crate::handlers::private::license::add_product))
                                .at("/delete-product", put(crate::handlers::private::license::delete_product))
                        )
                        .nest(
                            "/product",
                            Route::new()
                                .at("/freeze", put(crate::handlers::private::product::freeze))
                                .at("/unfreeze", put(crate::handlers::private::product::unfreeze))
                                .at("/create", put(crate::handlers::private::product::create))
                                .at("/delete", delete(crate::handlers::private::product::delete))
                        )
                        .nest(
                            "/data",
                            Route::new()
                                .at("/licenses", get(crate::handlers::private::data::get_licenses))
                                .at("/products", get(crate::handlers::private::data::get_products))
                                .at("/logins", get(crate::handlers::private::data::get_logins))
                                .at("/logs", get(crate::handlers::private::data::get_logs))
                        )
                )
        )
        .with(AddData::new(pool));

    tracing::info!("Server listening on http://0.0.0.0:3000");

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}