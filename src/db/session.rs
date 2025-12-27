use crate::db::DbPool;
use crate::types::core::Session;

pub struct SessionDb;

impl SessionDb {
    pub async fn create(
        pool: &DbPool,
        license_key: &str,
        started: u64,
    ) -> Result<i32, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO sessions (license_key, started, ended)
                 VALUES ($1, $2, NULL)
                 RETURNING id",
                &[&license_key, &(started as i64)],
            )
            .await?;

        Ok(row.get(0))
    }

    pub async fn get_by_id(
        pool: &DbPool,
        id: i32,
    ) -> Result<Option<Session>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let row = client
            .query_opt(
                "SELECT started, ended FROM sessions WHERE id = $1",
                &[&id],
            )
            .await?;

        Ok(row.map(|r| {
            let started: i64 = r.get(0);
            let ended: Option<i64> = r.get(1);
            Session {
                started: started as u64,
                ended: ended.map(|e| e as u64),
            }
        }))
    }

    pub async fn get_by_license(
        pool: &DbPool,
        license_key: &str,
    ) -> Result<Vec<Session>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT started, ended FROM sessions WHERE license_key = $1 ORDER BY started DESC",
                &[&license_key],
            )
            .await?;

        Ok(rows.iter().map(|row| {
            let started: i64 = row.get(0);
            let ended: Option<i64> = row.get(1);
            Session {
                started: started as u64,
                ended: ended.map(|e| e as u64),
            }
        }).collect())
    }

    pub async fn end_session(
        pool: &DbPool,
        id: i32,
        ended: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        client
            .execute(
                "UPDATE sessions SET ended = $1 WHERE id = $2",
                &[&(ended as i64), &id],
            )
            .await?;

        Ok(())
    }

    pub async fn delete(
        pool: &DbPool,
        id: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        client
            .execute(
                "DELETE FROM sessions WHERE id = $1",
                &[&id],
            )
            .await?;

        Ok(())
    }

    pub async fn get_active_sessions(
        pool: &DbPool,
        license_key: &str,
    ) -> Result<Vec<Session>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT started, ended FROM sessions
                 WHERE license_key = $1 AND ended IS NULL
                 ORDER BY started DESC",
                &[&license_key],
            )
            .await?;

        Ok(rows.iter().map(|row| {
            let started: i64 = row.get(0);
            Session {
                started: started as u64,
                ended: None,
            }
        }).collect())
    }
}
