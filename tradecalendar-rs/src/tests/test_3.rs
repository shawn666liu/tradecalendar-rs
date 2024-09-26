#[cfg(test)]
#[allow(unused_variables)]
mod tests {
    use anyhow::Result;
    use std::str::FromStr;

    use crate::get_buildin_calendar;
    use crate::time_helper::*;
    use crate::tradecalendar::*;

    #[test]
    fn buildin() -> Result<()> {
        let start = make_date(2024, 1, 2);
        let mut mgr = get_buildin_calendar(Some(start))?;
        let trade_day = make_date(2024, 9, 28);
        let trading = mgr.is_trading_day(&trade_day)?;
        println!("{} is tradeday? {}", trade_day, trading);
        let now = get_now();
        let change = mgr.time_changed(&now, false)?;
        println!("change info: {:#?}", change);
        Ok(())
    }

    #[test]
    fn test_calendar() -> Result<()> {
        let buf = "date,morning,trading,night,next
2021-01-01,false,false,false,2021-01-04
2021-01-02,false,false,false,2021-01-04
2021-01-03,false,false,false,2021-01-04
2021-01-04,false,true,true,2021-01-05
2021-01-05,true,true,true,2021-01-06
2021-01-06,true,true,true,2021-01-07
2021-01-07,true,true,true,2021-01-08
2021-01-08,true,true,true,2021-01-11
2021-01-09,true,false,false,2021-01-11
2021-01-10,false,false,false,2021-01-11
2021-01-11,false,true,true,2021-01-12
2021-01-12,true,true,true,2021-01-13
2021-01-13,true,true,true,2021-01-14
2021-01-14,true,true,true,2021-01-15
2021-01-15,true,true,true,2021-01-18
2021-01-16,true,false,false,2021-01-18
2021-01-17,false,false,false,2021-01-18
2021-01-18,false,true,true,2021-01-19
2021-01-19,true,true,true,2021-01-20
2021-01-20,true,true,true,2021-01-21
2021-01-21,true,true,true,2021-01-22
2021-01-22,true,true,true,2021-01-25
2021-01-23,true,false,false,2021-01-25
2021-01-24,false,false,false,2021-01-25
2021-01-25,false,true,true,2021-01-26
2021-01-26,true,true,true,2021-01-27
2021-01-27,true,true,true,2021-01-28
2021-01-28,true,true,true,2021-01-29
2021-01-29,true,true,true,2021-02-01
2021-01-30,true,false,false,2021-02-01
2021-01-31,false,false,false,2021-02-01
2021-02-01,false,true,true,2021-02-02
2021-12-29,true,true,true,2021-12-30
2021-12-30,true,true,true,2021-12-31
2021-12-31,true,true,false,2022-01-03";

        // 以上数据:
        // 1） 从 2021-02-01 后开始缺失，
        // 2） 由于 2021-12-31 这条数据实际是在2020年12月左右生成的, 其后一交易日当时只能推断出是2022-01-03,
        // 实际情况是2022年的公共假日在2021年12月左右公布，2022-01-03是节假日, 这个位置正确的值是2022-01-04
        // 不更新交易日历，仅通过fail_safe，是无法修正这个数据的

        let y20201230 = make_date(2020, 12, 30);
        let y20201231 = make_date(2020, 12, 31);
        let y20210101 = make_date(2021, 1, 1);
        let y20210102 = make_date(2021, 1, 2);
        let y20210104 = make_date(2021, 1, 4);
        let y20210105 = make_date(2021, 1, 5);
        let y20210108 = make_date(2021, 1, 8);
        let y20210111 = make_date(2021, 1, 11);
        let y20210120 = make_date(2021, 1, 20);
        let y20210121 = make_date(2021, 1, 21);
        let y20210202 = make_date(2021, 2, 2);
        let y20211231 = make_date(2021, 12, 31);
        let y20220101 = make_date(2022, 1, 1);
        let y20220102 = make_date(2022, 1, 2);
        let y20220103 = make_date(2022, 1, 3);
        let y20220104 = make_date(2022, 1, 4);
        let y20240101 = make_date(2024, 1, 1);

        let list = Tradingday::load_csv_read(buf.as_bytes())?;
        // println!("{:?}", list);

        // let wtr = CSV::save_csv_write(Vec::new(), list)?;
        // let data = String::from_utf8(wtr.into_inner()?)?;
        // println!("\ncsv result is\n{}", data);

        let mut mgr = TradeCalendar::new();
        mgr.set_config(
            &make_time(19, 30, 0),
            &make_time(20, 30, 0),
            &make_time(2, 31, 0),
            &make_time(8, 30, 0),
            &make_time(15, 30, 0),
        )?;

        mgr.reload(list)?;
        let td = mgr.get_next_trading_day(&y20210101, 1)?;
        assert_eq!(td.date, y20210104,);
        let td = mgr.get_next_trading_day(&y20210102, 1)?;
        assert_eq!(td.date, y20210104,);
        let td = mgr.get_next_trading_day(&y20210104, 1)?;
        assert_eq!(td.date, y20210105);
        let td = mgr.get_next_trading_day(&y20210104, 4)?;
        assert_eq!(td.date, y20210108);
        let td = mgr.get_next_trading_day(&y20210104, 5)?;
        assert_eq!(td.date, y20210111);
        let td = mgr.get_next_trading_day(&y20210108, 1)?;
        assert_eq!(td.date, y20210111);
        let td = mgr.get_prev_trading_day(&y20210108, 3)?;
        assert_eq!(td.date, y20210105);

        let datetime = date_at_hms(&y20210105, 9, 10, 5);
        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, false)?;
        assert_ne!(old_date, curr_date);
        assert_ne!(old_tday, curr_tday);
        assert!(opt_err.is_none());
        assert_eq!(mgr.is_trading(), true);

        let datetime = date_at_hms(&y20210108, 19, 28, 30);
        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, false)?;
        assert_ne!(old_tday, curr_tday);
        assert_eq!(mgr.current_tday(), &y20210108);
        assert_eq!(mgr.is_trading(), false);

        let datetime = date_at_hms(&y20210108, 19, 29, 30);
        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, false)?;
        assert_eq!(old_date, curr_date);
        assert_eq!(old_tday, curr_tday);
        assert_eq!(mgr.current_tday(), &y20210108);
        assert_eq!(mgr.is_trading(), false);

        let datetime = date_at_hms(&y20210108, 19, 30, 0);
        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, false)?;
        assert_ne!(old_tday, curr_tday);
        assert_eq!(mgr.current_tday(), &y20210111);
        assert_eq!(mgr.is_trading(), false);

        let datetime = date_at_hms(&y20210108, 20, 30, 0);
        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, false)?;
        assert_eq!(old_tday, curr_tday);
        assert_eq!(mgr.current_tday(), &y20210111);
        assert_eq!(mgr.is_trading(), true);

        // 中间数据缺失
        let datetime = date_at_hms(&y20210202, 0, 0, 0);
        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, true)?;
        assert!(opt_err.is_some());
        // println!("{}", opt_err.unwrap());
        assert_eq!(mgr.current_tday(), &y20210202);

        // fail_safe

        // missing before, 这种情况在实盘一般是不会出现的，因为日期总是向后推进，不会向前
        let datetime = date_at_hms(&y20201230, 20, 30, 0);
        let res = mgr.time_changed(&datetime, false);
        assert!(res.is_err());

        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, true)?;
        assert_ne!(old_tday, curr_tday);
        assert!(opt_err.is_some());
        assert_eq!(mgr.current_tday(), &y20201231);
        assert_eq!(mgr.is_trading(), true);

        // missing after, 实盘可能遭遇这种情况,
        let datetime = date_at_hms(&y20211231, 10, 30, 0);
        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, true)?;
        // println!(
        //     "({}) => {}, {}, {}, {}",
        //     datetime, old_tday, curr_tday, old_date, curr_date
        // );

        // 由于12月31日后面是元旦，是假期， 所以12月31日没有夜盘，交易日是不会切换的
        let datetime = date_at_hms(&y20211231, 21, 30, 0);
        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, true)?;
        // println!(
        //     "({}) => {}, {}, {}, {}",
        //     datetime, old_tday, curr_tday, old_date, curr_date
        // );
        assert!(opt_err.is_none());
        assert_eq!(mgr.current_tday(), &y20211231);
        assert_eq!(mgr.is_trading(), false);

        // let mut datetime = date_at_hms(&y20240101,0, 30, 0);
        // for idx in 1..=50 {
        //     datetime += chrono::Duration::hours(3);
        //     let (old_tday, curr_tday, old_date, curr_date, opt_err) =
        //         mgr.time_changed(&datetime, true)?;
        //     println!(
        //         "{:>2}: ({} {:>}) => {}, {}, {}, {}, trading? {}",
        //         idx,
        //         datetime,
        //         datetime.date().weekday(),
        //         old_tday,
        //         curr_tday,
        //         old_date,
        //         curr_date,
        //         mgr.is_trading()
        //     );
        // }

        // 仅能判断出2024-01-02没有凌晨盘，但无法确定2024-01-02是否节假日
        #[cfg(feature = "with-chrono")]
        let tday = mgr.fail_safe_tradingday(&(y20240101 + chrono::Duration::days(1)));
        #[cfg(feature = "with-jiff")]
        let tday =
            mgr.fail_safe_tradingday(&(y20240101 + std::time::Duration::from_secs(3600 * 24)));
        assert!(!tday.morning);

        let start = date_at_hms(&y20210120, 18, 22, 0);
        mgr.reset(Some(&start))?;
        assert_eq!(mgr.current_tday(), &y20210120);
        let start = MyDateTimeType::from_str("2021-01-20T20:22:00").expect("chrono");
        mgr.reset(Some(&start))?;
        assert_eq!(mgr.current_tday(), &y20210121);
        Ok(())
    }
}
