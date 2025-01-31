#[cfg(test)]
mod tests {
    use crate::db_helper::load_tradingdays_from_db;
    use crate::{get_calendar, TradingdayCache};
    use anyhow::Result;

    #[test]
    fn read_db() -> Result<()> {
        let query =
            "select date,morning,trading,night,next from calendar where date>='2024-01-01' limit 10";

        let conn = "postgresql://admin:Intel%40123@192.168.9.122:5432/future_info";
        let res = load_tradingdays_from_db(conn, query, None)?;

        // let conn = "postgres://readonly:readonly@192.168.9.122:9005/futuredb";

        // clickhouse使用mysql协议读取,低版本的clickhouse是支持的,ch升级之后不行了
        // let conn = "mysql://readonly:readonly@192.168.9.122:9004/futuredb";
        // let conn = "mysql://readonly:readonly@192.168.100.208:9004/futuredb";
        // let res = load_tradingdays_from_db(conn, query, Some("text".into()))?;

        for td in res.iter() {
            println!("{:?}", td);
        }
        Ok(())
    }

    #[test]
    fn read_clickhouse() -> Result<()> {
        let query =
            "SELECT ?fields FROM futuredb.calendar WHERE date>'2024-01-01' ORDER BY date limit 10";

        let conn = "clickhouse://readonly:readonly@192.168.9.122:8123/futuredb";
        let res = load_tradingdays_from_db(conn, query, None)?;

        for td in res.iter() {
            println!("{:?}", td);
        }
        Ok(())
    }

    #[test]
    fn test_get_calendar() -> Result<()> {
        let dburl = "clickhouse://readonly:readonly@192.168.9.122:8123/futuredb";
        let query = "SELECT ?fields FROM futuredb.calendar ORDER BY date";
        let mgr = get_calendar(dburl, query, None, Some(""), None)?;

        println!(
            "get_calendar(), from {:?}, to {:?}",
            mgr.min_date(),
            mgr.max_date()
        );

        Ok(())
    }
}
