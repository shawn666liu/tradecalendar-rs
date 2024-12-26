use jiff::civil::{Date, DateTime, Time};

pub type MyDateType = Date;

pub type MyDateTimeType = DateTime;

pub type MyTimeType = Time;

//////////////////////////////////////////////////////////////////////////////////////////////////

pub fn make_date(year: i32, month: u32, day: u32) -> MyDateType {
    return Date::constant(year as i16, month as i8, day as i8);
}

pub fn make_time(hour: u32, minute: u32, second: u32) -> MyTimeType {
    return Time::constant(hour as i8, minute as i8, second as i8, 0);
}

pub fn tomorrow(date: &MyDateType) -> MyDateType {
    return date.tomorrow().expect("tomorrow() failed");
}

pub fn yesterday(date: &MyDateType) -> MyDateType {
    return date.yesterday().expect("yesterday() failed");
}

pub fn date_at_hms(date: &MyDateType, hour: u32, minute: u32, second: u32) -> MyDateTimeType {
    return date.at(hour as i8, minute as i8, second as i8, 0);
}

pub fn get_now() -> MyDateTimeType {
    use jiff::Zoned;
    Zoned::now().datetime()
}

//////////////////////////////////////////////////////////////////////////////////////////////

/// 从1970-01-01开始的天数构造日期
#[allow(non_snake_case)]
pub fn date_from_days_since_epoch(days_since_epoch: i32) -> MyDateType {
    // function is pub(crate)
    // Date::from_unix_epoch_days(days_since_epoch)

    let DAYS_FROM_0000_01_01_TO_1970_01_01: i64 = 719_468;
    let DAYS_IN_ERA: i64 = 146_097;
    let days: i64 = days_since_epoch.into();

    let days = days + DAYS_FROM_0000_01_01_TO_1970_01_01;
    let era = days / DAYS_IN_ERA;
    let day_of_era = days % DAYS_IN_ERA;
    let year_of_era = (day_of_era - day_of_era / (1_460) + day_of_era / (36_524)
        - day_of_era / (DAYS_IN_ERA - (1)))
        / (365);
    let year = year_of_era + era * (400);
    let day_of_year = day_of_era - ((365) * year_of_era + year_of_era / (4) - year_of_era / (100));
    let month = (day_of_year * (5) + (2)) / (153);
    let day = day_of_year - ((153) * month + (2)) / (5) + (1);

    let month = if month < 10 { month + (3) } else { month - (9) };
    let year = if month <= 2 { year + (1) } else { year };

    Date::constant(year as i16, month as i8, day as i8)
}

/// 日期转为从1970-01-01以来的天数
#[allow(non_snake_case)]
pub fn date_to_days_since_epoch(date: &MyDateType) -> i32 {
    // method is pub(crate)
    // Date::to_unix_epoch_days(date)

    let DAYS_FROM_0000_01_01_TO_1970_01_01: i32 = 719_468;
    let DAYS_IN_ERA: i32 = 146_097;

    let year: i32 = date.year() as i32;
    let month: i32 = date.month() as i32;
    let day: i32 = date.day() as i32;

    let year = if month <= 2 { year - 1 } else { year };
    let month = if month > 2 { month - 3 } else { month + 9 };
    let era = year / (400);
    let year_of_era = year % (400);
    let day_of_year = (153 * month + 2) / (5) + day - (1);
    let day_of_era = year_of_era * (365) + year_of_era / (4) - year_of_era / (100) + day_of_year;
    let epoch_days = era * DAYS_IN_ERA + day_of_era - DAYS_FROM_0000_01_01_TO_1970_01_01;
    epoch_days
}

/// 从1970-01-01 00:00:00以来的纳秒总数构造日期时间
#[allow(non_snake_case)]
pub fn datetime_from_timestamp_nanos(nanos: i64) -> MyDateTimeType {
    let NANOS_PER_SECOND = 1_000_000_000;

    let secs = nanos.div_euclid(1_000_000_000);
    let nsecs = nanos.rem_euclid(1_000_000_000);
    let days = secs.div_euclid(86_400) as i32;
    let secs = secs.rem_euclid(86_400);

    let date = date_from_days_since_epoch(days);
    let time = time_from_midnight_nanos(secs * NANOS_PER_SECOND + nsecs);

    DateTime::from_parts(date, time)
}

/// 从1970-01-01 00:00:00以来的纳秒总数
#[allow(non_snake_case)]
pub fn datetime_to_timestamp_nanos(datetime: &MyDateTimeType) -> i64 {
    // method is private
    // datetime.to_nanosecond()

    let NANOS_PER_SECOND = 1_000_000_000;
    let NANOS_PER_CIVIL_DAY = 60 * 60 * 24 * NANOS_PER_SECOND;
    let date = datetime.date();
    let time = datetime.time();
    let days = date_to_days_since_epoch(&date);
    let mut nanos = time_to_midnight_nanos(&time);
    nanos += days as i64 * NANOS_PER_CIVIL_DAY;
    nanos
}

/// 从00:00:00开始的纳秒数创建时间
#[allow(non_snake_case)]
pub fn time_from_midnight_nanos(time_nanos: i64) -> MyTimeType {
    // function is pub(crate)
    // Time::from_nanosecond(time_nanos)

    let NANOS_PER_SECOND = 1_000_000_000;
    let NANOS_PER_MINUTE = 60 * 1_000_000_000;
    let NANOS_PER_HOUR = 60 * 60 * 1_000_000_000;

    let nanosecond = time_nanos;
    let hour = nanosecond / NANOS_PER_HOUR;
    let minute = (nanosecond % NANOS_PER_HOUR) / NANOS_PER_MINUTE;
    let second = (nanosecond % NANOS_PER_MINUTE) / NANOS_PER_SECOND;
    let subsec_nanosecond = nanosecond % NANOS_PER_SECOND;
    Time::constant(
        hour as i8,
        minute as i8,
        second as i8,
        subsec_nanosecond as i32,
    )
}

/// 从00:00:00开始的纳秒数,
/// max time: `23:59:59.999999999`
#[allow(non_snake_case)]
pub fn time_to_midnight_nanos(time: &MyTimeType) -> i64 {
    // method is pub(crate)
    // time.to_nanosecond()

    let NANOS_PER_SECOND = 1_000_000_000;
    let NANOS_PER_MINUTE = 60 * 1_000_000_000;
    let NANOS_PER_HOUR = 60 * 60 * 1_000_000_000;

    let mut nanos = time.hour() as i64 * NANOS_PER_HOUR;
    nanos += time.minute() as i64 * NANOS_PER_MINUTE;
    nanos += time.second() as i64 * NANOS_PER_SECOND;
    nanos += time.subsec_nanosecond() as i64;
    nanos
}
