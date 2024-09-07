use anyhow::{Context, Result};
use std::{fs::File, io::Write, path::Path};

#[cfg(feature = "with-chrono")]
use chrono::{Datelike, Duration, NaiveDate, Weekday};

#[cfg(feature = "with-jiff")]
use {
    jiff::civil::{Date, Weekday},
    jiff::ToSpan,
    std::ops::AddAssign,
};

use super::common::*;
use super::tradecalendar::*;

/// 将当年假期列表，转换为交易日列表(排除了周末及这些假期, 仅交易日)
pub fn holidays_to_tradingdays(holiday_list: &Vec<MyDateType>) -> Vec<MyDateType> {
    let mut the_day = make_date(holiday_list[0].year(), 1, 1);
    let next_year_first = make_date(holiday_list[0].year() + 1, 1, 1);
    let mut result: Vec<MyDateType> = Vec::with_capacity(260);
    while the_day < next_year_first {
        let dw = the_day.weekday();
        #[cfg(feature = "with-chrono")]
        {
            if dw != Weekday::Sat && dw != Weekday::Sun && !holiday_list.contains(&the_day) {
                result.push(the_day);
                // println!("{}", the_day);
            }
            the_day = tomorrow(&the_day);
        }
        #[cfg(feature = "with-jiff")]
        {
            if dw != Weekday::Saturday && dw != Weekday::Sunday && !holiday_list.contains(&the_day)
            {
                result.push(the_day);
                // println!("{}", the_day);
            }
            the_day = tomorrow(&the_day);
        }
    }
    println!("本年度交易日数量({})", result.len());
    return result;
}

/// 将交易日列表转换为Tradingday列表，交易日列表可以来自holidays_to_tradingdays()函数转换，也可以来自从其他平台的查询   
/// 更新: 现在非交易日也写入数据库，Tradingday的trading项为false，周六非交易日，但周五夜盘会持续到凌晨，所以其morning项可以为true   
/// 返回值: 上年度最后一个交易日及本年度所有日期构成的Tradingday列表
pub fn tradingdays_to_calendar(trading_days: &Vec<MyDateType>) -> Vec<Tradingday> {
    assert!(
        trading_days.len() > 2,
        "tradingdays_to_calendar() 输入日期数据太短"
    );

    // 去年计算上一年度的最后一个交易日的next时，是估算的，并不准确，因为那时国家尚未公布本新一年度的节假日，
    // 即我们并不知道后一年元旦放假的具体情况
    // 此时假期安排已经公布，需要进行修正
    // pre_day是上一年度的最后一个交易日，很可能是12月31日，如果不是的话，则倒退寻找

    let mut pre_day = yesterday(&make_date(trading_days[0].year(), 1, 1));

    let mut dw = pre_day.weekday();
    #[cfg(feature = "with-chrono")]
    while dw == Weekday::Sat || dw == Weekday::Sun {
        pre_day = yesterday(&pre_day);
        dw = pre_day.weekday();
    }
    #[cfg(feature = "with-jiff")]
    while dw == Weekday::Saturday || dw == Weekday::Sunday {
        pre_day = yesterday(&pre_day);
        dw = pre_day.weekday();
    }

    // pre_day_night: 表示pre_day是否有夜盘交易；后面会放元旦假，此时一定没有夜盘
    let mut pre_day_night = false;
    let first_tday = trading_days[0];
    let length = trading_days.len();
    let mut result: Vec<Tradingday> = Vec::with_capacity(length + 130);

    // 补充上年最后一个交易日的更新数据，
    // 注意: 如果要写数据库的话，这条记录必须是更新，而不是插入
    // 由于中国在十一月十二月没有额外的公共假期，所以很容易判断pre_day的morning，如果是周一则没有，其他日期则有
    let pre_year_last_tday = Tradingday {
        date: pre_day,
        #[cfg(feature = "with-chrono")]
        morning: dw != Weekday::Mon,
        #[cfg(feature = "with-jiff")]
        morning: dw != Weekday::Monday,
        trading: true,
        night: pre_day_night,
        next: first_tday,
    };
    result.push(pre_year_last_tday);

    let next_yuandan = make_date(trading_days[trading_days.len() - 1].year() + 1, 1, 1);

    println!(
        "正在准备数据, [{}, {}),请稍候... ",
        first_tday, next_yuandan
    );

    for idx in 0..length {
        let mut the_day = trading_days[idx];

        // 中间可能有非交易日
        while pre_day < yesterday(&the_day) {
            pre_day = tomorrow(&pre_day);
            let rec = Tradingday {
                date: pre_day,
                morning: pre_day_night,
                trading: false,
                night: false,
                next: the_day,
            };
            result.push(rec);
            pre_day_night = false
        }

        if idx == length - 1 {
            // 这是当年度的最后一个交易日天(不一定是12月31日，因为12月31日有可能是周末)，不能通过idx+1获取更后面一天next_tday,
            // 从the_day(交易)开始，到后一年元旦，中间都不交易，而且，元旦也不交易, the_day没有夜盘
            let mut next_year_first_trading_day = next_working_day(&the_day, 1);
            if next_year_first_trading_day.day() == 1 {
                // 如果元旦不是周末，向后再找
                next_year_first_trading_day = next_working_day(&next_year_first_trading_day, 1);
            }
            let rec = Tradingday {
                date: the_day,
                morning: pre_day_night,
                trading: true,
                night: false,
                next: next_year_first_trading_day,
            };
            result.push(rec);
            #[cfg(feature = "with-chrono")]
            {
                the_day += Duration::days(1);
            }

            #[cfg(feature = "with-jiff")]
            the_day.add_assign(1.days());

            while the_day < next_yuandan {
                let rec = Tradingday {
                    date: the_day,
                    morning: false,
                    trading: false,
                    night: false,
                    next: next_year_first_trading_day,
                };
                result.push(rec);
                the_day = tomorrow(&the_day);
            }
        } else {
            let next_trading_day = trading_days[idx + 1];
            // 判断当天凌晨有交易：昨天夜里有交易的话，则当天凌晨有交易，反之亦然，充要条件
            // 判断当天是否有夜盘：第二天是交易日，或者3天后是交易日且为星期一
            // 这个判断可靠吗？ 有没有可能，仅放假周六周日，但周五晚上没有夜盘的情况？
            #[cfg(feature = "with-chrono")]
            let has_night_mkt = next_trading_day == tomorrow(&the_day)
                || (next_trading_day == the_day + Duration::days(3)
                    && next_trading_day.weekday() == Weekday::Mon);

            #[cfg(feature = "with-jiff")]
            let has_night_mkt = next_trading_day == tomorrow(&the_day)
                || (next_trading_day == the_day + 3.days()
                    && next_trading_day.weekday() == Weekday::Monday);

            let rec = Tradingday {
                date: the_day,
                morning: pre_day_night,
                trading: true,
                night: has_night_mkt,
                next: next_trading_day,
            };
            result.push(rec);
            pre_day = the_day;
            pre_day_night = has_night_mkt;
        }
    }
    println!("自然日数量({}),含上年最后交易日", result.len());
    return result;
}

/// 生成 holiday.sql用于postgres
pub fn gen_holiday_sql<P: AsRef<Path>>(
    out_dir: P,
    holidays: &[MyDateType],
    holiday_names: &[String],
) -> anyhow::Result<()> {
    let out_dir = out_dir.as_ref();
    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir)
            .expect(&format!("create out dir `{}` failed.", out_dir.display()));
    }
    let p1 = out_dir.join("pg_holiday.sql");
    let mut f1 = File::create(&p1).with_context(|| p1.display().to_string())?;
    write!(f1, "{}", "insert into holiday (_date,_name) values ")?;
    let last_idx = holidays.len() - 1;
    for (idx, tday) in holidays.iter().enumerate() {
        #[cfg(feature = "with-chrono")]
        write!(
            f1,
            "('{}','{}')",
            tday.format("%Y-%m-%d"),
            holiday_names[idx],
        )?;
        #[cfg(feature = "with-jiff")]
        write!(
            f1,
            "('{}','{}')",
            tday.strftime("%Y-%m-%d"),
            holiday_names[idx],
        )?;

        if idx < last_idx {
            f1.write(b",")?;
        }
    }
    f1.write(b";")?;
    println!("pg_holiday.sql: {}", std::fs::canonicalize(p1)?.display());

    Ok(())
}

/// 生成交易日csv文件
///
/// 生成postgresql的trade_day表，只有_date一个字段
pub fn gen_trade_day_csv<P: AsRef<Path>>(tdays: &Vec<MyDateType>, out_dir: P) -> Result<()> {
    let out_dir = out_dir.as_ref();
    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir)
            .expect(&format!("create out dir `{}` failed.", out_dir.display()));
    }
    let p1 = out_dir.join("pg_trade_day.sql");
    let p2 = out_dir.join("trade_day.csv");
    let mut f1 = File::create(&p1).with_context(|| p1.display().to_string())?;
    let mut f2 = File::create(&p2).with_context(|| p2.display().to_string())?;
    write!(f1, "{}", "insert into trade_day (_date) values ")?;
    writeln!(f2, "tradeday")?;
    let last_idx = tdays.len() - 1;
    for (idx, tday) in tdays.iter().enumerate() {
        #[cfg(feature = "with-chrono")]
        {
            write!(f1, "('{}')", tday.format("%Y-%m-%d"))?;
            if idx < last_idx {
                f1.write(b",")?;
            }
            writeln!(f2, "{}", tday.format("%Y-%m-%d"))?;
        }
        #[cfg(feature = "with-jiff")]
        {
            write!(f1, "('{}')", tday.strftime("%Y-%m-%d"))?;
            if idx < last_idx {
                f1.write(b",")?;
            }
            writeln!(f2, "{}", tday.strftime("%Y-%m-%d"))?;
        }
    }
    f1.write(b";")?;
    println!("pg_trade_day.sql: {}", std::fs::canonicalize(p1)?.display());
    println!("trade_day.csv: {}", std::fs::canonicalize(p2)?.display());

    Ok(())
}

/// 生成calendar.csv文件
///
/// 生成clickhouse所需的futuredb.calendar表的sql文件
pub fn gen_calendar_csv<P: AsRef<Path>>(
    calendar: &Vec<Tradingday>,
    out_dir: P,
) -> anyhow::Result<()> {
    let out_dir = out_dir.as_ref();
    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir)
            .expect(&format!("create out dir `{}` failed.", out_dir.display()));
    }
    let p1 = out_dir.join("calendar.csv");
    let mut f1 = File::create(&p1).with_context(|| p1.display().to_string())?;

    let p2 = out_dir.join("ch_calendar.sql");
    let mut f2 = File::create(&p2).with_context(|| p2.display().to_string())?;

    writeln!(f1, "date,morning,trading,night,next")?;

    // calendar表的insert是覆盖式的，不用检测冲突，bool变量必须是short型
    f2.write(b"insert into futuredb.calendar (date,morning,trading,night,next) values ")?;

    let last_idx = calendar.len() - 1;
    for (idx, t) in calendar.iter().enumerate() {
        #[cfg(feature = "with-chrono")]
        {
            writeln!(
                f1,
                "{},{},{},{},{}",
                t.date.format("%Y-%m-%d"),
                t.morning,
                t.trading,
                t.night,
                t.next.format("%Y-%m-%d"),
            )?;
            write!(
                f2,
                "('{}',{},{},{},'{}')",
                t.date.format("%Y-%m-%d"),
                if t.morning { 1 } else { 0 },
                if t.trading { 1 } else { 0 },
                if t.night { 1 } else { 0 },
                t.next.format("%Y-%m-%d"),
            )?;
        }
        #[cfg(feature = "with-jiff")]
        {
            writeln!(
                f1,
                "{},{},{},{},{}",
                t.date.strftime("%Y-%m-%d"),
                t.morning,
                t.trading,
                t.night,
                t.next.strftime("%Y-%m-%d"),
            )?;
            write!(
                f2,
                "('{}',{},{},{},'{}')",
                t.date.strftime("%Y-%m-%d"),
                if t.morning { 1 } else { 0 },
                if t.trading { 1 } else { 0 },
                if t.night { 1 } else { 0 },
                t.next.strftime("%Y-%m-%d"),
            )?;
        }
        if idx < last_idx {
            f2.write(b",")?;
        }
    }
    writeln!(f2, ";")?;
    writeln!(f2, "")?;
    // 优化表，删除重复
    writeln!(f2, "optimize table futuredb.calendar final;")?;
    println!("calendar.csv: {}", std::fs::canonicalize(p1)?.display());
    println!("ch_calendar.sql: {}", std::fs::canonicalize(p2)?.display());

    return Ok(());
}

#[cfg(test)]
mod tests {
    // use serde::forward_to_deserialize_any;

    use super::*;
    // use std::env;

    #[test]
    fn t1() -> Result<()> {
        let holidays = vec![
            "2022-01-03",
            "2022-01-31",
            "2022-02-01",
            "2022-02-02",
            "2022-02-03",
            "2022-02-04",
            "2022-04-04",
            "2022-04-05",
            "2022-05-02",
            "2022-05-03",
            "2022-05-04",
            "2022-06-03",
            "2022-09-12",
            "2022-10-03",
            "2022-10-04",
            "2022-10-05",
            "2022-10-06",
            "2022-10-07",
        ];

        #[cfg(feature = "with-chrono")]
        let holidays: Vec<MyDateType> = holidays
            .iter()
            .map(|&x| {
                MyDateType::parse_from_str(x, "%Y-%m-%d")
                    .expect(&format!("parse holiday error:{}", x))
            })
            .collect();

        #[cfg(feature = "with-jiff")]
        let holidays: Vec<MyDateType> = holidays
            .iter()
            .map(|&x| {
                MyDateType::strptime("%Y-%m-%d", x).expect(&format!("parse holiday error:{}", x))
            })
            .collect();

        let out_dir = "./target/temp/sql";
        let td_lst = holidays_to_tradingdays(&holidays);
        gen_trade_day_csv(&td_lst, out_dir)?;
        let all_days = tradingdays_to_calendar(&td_lst);
        gen_calendar_csv(&all_days, out_dir)?;

        // for d in all_days.iter() {
        //     println!("{}", d);
        // }
        println!("Finished.");
        Ok(())
    }
}
