pub mod calendar_helper;
pub mod common;
mod db_helper;
mod tradecalendar;

use anyhow::Result;
use common::{make_date, MyDateType};

pub use crate::tradecalendar::*;

#[cfg(all(feature = "with-chrono", feature = "with-jiff"))]
compile_error!("features \"with-chrono\" and \"with-jiff\" cannot be enabled at the same time");

/// use build in csv file to load date list.
///
/// 使用内置的csv文件构造交易日历, 可以指定开始日期，因为很多时候不用从2009年那么早开始
pub fn get_buildin_calendar(buildin_start: Option<MyDateType>) -> Result<TradeCalendar> {
    let start = buildin_start.unwrap_or_else(|| make_date(2009, 1, 1));
    let csv_str = include_str!("../calendar.csv");
    let mut full_list = Tradingday::load_csv_read(csv_str.as_bytes())?;
    let (left, mid, _) = search_days(&full_list, &start);
    let index = if mid >= 0 { mid } else { left };
    if index > 0 {
        full_list.drain(0..index as usize);
    }
    let mut calendar = TradeCalendar::new();
    calendar.reload(full_list)?;
    return Ok(calendar);
}
