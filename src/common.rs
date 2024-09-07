#[cfg(feature = "with-chrono")]
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Weekday};

#[cfg(feature = "with-jiff")]
use {
    jiff::civil::{self, Date, DateTime, Time, Weekday},
    jiff::{ToSpan, Unit},
};

#[cfg(feature = "with-chrono")]
pub type MyDateType = NaiveDate;

#[cfg(feature = "with-jiff")]
pub type MyDateType = Date;

#[cfg(feature = "with-chrono")]
pub type MyDateTimeType = NaiveDateTime;

#[cfg(feature = "with-jiff")]
pub type MyDateTimeType = DateTime;

#[cfg(feature = "with-chrono")]
pub type MyTimeType = NaiveTime;

#[cfg(feature = "with-jiff")]
pub type MyTimeType = Time;

//////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "with-chrono")]
pub(crate) fn make_date(year: i32, month: u32, day: u32) -> MyDateType {
    return NaiveDate::from_ymd_opt(year, month, day).unwrap();
}

#[cfg(feature = "with-jiff")]
pub(crate) fn make_date(year: i16, month: i8, day: i8) -> MyDateType {
    return Date::constant(year, month, day);
}

#[cfg(feature = "with-chrono")]
pub(crate) fn make_time(hour: u32, min: u32, sec: u32) -> MyTimeType {
    return NaiveTime::from_hms_opt(hour, min, sec).unwrap();
}

#[cfg(feature = "with-jiff")]
pub(crate) fn make_time(hour: i8, minute: i8, second: i8) -> MyTimeType {
    return Time::constant(hour, minute, second, 0);
}

#[cfg(feature = "with-chrono")]
pub(crate) fn tomorrow(date: &MyDateType) -> MyDateType {
    return date.succ_opt().unwrap();
}

#[cfg(feature = "with-jiff")]
pub(crate) fn tomorrow(date: &MyDateType) -> MyDateType {
    return date.tomorrow().unwrap();
}

#[cfg(feature = "with-chrono")]
pub(crate) fn yesterday(date: &MyDateType) -> MyDateType {
    return date.pred_opt().unwrap();
}

#[cfg(feature = "with-jiff")]
pub(crate) fn yesterday(date: &MyDateType) -> MyDateType {
    return date.yesterday().unwrap();
}
