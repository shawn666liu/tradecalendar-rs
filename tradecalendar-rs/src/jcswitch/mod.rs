// jiff/chrono switch
#[allow(dead_code)]
#[cfg(feature = "with-chrono")]
mod tm_chrono;
#[cfg(feature = "with-chrono")]
pub use tm_chrono::*;

#[cfg(feature = "with-jiff")]
mod tm_jiff;
#[cfg(feature = "with-jiff")]
pub use tm_jiff::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "with-jiff")]
    #[test]
    fn get_date_jiff() {
        let date = date_from_days_since_epoch(19645);
        let days = date_to_days_since_epoch(&date);
        println!("by jiff: {}, days {}", date, days);
        assert_eq!(days, 19645);

        let now = get_now();
        let date = now.date();
        let time = now.time();
        let date_days = date_to_days_since_epoch(&date);
        let datetime_nanos = datetime_to_timestamp_nanos(&now);
        let time_nanos = time_to_midnight_nanos(&time);

        let date1 = date_from_days_since_epoch(date_days);
        let datetime1 = datetime_from_timestamp_nanos(datetime_nanos);
        let time1 = time_from_midnight_nanos(time_nanos);

        println!("datetime_nanos {datetime_nanos}, datetime {now} vs {datetime1}");
        println!("time_nanos {time_nanos}, time {time} vs {time1}");
        println!("days_since_epoch {date_days}, date {date} vs {date1}\n");

        assert_eq!(now, datetime1);
        assert_eq!(time, time1);
        assert_eq!(date, date1);
    }
    #[cfg(feature = "with-chrono")]
    #[test]
    fn get_date_chrono() {
        let date = date_from_days_since_epoch(19645);
        let days = date_to_days_since_epoch(&date);
        println!("by chrono: {}, days {}", date, days);
        assert_eq!(days, 19645);

        let now = get_now();
        let date = now.date();
        let time = now.time();
        let date_days = date_to_days_since_epoch(&date);
        let datetime_nanos = datetime_to_timestamp_nanos(&now);
        let time_nanos = time_to_midnight_nanos(&time);

        let date1 = date_from_days_since_epoch(date_days);
        let datetime1 = datetime_from_timestamp_nanos(datetime_nanos);
        let time1 = time_from_midnight_nanos(time_nanos);

        println!("datetime_nanos {datetime_nanos}, datetime {now} vs {datetime1}");
        println!("time_nanos {time_nanos}, time {time} vs {time1}");
        println!("days_since_epoch {date_days}, date {date} vs {date1}\n");

        assert_eq!(now, datetime1);
        assert_eq!(time, time1);
        assert_eq!(date, date1);
    }
}
