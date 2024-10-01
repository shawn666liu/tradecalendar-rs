use anyhow::{anyhow, Result};

use tradecalendar::jcswitch::{
    date_from_days_since_epoch, date_to_days_since_epoch, datetime_from_timestamp_nanos,
    time_from_midnight_nanos,
};
use tradecalendar::{
    self, get_buildin_calendar, get_calendar, get_csv_calendar, NotTradingSearchMethod,
    TradingdayCache,
};

pub struct TradeCalendar {
    entity: tradecalendar::TradeCalendar,
}

fn load_buildin_calendar(start_date: i32) -> Result<Box<TradeCalendar>> {
    let start_date = date_from_days_since_epoch(start_date);
    get_buildin_calendar(Some(start_date)).and_then(|r| Ok(Box::new(TradeCalendar { entity: r })))
}

fn load_csv_calendar(csv_file: String, start_date: i32) -> Result<Box<TradeCalendar>> {
    let start_date = date_from_days_since_epoch(start_date);
    get_csv_calendar(csv_file, Some(start_date))
        .and_then(|r| Ok(Box::new(TradeCalendar { entity: r })))
}

fn load_calendar(
    db_conn: String,
    query: String,
    proto: String,
    csv_file: String,
    start_date: i32,
) -> Result<Box<TradeCalendar>> {
    let start_date = date_from_days_since_epoch(start_date);
    let proto = if proto.is_empty() { None } else { Some(proto) };
    let csv_file = if csv_file.is_empty() {
        None
    } else {
        Some(csv_file)
    };
    get_calendar(&db_conn, &query, proto, csv_file, Some(start_date))
        .and_then(|r| Ok(Box::new(TradeCalendar { entity: r })))
}

impl TradeCalendar {
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

    //////////////////////////////////////////////////////////////////////////////////
    // 以下为有状态时的接口
    //////////////////////////////////////////////////////////////////////////////////

    pub fn reset(&mut self, start_time_nanos: i64) -> Result<()> {
        let datetime = datetime_from_timestamp_nanos(start_time_nanos);
        self.entity.reset(Some(&datetime))
    }

    pub fn is_trading(&self) -> bool {
        self.entity.is_trading()
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
    fn set_config(
        &mut self,
        tday_shift_nanos: i64,
        night_begin_nanos: i64,
        night_end_nanos: i64,
        day_begin_nanos: i64,
        day_end_nanos: i64,
    ) -> Result<()> {
        let tday_shift = time_from_midnight_nanos(tday_shift_nanos);
        let night_begin = time_from_midnight_nanos(night_begin_nanos);
        let night_end = time_from_midnight_nanos(night_end_nanos);
        let day_begin = time_from_midnight_nanos(day_begin_nanos);
        let day_end = time_from_midnight_nanos(day_end_nanos);
        self.entity
            .set_config(&tday_shift, &night_begin, &night_end, &day_begin, &day_end)
    }
}

#[cxx::bridge(namespace = "tradecalendarpp")]
mod ffi {
    struct TradingDay {
        date: i32,
        morning: bool,
        trading: bool,
        night: bool,
        next: i32,
    }

    struct TimeChangedResult {
        prev_tradeday: i32,
        curr_tradeday: i32,
        prev_date: i32,
        curr_date: i32,
        error_msg: String,
    }

    extern "Rust" {
        type TradeCalendar;
        fn load_buildin_calendar(start_date: i32) -> Result<Box<TradeCalendar>>;
        fn load_csv_calendar(csv_file: String, start_date: i32) -> Result<Box<TradeCalendar>>;
        fn load_calendar(
            db_conn: String,
            query: String,
            proto: String,
            csv_file: String,
            start_date: i32,
        ) -> Result<Box<TradeCalendar>>;
        //////////////////////////////////////////////////////////////////////////////////

        fn is_trading_day(self: &TradeCalendar, days_since_epoch: i32) -> Result<bool>;
        fn get_next_trading_day(
            self: &TradeCalendar,
            days_since_epoch: i32,
            num: usize,
        ) -> Result<i32>;
        fn get_prev_trading_day(
            self: &TradeCalendar,
            days_since_epoch: i32,
            num: usize,
        ) -> Result<i32>;
        fn get_trading_days_count(self: &TradeCalendar, start_dt: i32, end_dt: i32) -> usize;
        fn get_trading_days_list(self: &TradeCalendar, start_dt: i32, end_dt: i32) -> Vec<i32>;
        fn get_date_detail(self: &TradeCalendar, days_since_epoch: i32) -> Result<TradingDay>;
        fn trading_day_from_datetime(
            self: &TradeCalendar,
            datetime_nanos: i64,
            for_next: bool,
            is_finance_item: bool,
        ) -> Result<i32>;

        //////////////////////////////////////////////////////////////////////////////////////////////

        fn reset(self: &mut TradeCalendar, start_time_nanos: i64) -> Result<()>;
        fn is_trading(self: &TradeCalendar) -> bool;
        fn set_config(
            self: &mut TradeCalendar,
            tday_shift: i64,
            night_begin: i64,
            night_end: i64,
            day_begin: i64,
            day_end: i64,
        ) -> Result<()>;

        fn time_changed(
            self: &mut TradeCalendar,
            datetime_nano: i64,
            fail_safe: bool,
        ) -> Result<TimeChangedResult>;

    }
}

impl TradeCalendar {
    /// 获取某个日期的交易日详细信息, 无状态
    /// date, morning, trading, night, next
    pub fn get_date_detail(self: &TradeCalendar, days_since_epoch: i32) -> Result<ffi::TradingDay> {
        let date = date_from_days_since_epoch(days_since_epoch);
        match self.entity.get_date_detail(&date) {
            Some(t) => Ok(ffi::TradingDay {
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
        self: &mut TradeCalendar,
        datetime_nano: i64,
        fail_safe: bool,
    ) -> Result<ffi::TimeChangedResult> {
        let datetime = datetime_from_timestamp_nanos(datetime_nano);
        self.entity
            .time_changed(&datetime, fail_safe)
            .and_then(|r| {
                Ok(ffi::TimeChangedResult {
                    prev_tradeday: date_to_days_since_epoch(&r.0),
                    curr_tradeday: date_to_days_since_epoch(&r.1),
                    prev_date: date_to_days_since_epoch(&r.2),
                    curr_date: date_to_days_since_epoch(&r.3),
                    error_msg: r.4.unwrap_or_default(),
                })
            })
    }
}
