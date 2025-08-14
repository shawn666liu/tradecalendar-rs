use anyhow::{Result, anyhow};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::postgres::PgPoolOptions;

use crate::Tradingday;

/// connection string example:
/// "postgres://username:password@localhost:port/dbname"
/// "mysql://username:password@localhost:port/dbname"

/// query: 5 fields required, keep the order of feilds,
/// select date,morning,trading,night,next from your_table where date>'yyyy-mm-dd' order by date
pub fn load_tradingdays_from_sqlx(conn_string: &str, query: &str) -> Result<Vec<Tradingday>> {
    let lower = conn_string.to_lowercase();

    if lower.starts_with("postgres") {
        let rt = tokio::runtime::Runtime::new()?;
        let res = rt.block_on(async {
            let pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(conn_string)
                .await?;
            let trading_days: Vec<Tradingday> = sqlx::query_as::<_, Tradingday>(query)
                .fetch_all(&pool)
                .await?;
            Ok::<Vec<Tradingday>, anyhow::Error>(trading_days)
        })?;
        return Ok(res);
    } else if lower.starts_with("mysql") {
        let rt = tokio::runtime::Runtime::new()?;
        let res: Vec<Tradingday> = rt.block_on(async {
            let pool = MySqlPoolOptions::new()
                .max_connections(1)
                .connect(conn_string)
                .await?;
            let trading_days: Vec<Tradingday> = sqlx::query_as::<_, Tradingday>(query)
                .fetch_all(&pool)
                .await?;
            Ok::<Vec<Tradingday>, anyhow::Error>(trading_days)
        })?;
        return Ok(res);
    } else {
        return Err(anyhow!(
            "unsupported connection string for sqlx: {}",
            conn_string
        ));
    }
}
