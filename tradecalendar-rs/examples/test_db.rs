#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tradecalendar::*;

    #[test]
    fn read_odbc() -> Result<()> {
        let query = "SELECT date,morning,trading,night,next FROM calendar WHERE date>='2024-01-01' ORDER BY date limit 10";

        let conn = "Driver={PostgreSQL Unicode};Server=192.168.9.122;UID=readonly;PWD=readonly;PORT=5432;Database=future_info";
        let res = load_tradingdays_from_odbc(conn, query)?;

        for td in res.iter() {
            println!("{:?}", td);
        }
        Ok(())
    }

    #[test]
    fn read_clickhouse() -> Result<()> {
        let query = "SELECT date,morning,trading,night,next FROM futuredb.calendar WHERE date>'2024-01-01' ORDER BY date limit 10";

        let conn = "clickhouse://readonly:readonly@192.168.9.122:8123/futuredb?connect_timeout=45&receive_timeout=300";
        let res = load_tradingdays_from_clickhouse(conn, query, None)?;

        for td in res.iter() {
            println!("{:?}", td);
        }
        Ok(())
    }

    #[test]
    fn read_sqlx() -> Result<()> {
        let query = "SELECT date,morning,trading,night,next FROM calendar WHERE date>='2024-01-01' ORDER BY date limit 10";

        let conn = "postgres://readonly:readonly@192.168.9.122:5432/future_info?connect_timeout=45";
        let res = load_tradingdays_from_sqlx(conn, query)?;

        for td in res.iter() {
            println!("{:?}", td);
        }
        Ok(())
    }

    #[test]
    fn read_db() -> Result<()> {
        let query = "select date,morning,trading,night,next from calendar where date>='2024-01-01' limit 10";

        let conn = "postgresql://readonly:readonly@192.168.9.122:5432/future_info";
        let res = load_tradingdays_from_db(conn, query, None)?;

        for td in res.iter() {
            println!("{:?}", td);
        }
        Ok(())
    }

    #[test]
    fn test_get_calendar() -> Result<()> {
        let dburl = "clickhouse://readonly:readonly@192.168.9.122:8123/futuredb";
        let query = "SELECT date,morning,trading,night,next FROM futuredb.calendar ORDER BY date";
        let mgr = get_calendar(dburl, query, Some(""), None, None)?;

        println!(
            "get_calendar(), from {:?}, to {:?}",
            mgr.min_date(),
            mgr.max_date()
        );

        Ok(())
    }
}

fn main() {
    println!("Hello, world!");
}
