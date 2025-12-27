use crate::{db::DbPool, types::core::License};
use std::sync::Arc;

pub struct LicenseDb;

impl LicenseDb {
    pub async fn create(
        pool: &DbPool,
        license: License,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut client = pool.get().await?;

        // Start a transaction to ensure atomicity
        let transaction = client.transaction().await?;

        // Insert the license (convert Arc<str> to &str for ToSql)
        transaction
            .execute(
                "INSERT INTO licenses (license_key, hwid) VALUES ($1, $2)",
                &[&license.license_key.as_ref(), &license.hwid.as_ref()],
            )
            .await?;

        // Insert all owned license_products
        for product in &license.products {
            transaction
                .execute(
                    "INSERT INTO license_products (license_key, product_id, time, started_at)
                     VALUES ($1, $2, $3, $4)",
                    &[
                        &license.license_key.as_ref(),
                        &product.product.as_ref(),
                        &(product.time as i64),
                        &(product.started_at as i64),
                    ],
                )
                .await?;
        }

        // Commit the transaction
        transaction.commit().await?;

        Ok(())
    }

    pub async fn get_by_key(
        pool: &DbPool,
        license_key: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let row = client
            .query_opt(
                "SELECT hwid FROM licenses WHERE license_key = $1",
                &[&license_key],
            )
            .await?;

        Ok(row.map(|r| r.get(0)))
    }

    pub async fn update_hwid(
        pool: &DbPool,
        license_key: &str,
        hwid: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        client
            .execute(
                "UPDATE licenses SET hwid = $1 WHERE license_key = $2",
                &[&hwid, &license_key],
            )
            .await?;

        Ok(())
    }

    pub async fn delete(
        pool: &DbPool,
        license_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        client
            .execute(
                "DELETE FROM licenses WHERE license_key = $1",
                &[&license_key],
            )
            .await?;

        Ok(())
    }

    pub async fn exists(
        pool: &DbPool,
        license_key: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT EXISTS(SELECT 1 FROM licenses WHERE license_key = $1)",
                &[&license_key],
            )
            .await?;

        Ok(row.get(0))
    }

    pub async fn get(
        pool: &DbPool,
        license_key: &str,
    ) -> Result<Option<License>, Box<dyn std::error::Error>> {
        use crate::db::{product::ProductDb, session::SessionDb};

        // Get basic license info
        let hwid = Self::get_by_key(pool, license_key).await?;

        if hwid.is_none() {
            return Ok(None);
        }

        let hwid = hwid.unwrap();

        // Get owned products - returns LicenseProduct directly now
        let products = ProductDb::get_license_products(pool, license_key).await?;

        // Get sessions
        let sessions = SessionDb::get_by_license(pool, license_key).await?;

        Ok(Some(License {
            license_key: Arc::from(license_key),
            products,
            hwid: Arc::from(hwid),
            sessions,
        }))
    }
}
