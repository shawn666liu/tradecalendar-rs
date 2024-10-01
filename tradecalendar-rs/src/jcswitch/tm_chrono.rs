// use anyhow::{anyhow, Result};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

pub type MyDateType = NaiveDate;
pub type MyDateTimeType = NaiveDateTime;
pub type MyTimeType = NaiveTime;

pub fn make_date(year: i32, month: u32, day: u32) -> MyDateType {
    return NaiveDate::from_ymd_opt(year, month, day).unwrap();
}

pub fn make_time(hour: u32, min: u32, sec: u32) -> MyTimeType {
    return NaiveTime::from_hms_opt(hour, min, sec).unwrap();
}

pub fn tomorrow(date: &MyDateType) -> MyDateType {
    return date.succ_opt().unwrap();
}

pub fn yesterday(date: &MyDateType) -> MyDateType {
    return date.pred_opt().unwrap();
}

pub fn date_at_hms(date: &MyDateType, hour: u32, min: u32, sec: u32) -> MyDateTimeType {
    return date.and_hms_opt(hour, min, sec).unwrap();
}

pub fn get_now() -> MyDateTimeType {
    use chrono::Local;
    Local::now().naive_local()
}

pub fn date_from_i32(days_since_epoch: i32) -> MyDateType {
    // 1970年1月1日是公元1年之后的第719,163天。
    let days_from_ce = days_since_epoch + 719163;
    // 从天数创建日期。
    NaiveDate::from_num_days_from_ce_opt(days_from_ce).unwrap()
    // .ok_or_else(|| {
    //     anyhow!(
    //         "days_since_epoch({}), convert to date failed",
    //         days_since_epoch
    //     )
    // })?
}
