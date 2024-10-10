use anyhow::{anyhow, Result};

use tradecalendar::jcswitch::{
    date_from_days_since_epoch, date_to_days_since_epoch, datetime_from_timestamp_nanos, make_date,
    time_from_midnight_nanos, time_to_midnight_nanos,
};
use tradecalendar::{self, NotTradingSearchMethod, TradingdayCache};

pub struct TradeCalendarPP {
    entity: tradecalendar::TradeCalendar,
}

fn get_buildin_calendar(start_date: i32) -> Result<Box<TradeCalendarPP>> {
    let start_date = date_from_days_since_epoch(start_date);
    tradecalendar::get_buildin_calendar(Some(start_date))
        .and_then(|r| Ok(Box::new(TradeCalendarPP { entity: r })))
}

fn get_csv_calendar(csv_file: String, start_date: i32) -> Result<Box<TradeCalendarPP>> {
    let start_date = date_from_days_since_epoch(start_date);
    tradecalendar::get_csv_calendar(csv_file, Some(start_date))
        .and_then(|r| Ok(Box::new(TradeCalendarPP { entity: r })))
}

fn get_calendar(
    db_conn: String,
    query: String,
    proto: String,
    csv_file: String,
    start_date: i32,
) -> Result<Box<TradeCalendarPP>> {
    let start_date = date_from_days_since_epoch(start_date);
    let proto = if proto.is_empty() { None } else { Some(proto) };
    let csv_file = if csv_file.is_empty() {
        None
    } else {
        Some(csv_file)
    };
    tradecalendar::get_calendar(&db_conn, &query, proto, csv_file, Some(start_date))
        .and_then(|r| Ok(Box::new(TradeCalendarPP { entity: r })))
}

impl TradeCalendarPP {
    pub fn reload(
        &mut self,
        db_conn: String,
        query: String,
        proto: String,
        csv_file: String,
        start_date: i32,
    ) -> Result<()> {
        let proto = if proto.is_empty() { None } else { Some(proto) };
        let csv_file = if csv_file.is_empty() {
            None
        } else {
            Some(csv_file)
        };
        let start_date = date_from_days_since_epoch(start_date);
        tradecalendar::reload_calendar(
            &mut self.entity,
            &db_conn,
            &query,
            proto,
            csv_file,
            Some(start_date),
        )
    }

    //////////////////////////////////////////////////////////////////////////////////
    // 以下为无状态接口
    //////////////////////////////////////////////////////////////////////////////////

    pub fn is_trading_day(&self, days_since_epoch: i32) -> Result<bool> {
        let date = date_from_days_since_epoch(days_since_epoch);
        self.entity.is_trading_day(&date)
    }
    pub fn get_next_trading_day(&self, days_since_epoch: i32, num: usize) -> Result<i32> {
        let date = date_from_days_since_epoch(days_since_epoch);
        self.entity
            .get_next_trading_day(&date, num)
            .and_then(|t| Ok(date_to_days_since_epoch(&t.date)))
    }
    pub fn get_prev_trading_day(&self, days_since_epoch: i32, num: usize) -> Result<i32> {
        let date = date_from_days_since_epoch(days_since_epoch);
        self.entity
            .get_prev_trading_day(&date, num)
            .and_then(|t| Ok(date_to_days_since_epoch(&t.date)))
    }
    /// 计算从start_date(含)到end_date(含)之间交易日的个数, 超出范围的部分将被忽略
    pub fn get_trading_days_count(&self, start_dt: i32, end_dt: i32) -> usize {
        let start_dt = date_from_days_since_epoch(start_dt);
        let end_dt = date_from_days_since_epoch(end_dt);
        self.entity.get_trading_days_count(&start_dt, &end_dt)
    }

    /// start_date(含)到end_date(含)之间交易日
    pub fn get_trading_days_list(&self, start_dt: i32, end_dt: i32) -> Vec<i32> {
        let start_dt = date_from_days_since_epoch(start_dt);
        let end_dt = date_from_days_since_epoch(end_dt);
        let r = self.entity.get_trading_day_slice(&start_dt, &end_dt);
        r.iter()
            .map(|t| date_to_days_since_epoch(&t.date))
            .collect()
    }

    fn trading_day_from_datetime(
        &self,
        datetime_nanos: i64,
        for_next: bool,
        is_finance_item: bool,
    ) -> Result<i32> {
        let datetime = datetime_from_timestamp_nanos(datetime_nanos);
        let method = if for_next {
            NotTradingSearchMethod::Next
        } else {
            NotTradingSearchMethod::Prev
        };
        self.entity
            .trading_day_from_datetime(&datetime, method, is_finance_item)
            .and_then(|r| Ok(date_to_days_since_epoch(&r)))
    }

    fn max_date(&self) -> i32 {
        let val = self
            .entity
            .max_date()
            .and_then(|m| Some(*m))
            .unwrap_or_else(|| make_date(1970, 1, 1));
        date_to_days_since_epoch(&val)
    }

    fn min_date(&self) -> i32 {
        let val = self
            .entity
            .min_date()
            .and_then(|m| Some(*m))
            .unwrap_or_else(|| make_date(1970, 1, 1));
        date_to_days_since_epoch(&val)
    }

    //////////////////////////////////////////////////////////////////////////////////
    // 以下为有状态时的接口
    //////////////////////////////////////////////////////////////////////////////////

    pub fn reset(&mut self, start_time_nanos: i64) -> Result<()> {
        let start = if start_time_nanos == 0 {
            Some(datetime_from_timestamp_nanos(start_time_nanos))
        } else {
            None
        };
        self.entity.reset(start.as_ref())
    }

    pub fn is_trading(&self) -> bool {
        self.entity.is_trading()
    }

    /// 前一交易日
    pub fn prev_tday(&self) -> i32 {
        date_to_days_since_epoch(self.entity.prev_tday())
    }

    /// 获取当前交易日
    pub fn current_tday(&self) -> i32 {
        date_to_days_since_epoch(self.entity.current_tday())
    }

    /// 后一交易日
    pub fn next_tday(&self) -> i32 {
        date_to_days_since_epoch(self.entity.next_tday())
    }
}

#[cxx::bridge(namespace = "tradecalendarpp")]
mod ffi {
    struct TradingDayPP {
        date: i32,
        morning: bool,
        trading: bool,
        night: bool,
        next: i32,
    }

    struct TimeChangedResultPP {
        prev_tradeday: i32,
        curr_tradeday: i32,
        prev_date: i32,
        curr_date: i32,
        error_msg: String,
    }

    struct TradingCheckConfigPP {
        /// 夜盘属于下一个交易日，这个变量指示什么时间点进行切换，一般是夜里19:00~20点，缺省19:30
        pub tday_shift: i64,

        //-------------------- begin 以下几个字段用来判断接口是否应该处于连接状态
        /// 缺省夜里 20:30
        pub night_begin: i64,
        /// 缺省凌晨 2:31
        pub night_end: i64,
        /// 缺省早上 8:30
        pub day_begin: i64,
        /// 缺省下午 15:30
        pub day_end: i64,
        //-------------------- end
    }

    extern "Rust" {
        type TradeCalendarPP;
        fn get_buildin_calendar(start_date: i32) -> Result<Box<TradeCalendarPP>>;
        fn get_csv_calendar(csv_file: String, start_date: i32) -> Result<Box<TradeCalendarPP>>;
        fn get_calendar(
            db_conn: String,
            query: String,
            proto: String,
            csv_file: String,
            start_date: i32,
        ) -> Result<Box<TradeCalendarPP>>;

        //////////////////////////////////////////////////////////////////////////////////

        fn reload(
            self: &mut TradeCalendarPP,
            db_conn: String,
            query: String,
            proto: String,
            csv_file: String,
            start_date: i32,
        ) -> Result<()>;

        //////////////////////////////////////////////////////////////////////////////////

        fn is_trading_day(self: &TradeCalendarPP, days_since_epoch: i32) -> Result<bool>;
        fn get_next_trading_day(
            self: &TradeCalendarPP,
            days_since_epoch: i32,
            num: usize,
        ) -> Result<i32>;
        fn get_prev_trading_day(
            self: &TradeCalendarPP,
            days_since_epoch: i32,
            num: usize,
        ) -> Result<i32>;
        fn get_trading_days_count(self: &TradeCalendarPP, start_dt: i32, end_dt: i32) -> usize;
        fn get_trading_days_list(self: &TradeCalendarPP, start_dt: i32, end_dt: i32) -> Vec<i32>;
        fn get_date_detail(self: &TradeCalendarPP, days_since_epoch: i32) -> Result<TradingDayPP>;
        fn trading_day_from_datetime(
            self: &TradeCalendarPP,
            datetime_nanos: i64,
            for_next: bool,
            is_finance_item: bool,
        ) -> Result<i32>;

        fn max_date(&self) -> i32;
        fn min_date(&self) -> i32;

        //////////////////////////////////////////////////////////////////////////////////////////////

        fn reset(self: &mut TradeCalendarPP, start_time_nanos: i64) -> Result<()>;
        fn is_trading(self: &TradeCalendarPP) -> bool;
        fn set_config(self: &mut TradeCalendarPP, cfg: &TradingCheckConfigPP) -> Result<()>;

        fn get_config(self: &TradeCalendarPP) -> TradingCheckConfigPP;

        fn time_changed(
            self: &mut TradeCalendarPP,
            datetime_nano: i64,
            fail_safe: bool,
        ) -> Result<TimeChangedResultPP>;

        fn prev_tday(self: &TradeCalendarPP) -> i32;
        fn current_tday(self: &TradeCalendarPP) -> i32;
        fn next_tday(self: &TradeCalendarPP) -> i32;

    }
}

impl TradeCalendarPP {
    /// 获取某个日期的交易日详细信息, 无状态
    /// date, morning, trading, night, next
    pub fn get_date_detail(
        self: &TradeCalendarPP,
        days_since_epoch: i32,
    ) -> Result<ffi::TradingDayPP> {
        let date = date_from_days_since_epoch(days_since_epoch);
        match self.entity.get_date_detail(&date) {
            Some(t) => Ok(ffi::TradingDayPP {
                date: date_to_days_since_epoch(&t.date),
                morning: t.morning,
                trading: t.trading,
                night: t.night,
                next: date_to_days_since_epoch(&t.next),
            }),
            None => Err(anyhow!(
                "detail for {}({}) not found",
                days_since_epoch,
                date
            )),
        }
    }

    /// 时间改变，重新计算内部状态
    ///
    /// fail_safe: 在失败时(主要是calendar没有及时更新的情况)尝试补救?
    ///
    /// 返回值: tuple(上个交易日, 当前交易日, 上个自然日, 当前自然日, Option<Error_Message>)
    ///
    pub fn time_changed(
        self: &mut TradeCalendarPP,
        datetime_nano: i64,
        fail_safe: bool,
    ) -> Result<ffi::TimeChangedResultPP> {
        let datetime = datetime_from_timestamp_nanos(datetime_nano);
        self.entity
            .time_changed(&datetime, fail_safe)
            .and_then(|r| {
                Ok(ffi::TimeChangedResultPP {
                    prev_tradeday: date_to_days_since_epoch(&r.0),
                    curr_tradeday: date_to_days_since_epoch(&r.1),
                    prev_date: date_to_days_since_epoch(&r.2),
                    curr_date: date_to_days_since_epoch(&r.3),
                    error_msg: r.4.unwrap_or_default(),
                })
            })
    }

    pub fn get_config(&self) -> ffi::TradingCheckConfigPP {
        let cfg = self.entity.get_config();
        ffi::TradingCheckConfigPP {
            tday_shift: time_to_midnight_nanos(&cfg.tday_shift),
            night_begin: time_to_midnight_nanos(&cfg.night_begin),
            night_end: time_to_midnight_nanos(&cfg.night_end),
            day_begin: time_to_midnight_nanos(&cfg.day_begin),
            day_end: time_to_midnight_nanos(&cfg.day_end),
        }
    }

    /// 重置日期边界的一些配置,
    /// 调用此函数之后，可以调用time_changed()刷新状态
    ///
    /// tday_shift: 交易日切换的时间点，缺省值 19:30:00, 影响trading_day()/prev_tday()/next_tday()
    ///
    /// 以下4个配置影响 is_trading()
    ///
    /// night_begin: 缺省值 20:30:00
    ///
    /// night_end: 缺省值 2:31:00
    ///
    /// day_begin: 缺省值 8:30:00
    ///
    /// day_end: 缺省值 15:30:00
    pub fn set_config(&mut self, cfg: &ffi::TradingCheckConfigPP) -> Result<()> {
        let cfg1 = tradecalendar::TradingCheckConfig {
            tday_shift: time_from_midnight_nanos(cfg.tday_shift),
            night_begin: time_from_midnight_nanos(cfg.night_begin),
            night_end: time_from_midnight_nanos(cfg.night_end),
            day_begin: time_from_midnight_nanos(cfg.day_begin),
            day_end: time_from_midnight_nanos(cfg.day_end),
        };
        self.entity.set_config(&cfg1)
    }
}
