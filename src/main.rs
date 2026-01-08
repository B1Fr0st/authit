use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use tracing::{Level, error, info};
use tracing_subscriber;

mod handlers;
mod auth;
use crate::handlers::*;


pub struct AppState {
    db_pool: sqlx::PgPool,
    redis_client: redis::Client,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing subscriber to print to stdout
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL env var not set")).await {
            Ok(pool) => {
            info!("Connected to database @ {}", std::env::var("DATABASE_URL").unwrap()); //unwrap is safe due to expect earlier
            pool
        }
        Err(err) => {
            error!("Failed to connect to database: {}", err);
            std::process::exit(1);
        }
    };

    // Connect to Redis
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = match redis::Client::open(redis_url.as_str()) {
        Ok(client) => {
            // Test connection
            match client.get_multiplexed_async_connection().await {
                Ok(_) => {
                    info!("Connected to Redis @ {}", redis_url);
                    client
                }
                Err(err) => {
                    error!("Failed to connect to Redis: {}", err);
                    std::process::exit(1);
                }
            }
        }
        Err(err) => {
            error!("Failed to create Redis client: {}", err);
            std::process::exit(1);
        }
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db_pool: pool.clone(),
                redis_client: redis_client.clone()
            }))
            .service(
                web::scope("/api/v1")
                    .route("/health-check", web::get().to(public::health_check))
                    .route("/auth", web::post().to(public::auth))
                    .service(web::scope("/account")
                        .route("/login", web::post().to(account::login))
                        .route("/redeem", web::post().to(account::redeem))
                        .route("/set-role", web::post().to(account::set_role))
                        .route("/products", web::get().to(account::products))
                    )
                    .service(web::scope("/product")
                        .route("/generate-key", web::post().to(product::generate_key))
                        .route("/compensate", web::post().to(product::compensate))
                    )
            )
    })
    .bind(("0.0.0.0", 5593))?
    .run()
    .await
}