pub mod calendar_helper;
mod db_helper;
pub mod jcswitch;
mod tradecalendar;

use std::path::Path;

use anyhow::Result;
use db_helper::load_tradingdays;
use jcswitch::MyDateType;

pub use crate::tradecalendar::*;

#[cfg(test)]
mod tests;

#[cfg(all(feature = "with-chrono", feature = "with-jiff"))]
compile_error!("features \"with-chrono\" and \"with-jiff\" cannot be enabled at the same time");

/// use buildin csv file to load date list.
///
/// 使用内置的csv文件构造交易日历, 可以指定开始日期，因为很多时候不用从2009年那么早开始
pub fn get_buildin_calendar(start_date: Option<MyDateType>) -> Result<TradeCalendar> {
    let csv_str = include_str!("../calendar.csv");
    let mut full_list = Tradingday::load_csv_read(csv_str.as_bytes())?;
    if let Some(start) = start_date {
        let (left, mid, _) = search_days(&full_list, &start);
        let index = if mid >= 0 { mid } else { left };
        if index > 0 {
            full_list.drain(0..index as usize);
        }
    }
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
    if let Some(start) = start_date {
        let (left, mid, _) = search_days(&full_list, &start);
        let index = if mid >= 0 { mid } else { left };
        if index > 0 {
            full_list.drain(0..index as usize);
        }
    }
    let mut calendar = TradeCalendar::new();
    calendar.reload(full_list)?;
    Ok(calendar)
}

/// 首先通过数据库查询,若失败则通过csv文件获取,最后使用内置的数据
///
/// 如果数据库查询有效,则不使用start_date过滤,因为query里面可以直接过滤
pub fn get_calendar<P: AsRef<Path>>(
    db_conn: &str,
    query: &str,
    proto: Option<String>,
    csv_file: Option<P>,
    start_date: Option<MyDateType>,
) -> Result<TradeCalendar> {
    let result = load_tradingdays(db_conn, query, proto);
    if let Ok(vec) = result {
        if !vec.is_empty() {
            let mut calendar = TradeCalendar::new();
            calendar.reload(vec).unwrap();
            return Ok(calendar);
        }
    }
    if let Some(file) = csv_file {
        let result = get_csv_calendar(file, start_date);
        if let Ok(res) = result {
            return Ok(res);
        }
    }
    get_buildin_calendar(start_date)
}
