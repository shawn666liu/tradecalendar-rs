#[cfg(test)]
mod tests {
    use crate::db_helper::load_tradingdays_from_db;
    use anyhow::Result;

    #[test]
    fn read_db() -> Result<()> {
        let query =
            "select date,morning,trading,night,next from calendar where date>='2024-01-01' limit 10";

        // let conn = "postgresql://admin:Intel%40123@192.168.9.122:5432/future_info";
        // let res = load_calendar(conn, query, None)?;

        // let conn = "postgres://readonly:readonly@192.168.9.122:9005/futuredb";

        // clickhouse使用mysql协议读取
        // let conn = "mysql://readonly:readonly@192.168.9.122:9004/futuredb";
        let conn = "mysql://readonly:readonly@192.168.100.208:9004/futuredb";
        let res = load_tradingdays_from_db(conn, query, Some("text".into()))?;

        for td in res.iter() {
            println!("{:?}", td);
        }
        Ok(())
    }
}
