use jiff::civil::{Date, DateTime, Time};

pub type MyDateType = Date;

pub type MyDateTimeType = DateTime;

pub type MyTimeType = Time;

//////////////////////////////////////////////////////////////////////////////////////////////////

pub fn make_date(year: i16, month: i8, day: i8) -> MyDateType {
    return Date::constant(year, month, day);
}

pub fn make_time(hour: i8, minute: i8, second: i8) -> MyTimeType {
    return Time::constant(hour, minute, second, 0);
}

pub fn tomorrow(date: &MyDateType) -> MyDateType {
    return date.tomorrow().unwrap();
}

pub fn yesterday(date: &MyDateType) -> MyDateType {
    return date.yesterday().unwrap();
}

pub fn date_at_hms(date: &MyDateType, hour: i8, minute: i8, second: i8) -> MyDateTimeType {
    return date.at(hour, minute, second, 0);
}

pub fn get_now() -> MyDateTimeType {
    use jiff::Zoned;
    Zoned::now().datetime()
}

//////////////////////////////////////////////////////////////////////////////////////////////

pub fn date_from_i32(days: i32) -> MyDateType {
    // Date::from_unix_epoch_days(days)

    let DAYS_FROM_0000_01_01_TO_1970_01_01: i64 = 719_468;
    let DAYS_IN_ERA: i64 = 146_097;
    let days: i64 = days.into();

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
