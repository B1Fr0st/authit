use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;

pub mod license;
pub mod product;
pub mod session;

pub type DbPool = Pool;

pub fn create_pool(database_url: &str) -> Result<DbPool, Box<dyn std::error::Error>> {
    let mut cfg = Config::new();

    // Parse the database URL
    // Format: postgres://user:password@host:port/database
    let url = database_url.trim_start_matches("postgres://");

    if let Some((credentials, rest)) = url.split_once('@') {
        if let Some((user, password)) = credentials.split_once(':') {
            cfg.user = Some(user.to_string());
            cfg.password = Some(password.to_string());
        }

        if let Some((host_port, dbname)) = rest.split_once('/') {
            if let Some((host, port)) = host_port.split_once(':') {
                cfg.host = Some(host.to_string());
                if let Ok(port_num) = port.parse::<u16>() {
                    cfg.port = Some(port_num);
                }
            } else {
                cfg.host = Some(host_port.to_string());
            }
            cfg.dbname = Some(dbname.to_string());
        }
    }

    Ok(cfg.create_pool(Some(Runtime::Tokio1), NoTls)?)
}

pub async fn init_db(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    // Create products table
    client.execute(
        "CREATE TABLE IF NOT EXISTS products (
            id VARCHAR PRIMARY KEY,
            frozen BOOLEAN NOT NULL DEFAULT FALSE,
            frozen_at BIGINT NOT NULL DEFAULT 0
        )",
        &[],
    ).await?;

    // Create licenses table
    client.execute(
        "CREATE TABLE IF NOT EXISTS licenses (
            license_key VARCHAR PRIMARY KEY,
            hwid VARCHAR NOT NULL
        )",
        &[],
    ).await?;

    // Create license_products table (owned by licenses - one-to-many)
    // Each license owns its product subscriptions with individual time/started_at values
    client.execute(
        "CREATE TABLE IF NOT EXISTS license_products (
            license_key VARCHAR NOT NULL REFERENCES licenses(license_key) ON DELETE CASCADE,
            product_id VARCHAR NOT NULL REFERENCES products(id) ON DELETE CASCADE,
            time BIGINT NOT NULL,
            started_at BIGINT NOT NULL,
            PRIMARY KEY (license_key, product_id)
        )",
        &[],
    ).await?;

    // Create sessions table
    client.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id SERIAL PRIMARY KEY,
            license_key VARCHAR NOT NULL REFERENCES licenses(license_key) ON DELETE CASCADE,
            started BIGINT NOT NULL,
            ended BIGINT
        )",
        &[],
    ).await?;

    // Create login_logs table
    client.execute(
        "CREATE TABLE IF NOT EXISTS login_logs (
            id SERIAL PRIMARY KEY,
            license_key VARCHAR NOT NULL,
            time BIGINT NOT NULL,
            hwid VARCHAR NOT NULL,
            response VARCHAR NOT NULL
        )",
        &[],
    ).await?;

    Ok(())
}
