// use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

pub type MyDateType = NaiveDate;
pub type MyDateTimeType = NaiveDateTime;
pub type MyTimeType = NaiveTime;

pub fn make_date(year: i32, month: u32, day: u32) -> MyDateType {
    return NaiveDate::from_ymd_opt(year, month, day).expect("from_ymd_opt() failed");
}

pub fn make_time(hour: u32, min: u32, sec: u32) -> MyTimeType {
    return NaiveTime::from_hms_opt(hour, min, sec).expect("from_hms_opt() failed");
}

pub fn tomorrow(date: &MyDateType) -> MyDateType {
    return date.succ_opt().expect("succ_opt() failed");
}

pub fn yesterday(date: &MyDateType) -> MyDateType {
    return date.pred_opt().expect("pred_opt() failed");
}

pub fn date_at_hms(date: &MyDateType, hour: u32, min: u32, sec: u32) -> MyDateTimeType {
    return date
        .and_hms_opt(hour, min, sec)
        .expect("and_hms_opt() failed");
}

pub fn get_now() -> MyDateTimeType {
    use chrono::Local;
    Local::now().naive_local()
}

/// 从1970-01-01开始的天数构造日期
pub fn date_from_days_since_epoch(days_since_epoch: i32) -> MyDateType {
    // 1970年1月1日是公元1年之后的第719,163天。
    let days_from_ce = days_since_epoch + 719163;
    // 从天数创建日期。
    NaiveDate::from_num_days_from_ce_opt(days_from_ce).expect("from_num_days_from_ce_opt() failed")
}

/// 日期转为从1970-01-01以来的天数
pub fn date_to_days_since_epoch(date: &MyDateType) -> i32 {
    // 计算自公元1年1月1日以来的天数
    let days_since_ce = date.num_days_from_ce() as i32;
    // 计算从Unix Epoch到公元1年1月1日的天数
    let days_from_epoch_to_ce = 719163;
    // 计算从Unix Epoch到指定日期的天数
    days_since_ce - days_from_epoch_to_ce
}

/// 从1970-01-01 00:00:00以来的纳秒总数构造日期时间
pub fn datetime_from_timestamp_nanos(nanos: i64) -> MyDateTimeType {
    DateTime::from_timestamp_nanos(nanos).naive_utc()
}

/// 从1970-01-01 00:00:00以来的纳秒总数
pub fn datetime_to_timestamp_nanos(datetime: &MyDateTimeType) -> i64 {
    datetime
        .and_utc()
        .timestamp_nanos_opt()
        .expect("timestamp_nanos_opt() failed")
}

/// 从00:00:00开始的纳秒数创建时间
pub fn time_from_midnight_nanos(time_nanos: i64) -> MyTimeType {
    let secs = (time_nanos / 1_000_000_000) as u32;
    let nano = (time_nanos % 1_000_000_000) as u32;
    NaiveTime::from_num_seconds_from_midnight_opt(secs, nano)
        .expect("from_num_seconds_from_midnight_opt() failed")
}

/// 从00:00:00开始的纳秒数
pub fn time_to_midnight_nanos(time: &NaiveTime) -> i64 {
    let secs = time.num_seconds_from_midnight() as i64;
    secs * 1_000_000_000 + time.nanosecond() as i64
}
