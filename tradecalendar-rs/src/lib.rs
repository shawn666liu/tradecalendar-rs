mod db_clickhouse;
mod db_odbc;
mod db_sqlx;
pub mod jcswitch;
mod tests;
mod tradecalendar;

use anyhow::{Result, anyhow};
use std::path::Path;

pub use db_clickhouse::load_tradingdays_from_clickhouse;
pub use db_odbc::load_tradingdays_from_odbc;
pub use db_sqlx::load_tradingdays_from_sqlx;
use jcswitch::{MyDateType, get_now};

pub use tradecalendar::*;

#[cfg(all(feature = "with-chrono", feature = "with-jiff"))]
compile_error!("features \"with-chrono\" and \"with-jiff\" cannot be enabled at the same time");

/// 移除掉start_date之前的数据
pub fn drain_tday_list(full_list: &mut Vec<Tradingday>, start_date: Option<MyDateType>) {
    if let Some(start) = start_date {
        let (left, mid, _) = search_days(&full_list, &start);
        let index = if mid >= 0 { mid } else { left };
        if index > 0 {
            full_list.drain(0..index as usize);
        }
    }
}

/// use buildin csv file to load tradingday list.
pub fn load_tradingdays_buildin() -> Result<Vec<Tradingday>> {
    let csv_str = include_str!("../../calendar.csv");
    Tradingday::load_csv_read(csv_str.as_bytes())
}

/// 尝试从数据库, csv文件, 内置数据中加载交易日, 然后取最后日期最大的那个
pub fn load_latest_tradingdays<P: AsRef<Path>>(
    db_conn: &str,
    query: &str,
    csv_file: Option<P>,
) -> Result<Vec<Tradingday>> {
    // 内部函数
    fn _find_latest_(
        v1: Option<Vec<Tradingday>>,
        v2: Option<Vec<Tradingday>>,
    ) -> Option<Vec<Tradingday>> {
        match (v1, v2) {
            (None, None) => None,
            (None, Some(r2)) => Some(r2),
            (Some(r1), None) => Some(r1),
            (Some(r1), Some(r2)) => match (r1.last(), r2.last()) {
                (None, None) => None,
                (None, Some(_)) => Some(r2),
                (Some(_), None) => Some(r1),
                (Some(t1), Some(t2)) => {
                    if t1.date >= t2.date {
                        Some(r1)
                    } else {
                        Some(r2)
                    }
                }
            },
        }
    }
    let res1 = match load_tradingdays_from_db(db_conn, query) {
        Ok(r) => {
            println!(
                "==> {}, load_tradingdays_from_db() count {}, first {:?}, last {:?}",
                get_now(),
                r.len(),
                r.first(),
                r.last()
            );
            Some(r)
        }
        Err(e) => {
            println!("==> {}, load_tradingdays_from_db() error: {}", get_now(), e);
            None
        }
    };
    let res2 = csv_file.and_then(|f| match Tradingday::load_csv_file(f) {
        Ok(r) => {
            println!(
                "==> {}, load_csv_file() count {}, first {:?}, last {:?}",
                get_now(),
                r.len(),
                r.first(),
                r.last()
            );
            Some(r)
        }
        Err(e) => {
            println!("==> {}, load_csv_file() error: {}", get_now(), e);
            None
        }
    });
    let res3 = _find_latest_(res1, res2);
    let res4 = load_tradingdays_buildin().ok();
    _find_latest_(res3, res4).ok_or(anyhow!("no tradingdays loaded"))
}

/// 使用内置的csv文件构造交易日历, 可以指定开始日期，因为很多时候不用从2009年那么早开始
pub fn get_buildin_calendar(start_date: Option<MyDateType>) -> Result<TradeCalendar> {
    let mut full_list = load_tradingdays_buildin()?;
    drain_tday_list(&mut full_list, start_date);
    let mut calendar = TradeCalendar::new();
    calendar.reload(full_list)?;
    return Ok(calendar);
}

/// use external csv file to load date list.
///
/// 使用外部的csv文件构造交易日历, 可以指定开始日期，因为很多时候不用从2009年那么早开始
pub fn get_csv_calendar<P: AsRef<Path>>(
    csv_file: P,
    start_date: Option<MyDateType>,
) -> Result<TradeCalendar> {
    let mut full_list = Tradingday::load_csv_file(csv_file)?;
    drain_tday_list(&mut full_list, start_date);
    let mut calendar = TradeCalendar::new();
    calendar.reload(full_list)?;
    Ok(calendar)
}

/// 从数据库加载交易日并创建TradeCalendar对象
pub fn get_db_calendar(db_conn: &str, query: &str) -> Result<TradeCalendar> {
    let full_list = load_tradingdays_from_db(db_conn, query)?;
    let mut calendar = TradeCalendar::new();
    calendar.reload(full_list)?;
    Ok(calendar)
}

/// 尝试从数据库, csv文件, 内置数据中加载交易日, 然后取最后日期最大的那个
pub fn get_calendar<P: AsRef<Path>>(
    db_conn: &str,
    query: &str,
    csv_file: Option<P>,
    start_date: Option<MyDateType>,
) -> Result<TradeCalendar> {
    let mut vec = load_latest_tradingdays(db_conn, query, csv_file)?;
    if vec.is_empty() {
        return Err(anyhow!("tradingday list is empty"));
    }
    drain_tday_list(&mut vec, start_date);
    if !vec.is_empty() {
        let mut calendar = TradeCalendar::new();
        calendar.reload(vec)?;
        return Ok(calendar);
    }
    return Err(anyhow!(
        "tradingday list becomes empty after filter by `{:?}`",
        start_date
    ));
}

/// 对于需要长期运行的程序, 可以定期更新交易日历, 保持calender对象不销毁,
/// 只要维护数据库或者外部csv文件即可
pub fn reload_calendar<P: AsRef<Path>>(
    calendar: &mut TradeCalendar,
    db_conn: &str,
    query: &str,
    csv_file: Option<P>,
    start_date: Option<MyDateType>,
) -> Result<()> {
    let mut vec = load_latest_tradingdays(db_conn, query, csv_file)?;
    if vec.is_empty() {
        return Err(anyhow!("tradingday list is empty"));
    }
    drain_tday_list(&mut vec, start_date);
    if !vec.is_empty() {
        calendar.reload(vec)?;
        return Ok(());
    }
    return Err(anyhow!(
        "tradingday list becomes empty after filter by `{:?}`",
        start_date
    ));
}

/// load Tradingday from db
/// conn format:  
/// 1) postgres://user:passwd@localhost:5432/dbname
/// 2) mysql://user:passwd@localhost:3306/dbname
/// 3) clickhouse://user:passwd@localhost:8123/dbname
/// 4) odbc connection string, Driver={PostgreSQL Unicode};Server=localhost;PORT=5432;UID=username;PWD=password;Database=dbname
///
/// query: 5 fields required, keep the order of feilds,
/// select date,morning,trading,night,next from your_table where date>='yyyy-mm-dd' order by date
pub fn load_tradingdays_from_db(conn: &str, query: &str) -> Result<Vec<Tradingday>> {
    if conn.is_empty() || query.is_empty() {
        return Err(anyhow!("connection string or query is empty"));
    }
    let lower = conn.to_lowercase();
    if lower.starts_with("clickhouse://") {
        return load_tradingdays_from_clickhouse(conn, query);
    } else if lower.starts_with("mysql") || lower.starts_with("postgres") {
        return load_tradingdays_from_sqlx(conn, query);
    } else {
        load_tradingdays_from_odbc(conn, query)
    }
}
