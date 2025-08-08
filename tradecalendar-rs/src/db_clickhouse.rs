use anyhow::{Result, anyhow};
use clickhouse::Client;
use clickhouse::Row;
use serde::Deserialize;

use crate::Tradingday;
use crate::jcswitch::date_from_days_since_epoch;

#[allow(dead_code)]
#[derive(Row, Deserialize)]
struct TradingDayRow {
    pub date: u16,
    pub morning: bool,
    pub trading: bool,
    pub night: bool,
    pub next: u16,
}

fn to_tradingday(td: &TradingDayRow) -> Tradingday {
    Tradingday {
        date: date_from_days_since_epoch(td.date as i32),
        morning: td.morning,
        trading: td.trading,
        night: td.night,
        next: date_from_days_since_epoch(td.next as i32),
    }
}

/// load trading days from clickhouse database
///
/// conn -> clickhouse://user:passwd@localhost:8123/dbname
/// http client, so port must be 8123
/// query -> "SELECT ?fields FROM futuredb.calendar WHERE date>'yyyy-mm-dd' ORDER BY date"
/// https://github.com/ClickHouse/clickhouse-rs
pub fn load_tradingdays_from_clickhouse(conn: &str, query: &str) -> Result<Vec<Tradingday>> {
    // conn -> user:passwd@localhost:8123/dbname
    let connvec = conn
        .split("://")
        .last()
        .and_then(|s| Some(s.split('@').collect::<Vec<&str>>()))
        .ok_or_else(|| anyhow!("parse connection string failed"))?;
    // url_db -> ["localhost:8123", "dbname"]
    let url_db = connvec
        .last()
        .and_then(|s| Some(s.split("/").collect::<Vec<&str>>()))
        .ok_or_else(|| anyhow!("connection string has no url"))?;
    if url_db.len() != 2 {
        return Err(anyhow!("bad format for db url and database"));
    }

    let rt = tokio::runtime::Runtime::new()?;

    let res = rt.block_on(async {
        let mut client = Client::default()
            .with_url(format!("http://{}", url_db[0]))
            .with_database(url_db[1]);
        if connvec.len() > 1 {
            // may has user and password
            let up: Vec<_> = connvec[0].split(':').collect();
            if up.len() != 2 {
                return Err(anyhow!("bad format for db user and passwd"));
            }
            client = client.with_user(up[0]).with_password(up[1]);
        }
        let rows = client.query(query).fetch_all::<TradingDayRow>().await?;
        let result: Vec<_> = rows.iter().map(to_tradingday).collect();
        Ok(result)
    })?;

    return Ok(res);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_clickhouse() -> Result<()> {
        let query = "SELECT date,morning,trading,night,next FROM futuredb.calendar WHERE date>'2024-01-01' ORDER BY date limit 10";

        let conn = "clickhouse://readonly:readonly@192.168.9.122:8123/futuredb";
        let res = load_tradingdays_from_clickhouse(conn, query)?;

        for td in res.iter() {
            println!("{:?}", td);
        }
        Ok(())
    }
}
