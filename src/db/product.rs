use crate::db::DbPool;
use crate::types::core::{Product, LicenseProduct};
use std::sync::Arc;

pub struct ProductDb;

impl ProductDb {
    pub async fn create(
        pool: &DbPool,
        id: &str,
        frozen: bool,
        frozen_at: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        client
            .execute(
                "INSERT INTO products (id, frozen, frozen_at) VALUES ($1, $2, $3)",
                &[&id, &frozen, &(frozen_at as i64)],
            )
            .await?;

        Ok(())
    }

    pub async fn get_by_id(
        pool: &DbPool,
        id: &str,
    ) -> Result<Option<Product>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let row = client
            .query_opt(
                "SELECT frozen, frozen_at FROM products WHERE id = $1",
                &[&id],
            )
            .await?;

        Ok(row.map(|r| {
            let frozen: bool = r.get(0);
            let frozen_at: i64 = r.get(1);
            Product {
                id: Arc::from(id),
                frozen,
                frozen_at: frozen_at as u64,
            }
        }))
    }

    pub async fn update_frozen(
        pool: &DbPool,
        id: &str,
        frozen: bool,
        frozen_at: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        client
            .execute(
                "UPDATE products SET frozen = $1, frozen_at = $2 WHERE id = $3",
                &[&frozen, &(frozen_at as i64), &id],
            )
            .await?;

        Ok(())
    }

    pub async fn delete(
        pool: &DbPool,
        id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        client
            .execute(
                "DELETE FROM products WHERE id = $1",
                &[&id],
            )
            .await?;

        Ok(())
    }

    pub async fn exists(
        pool: &DbPool,
        id: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT EXISTS(SELECT 1 FROM products WHERE id = $1)",
                &[&id],
            )
            .await?;

        Ok(row.get(0))
    }

    pub async fn add_to_license(
        pool: &DbPool,
        license_key: &str,
        product_id: &str,
        time: u64,
        started_at: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        client
            .execute(
                "INSERT INTO license_products (license_key, product_id, time, started_at)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (license_key, product_id)
                 DO UPDATE SET time = $3, started_at = $4",
                &[&license_key, &product_id, &(time as i64), &(started_at as i64)],
            )
            .await?;

        Ok(())
    }

    pub async fn get_license_products(
        pool: &DbPool,
        license_key: &str,
    ) -> Result<Vec<LicenseProduct>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT product_id, time, started_at
                 FROM license_products
                 WHERE license_key = $1",
                &[&license_key],
            )
            .await?;

        Ok(rows.iter().map(|row| {
            let product_id: String = row.get(0);
            let time: i64 = row.get(1);
            let started_at: i64 = row.get(2);
            LicenseProduct {
                product: Arc::from(product_id),
                time: time as u64,
                started_at: started_at as u64,
            }
        }).collect())
    }

    pub async fn remove_from_license(
        pool: &DbPool,
        license_key: &str,
        product_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        client
            .execute(
                "DELETE FROM license_products WHERE license_key = $1 AND product_id = $2",
                &[&license_key, &product_id],
            )
            .await?;

        Ok(())
    }
}
