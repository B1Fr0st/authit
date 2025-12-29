use crate::db::DbPool;
use crate::types::requests::{AuthHeaders, LoginResponse};
use crate::types::data::Login;
use std::sync::Arc;

pub struct LoginData;

impl LoginData {
    pub async fn log_login(
        pool: &DbPool,
        time: u64,
        auth: &AuthHeaders,
        response: LoginResponse,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let response_str = match response {
            LoginResponse::Ok => "Ok",
            LoginResponse::InvalidLicense => "InvalidLicense",
            LoginResponse::HWIDMismatch => "HWIDMismatch",
            LoginResponse::LicenseExpired => "LicenseExpired",
            LoginResponse::LicenseFrozen => "LicenseFrozen",
            LoginResponse::MissingHeaders => "MissingHeaders",
        };

        client
            .execute(
                "INSERT INTO login_logs (license_key, time, hwid, response)
                 VALUES ($1, $2, $3, $4)",
                &[&auth.license, &(time as i64), &auth.hwid, &response_str],
            )
            .await?;

        Ok(())
    }

    pub async fn get_logs_by_license(
        pool: &DbPool,
        license_key: &str,
        limit: i64,
    ) -> Result<Vec<Login>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT time, hwid, response
                 FROM login_logs
                 WHERE license_key = $1
                 ORDER BY time DESC
                 LIMIT $2",
                &[&license_key, &limit],
            )
            .await?;

        Ok(rows.iter().map(|row| {
            let time: i64 = row.get(0);
            let hwid: String = row.get(1);
            let response_str: String = row.get(2);

            let response = Self::parse_response(&response_str);

            Login {
                license: Arc::from(license_key),
                time: time as u64,
                hwid: Arc::from(hwid),
                response,
            }
        }).collect())
    }

    pub async fn get_recent_logs(
        pool: &DbPool,
        limit: i64,
    ) -> Result<Vec<Login>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT license_key, time, hwid, response
                 FROM login_logs
                 ORDER BY time DESC
                 LIMIT $1",
                &[&limit],
            )
            .await?;

        Ok(rows.iter().map(|row| {
            let license_key: String = row.get(0);
            let time: i64 = row.get(1);
            let hwid: String = row.get(2);
            let response_str: String = row.get(3);

            let response = Self::parse_response(&response_str);

            Login {
                license: Arc::from(license_key),
                time: time as u64,
                hwid: Arc::from(hwid),
                response,
            }
        }).collect())
    }

    // Helper function to parse response string back to enum
    fn parse_response(s: &str) -> LoginResponse {
        match s {
            "Ok" => LoginResponse::Ok,
            "InvalidLicense" => LoginResponse::InvalidLicense,
            "HWIDMismatch" => LoginResponse::HWIDMismatch,
            "LicenseExpired" => LoginResponse::LicenseExpired,
            "LicenseFrozen" => LoginResponse::LicenseFrozen,
            "MissingHeaders" => LoginResponse::MissingHeaders,
            _ => LoginResponse::InvalidLicense, // Default fallback
        }
    }

    pub async fn delete_logs_by_license(
        pool: &DbPool,
        license_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        client
            .execute(
                "DELETE FROM login_logs WHERE license_key = $1",
                &[&license_key],
            )
            .await?;

        Ok(())
    }
}
