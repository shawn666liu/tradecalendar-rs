// #![feature(try_blocks)]

use anyhow::{anyhow, Result};
use clap::{arg, value_parser, Command};
use encoding_rs_io::DecodeReaderBytes;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[cfg(feature = "with-chrono")]
use chrono::NaiveDate;
#[cfg(feature = "with-jiff")]
use jiff::civil::Date;

use tradecalendar::calendar_helper::*;
use tradecalendar::common::*;

// input 节假日文件格式如下
/*
date,name
2023-01-02,元旦
2023-01-23,春节
2023-01-24,春节
2023-01-25,春节
2023-01-26,春节
2023-01-27,春节
2023-04-05,清明节
2023-05-01,劳动节
2023-05-02,劳动节
2023-05-03,劳动节
2023-06-22,端午节
2023-06-23,端午节
2023-09-29,中秋
2023-10-02,国庆
2023-10-03,国庆
2023-10-04,国庆
2023-10-05,国庆
2023-10-06,国庆
*/

/// 输入节假日文件(不含周六周日)，生成交易日文件和calendar文件
fn main() -> Result<()> {
    let matches = Command::new("交易日更新程序")
        .version("0.1.0")
        .author("Shawn Liu <shawn666.liu@hotmail.com>")
        .about("Convert Holidays to Tradingdays, generate postgres/clickhouse sql files")
        .arg(
            arg!(-i --input <FILE> "holidays输入文件完整路径,格式为每行一条节假日%Y-%m-%d,或者逗号分隔取第一条,不含周六周日,没有csv header行")
            .value_parser(value_parser!(PathBuf)))
        .arg(
            arg!(-o --outdir <DIR> "sql文件输出的目录(可选), 将生成pg_trade_day.sql和ch_calendar.sql")
            .required(false))
        .get_matches();

    let holidays_file = matches
        .get_one::<PathBuf>("input")
        .ok_or_else(|| anyhow!("请使用-i指定输入的节假日文件"))?;

    // 如果没有提供输出目录，则在当前目录下的output目录
    let out_dir = match matches.get_one::<String>("outdir") {
        Some(output) => output,
        _ => "./output",
    };

    let file = File::open(holidays_file)?;
    let lines = BufReader::new(DecodeReaderBytes::new(file)).lines();

    // 使用循环,
    let mut holidays = Vec::new();
    let mut holiday_names: Vec<String> = Vec::new();
    for line_res in lines {
        let line = line_res?;
        println!("{}", line);
        let line = line.trim();
        let mut splt = line.split(',');
        match splt.next() {
            Some(item) => {
                let item = item.trim();
                // 空行,非数字开头的都忽略掉
                if !item.is_empty() && item.chars().next().unwrap().is_ascii_digit() {
                    #[cfg(feature = "with-chrono")]
                    let date = NaiveDate::parse_from_str(item, "%Y-%m-%d")?;
                    #[cfg(feature = "with-jiff")]
                    let date = Date::strptime("%Y-%m-%d", item)?;
                    holidays.push(date);
                    match splt.next() {
                        Some(name) => holiday_names.push(name.into()),
                        None => holiday_names.push("".to_owned()),
                    }
                }
            }
            None => {}
        }
    }

    // 使用map, 演示iter.map()里面用?返回错误值
    // let holidays = lines
    //     .into_iter()
    //     .map(|x| {
    //         Ok(NaiveDate::parse_from_str(
    //             &(x?).split(',').next().unwrap(),
    //             "%Y-%m-%d",
    //         )?)
    //     })
    //     .collect::<Result<Vec<_>, anyhow::Error>>()?;

    // 演示使用map和try，需要 #![feature(try_blocks)]
    // let holidays = lines
    //     .into_iter()
    //     .map(|x| try { NaiveDate::parse_from_str(&(x?).split(',').next().unwrap(), "%Y-%m-%d")? })
    //     .collect::<Result<Vec<_>, anyhow::Error>>()?;

    gen_holiday_sql(out_dir, &holidays, &holiday_names)?;
    let td_list = holidays_to_tradingdays(&holidays);
    gen_trade_day_csv(&td_list, out_dir)?;
    let all_days = tradingdays_to_calendar(&td_list);
    gen_calendar_csv(&all_days, out_dir)?;
    println!("Finished.");

    Ok(())
}
