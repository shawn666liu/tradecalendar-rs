use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use pyo3::Python;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use pyo3_stub_gen::define_stub_info_gatherer;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyfunction, gen_stub_pymethods};

use tradecalendar::{
    self, NotTradingSearchMethod, TradingdayCache, jcswitch::make_date, reload_calendar,
};

fn to_pyerr(e: anyhow::Error) -> PyErr {
    PyErr::new::<pyo3::exceptions::PyException, _>(e.to_string())
}

/// make struct and field public for other project using
///
/// like hotselectpy crate
#[gen_stub_pyclass]
#[pyclass]
pub struct TradeCalendar {
    pub entity: tradecalendar::TradeCalendar,
}

#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (start_date=None))]
pub fn get_buildin_calendar(start_date: Option<NaiveDate>) -> PyResult<TradeCalendar> {
    tradecalendar::get_buildin_calendar(start_date)
        .and_then(|r| Ok(TradeCalendar { entity: r }))
        .map_err(to_pyerr)
}

#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (csv_file, start_date=None))]
pub fn get_csv_calendar(csv_file: &str, start_date: Option<NaiveDate>) -> PyResult<TradeCalendar> {
    tradecalendar::get_csv_calendar(csv_file, start_date)
        .and_then(|r| Ok(TradeCalendar { entity: r }))
        .map_err(to_pyerr)
}

/// 连接字符串：   
///
/// postgres://user:passwd@localhost:5432/dbname  
///
/// mysql://user:passwd@localhost:3306/dbname   
///
/// clickhouse://user:passwd@localhost:8123/dbname
///
/// odbc: Driver={PostgreSQL Unicode};Server=localhost;PORT=5432;UID=user;PWD=passwd;Database=dbname
///
/// query: 5 fields required, keep the order of fields,
///
/// select date,morning,trading,night,next from your_table where date>='yyyy-mm-dd' order by date
#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (db_conn, query, csv_file=None, start_date=None))]
pub fn get_calendar(
    db_conn: &str,
    query: &str,
    csv_file: Option<String>,
    start_date: Option<NaiveDate>,
) -> PyResult<TradeCalendar> {
    tradecalendar::get_calendar(db_conn, query, csv_file, start_date, None)
        .and_then(|r| Ok(TradeCalendar { entity: r }))
        .map_err(to_pyerr)
}

#[gen_stub_pymethods]
#[pymethods]
impl TradeCalendar {
    /// 连接字符串：   
    ///
    /// postgres://user:passwd@localhost:5432/dbname  
    ///
    /// mysql://user:passwd@localhost:3306/dbname   
    ///
    /// clickhouse://user:passwd@localhost:8123/dbname
    ///
    /// odbc: Driver={PostgreSQL Unicode};Server=localhost;PORT=5432;UID=user;PWD=passwd;Database=dbname
    ///
    /// query: 5 fields required, keep the order of fields,
    ///
    /// select date,morning,trading,night,next from your_table where date>='yyyy-mm-dd' order by date
    #[pyo3(signature = (db_conn, query, csv_file=None, start_date=None))]
    fn reload(
        &mut self,
        db_conn: &str,
        query: &str,
        csv_file: Option<String>,
        start_date: Option<NaiveDate>,
    ) -> PyResult<()> {
        reload_calendar(&mut self.entity, db_conn, query, csv_file, start_date, None)
            .map_err(to_pyerr)
    }

    fn is_trading_day(&self, date: NaiveDate) -> PyResult<bool> {
        self.entity.is_trading_day(&date).map_err(to_pyerr)
    }
    #[pyo3(signature = (date, num=1))]
    fn get_next_trading_day(&self, date: NaiveDate, num: usize) -> PyResult<NaiveDate> {
        self.entity
            .get_next_trading_day(&date, num)
            .and_then(|t| Ok(t.date))
            .map_err(to_pyerr)
    }
    #[pyo3(signature = (date, num=1))]
    fn get_prev_trading_day(&self, date: NaiveDate, num: usize) -> PyResult<NaiveDate> {
        self.entity
            .get_prev_trading_day(&date, num)
            .and_then(|t| Ok(t.date))
            .map_err(to_pyerr)
    }
    /// 计算从start_date(含)到end_date(含)之间交易日的个数, 超出范围的部分将被忽略
    fn get_trading_days_count(&self, start_dt: NaiveDate, end_dt: NaiveDate) -> usize {
        self.entity.get_trading_days_count(&start_dt, &end_dt)
    }

    /// start_date(含)到end_date(含)之间交易日
    fn get_trading_days_list(&self, start_dt: NaiveDate, end_dt: NaiveDate) -> Vec<NaiveDate> {
        let r = self.entity.get_trading_day_slice(&start_dt, &end_dt);
        r.iter().map(|t| t.date).collect()
    }
    /// 获取某个日期的交易日详细信息
    /// date, morning, trading, night, next
    fn get_date_detail<'a>(&self, py: Python<'a>, theday: NaiveDate) -> Option<Bound<'a, PyDict>> {
        match self.entity.get_date_detail(&theday) {
            Some(t) => {
                let d = PyDict::new(py);
                let _ = d.set_item("date", t.date);
                let _ = d.set_item("morning", t.morning);
                let _ = d.set_item("trading", t.trading);
                let _ = d.set_item("night", t.night);
                let _ = d.set_item("next", t.next);
                return Some(d);
            }
            None => None,
        }
    }

    /// 根据输入时间获取交易日,
    ///
    /// 如果输入的时间点是非交易时段, 则利用method确定是取前一个交易日, 还是后一交易日,
    ///
    /// 交易时段内的时间点不受影响
    ///
    /// is_finance_item, 金融期货的下午收盘时间点为15:15, 其他商品15:00
    fn trading_day_from_datetime(
        &self,
        input: NaiveDateTime,
        for_next: bool,
        is_finance_item: bool,
    ) -> PyResult<NaiveDate> {
        let method = if for_next {
            NotTradingSearchMethod::Next
        } else {
            NotTradingSearchMethod::Prev
        };
        self.entity
            .trading_day_from_datetime(&input, method, is_finance_item)
            .map_err(to_pyerr)
    }

    fn max_date(&self) -> NaiveDate {
        self.entity
            .max_date()
            .and_then(|d| Some(*d))
            .unwrap_or_else(|| make_date(1970, 1, 1))
    }

    fn min_date(&self) -> NaiveDate {
        self.entity
            .min_date()
            .and_then(|d| Some(*d))
            .unwrap_or_else(|| make_date(1970, 1, 1))
    }

    //////////////////////////////////////////////////////////////////////////////////
    // 以下为有状态时的接口
    //////////////////////////////////////////////////////////////////////////////////
    #[pyo3(signature = (start_time=None))]
    fn reset(&mut self, start_time: Option<NaiveDateTime>) -> PyResult<()> {
        self.entity.reset(start_time.as_ref()).map_err(to_pyerr)
    }

    fn is_trading(&self) -> bool {
        self.entity.is_trading()
    }

    /// 时间改变，重新计算内部状态
    ///
    /// fail_safe: 在失败时(主要是calendar没有及时更新的情况)尝试补救?
    ///
    /// 返回值: tuple(上个交易日, 当前交易日, 上个自然日, 当前自然日, Option<Error_Message>)
    ///
    fn time_changed(
        &mut self,
        datetime: NaiveDateTime,
        fail_safe: bool,
    ) -> PyResult<(NaiveDate, NaiveDate, NaiveDate, NaiveDate, Option<String>)> {
        self.entity
            .time_changed(&datetime, fail_safe)
            .map_err(to_pyerr)
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
        tday_shift: NaiveTime,
        night_begin: NaiveTime,
        night_end: NaiveTime,
        day_begin: NaiveTime,
        day_end: NaiveTime,
    ) -> PyResult<()> {
        let cfg = tradecalendar::TradingCheckConfig {
            tday_shift,
            night_begin,
            night_end,
            day_begin,
            day_end,
        };
        self.entity.set_config(&cfg).map_err(to_pyerr)
    }

    /// 返回值参看set_config
    pub fn get_config(&self) -> (NaiveTime, NaiveTime, NaiveTime, NaiveTime, NaiveTime) {
        let cfg = self.entity.get_config();
        (
            cfg.tday_shift,
            cfg.night_begin,
            cfg.night_end,
            cfg.day_begin,
            cfg.day_end,
        )
    }

    /// 前一交易日
    pub fn prev_tday(&self) -> NaiveDate {
        *self.entity.prev_tday()
    }

    /// 获取当前交易日
    pub fn current_tday(&self) -> NaiveDate {
        *self.entity.current_tday()
    }

    /// 后一交易日
    pub fn next_tday(&self) -> NaiveDate {
        *self.entity.next_tday()
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn tradecalendarpy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_buildin_calendar, m)?)?;
    m.add_function(wrap_pyfunction!(get_csv_calendar, m)?)?;
    m.add_function(wrap_pyfunction!(get_calendar, m)?)?;
    m.add_class::<TradeCalendar>()?;
    Ok(())
}

// Define a function to gather stub information.
define_stub_info_gatherer!(stub_info);
