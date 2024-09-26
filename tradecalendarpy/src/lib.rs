use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::Python;

use pyo3_stub_gen::{
    define_stub_info_gatherer, derive::gen_stub_pyclass, derive::gen_stub_pyfunction,
    derive::gen_stub_pymethods,
};

use tradecalendar::{self, get_buildin_calendar, get_calendar, get_csv_calendar, TradingdayCache};

use tradecalendar::TradeCalendar as RsCalendar;

struct PyAnyhowErr(anyhow::Error);

impl From<PyAnyhowErr> for PyErr {
    fn from(value: PyAnyhowErr) -> Self {
        PyErr::new::<pyo3::exceptions::PyException, _>(value.0.to_string())
    }
}

// impl Into<PyErr> for AnyhowError {
//     fn into(self) -> PyErr {
//         PyErr::new::<pyo3::exceptions::PyException, _>(self.0.to_string())
//     }
// }

impl From<anyhow::Error> for PyAnyhowErr {
    fn from(value: anyhow::Error) -> Self {
        Self(value)
    }
}

// impl Into<AnyhowError> for anyhow::Error {
//     fn into(self) -> AnyhowError {
//         AnyhowError(self)
//     }
// }

// fn pydate_to_date(py: Python, dt: Py<PyDate>) -> PyResult<NaiveDate> {
//     let d = dt.bind(py);
//     Ok(make_date(
//         d.get_year(),
//         d.get_month() as u32,
//         d.get_day() as u32,
//     ))
// }

// fn to_pydate<'a>(py: Python<'a>, d: &NaiveDate) -> Bound<'a, PyDate> {
//     PyDate::new_bound(py, d.year(), d.month() as u8, d.day() as u8).unwrap()
// }

fn to_pyerr(e: anyhow::Error) -> PyErr {
    PyErr::new::<pyo3::exceptions::PyException, _>(e.to_string())
}

#[gen_stub_pyclass]
#[pyclass]
#[allow(dead_code)]
struct TradeCalendar {
    entity: RsCalendar,
}

// #[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (start_date=None))]
fn load_buildin_calendar(start_date: Option<NaiveDate>) -> Result<TradeCalendar, PyAnyhowErr> {
    let calendar = get_buildin_calendar(start_date);
    match calendar {
        Ok(cal) => return Ok(TradeCalendar { entity: cal }),
        Err(e) => {
            return Err(e.into());
        }
    }
}

// #[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (csv_file, start_date=None))]
fn load_csv_calendar(
    csv_file: &str,
    start_date: Option<NaiveDate>,
) -> Result<TradeCalendar, PyAnyhowErr> {
    let calendar = get_csv_calendar(csv_file, start_date);
    match calendar {
        Ok(cal) => return Ok(TradeCalendar { entity: cal }),
        Err(e) => return Err(e.into()),
    }
}

// #[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(signature = (db_conn, query, proto=None, csv_file=None, start_date=None))]
fn load_calendar(
    db_conn: &str,
    query: &str,
    proto: Option<String>,
    csv_file: Option<String>,
    start_date: Option<NaiveDate>,
) -> Result<TradeCalendar, PyAnyhowErr> {
    let calendar = get_calendar(db_conn, query, proto, csv_file, start_date);
    match calendar {
        Ok(cal) => return Ok(TradeCalendar { entity: cal }),
        Err(e) => {
            return Err(e.into());
        }
    }
}

// #[gen_stub_pymethods]
#[pymethods]
impl TradeCalendar {
    fn is_trading_day(&self, date: NaiveDate) -> PyResult<bool> {
        self.entity.is_trading_day(&date).map_err(to_pyerr)
    }
    fn get_next_trading_day(&self, date: NaiveDate, num: usize) -> Result<NaiveDate, PyAnyhowErr> {
        let tday = self.entity.get_next_trading_day(&date, num);
        match tday {
            Ok(t) => Ok(t.date),
            Err(e) => Err(e.into()),
        }
    }
    fn get_prev_trading_day(&self, date: NaiveDate, num: usize) -> PyResult<NaiveDate> {
        let tday = self.entity.get_prev_trading_day(&date, num);
        match tday {
            Ok(t) => Ok(t.date),
            Err(e) => Err(to_pyerr(e)),
        }
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
                let d = PyDict::new_bound(py);
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

    //////////////////////////////////////////////////////////////////////////////////
    // 以下为有状态时的接口
    //////////////////////////////////////////////////////////////////////////////////

    fn reset(&mut self, start_time: Option<NaiveDateTime>) -> PyResult<()> {
        match self.entity.reset(start_time.as_ref()) {
            Ok(_) => Ok(()),
            Err(e) => Err(to_pyerr(e)),
        }
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
        match self.entity.time_changed(&datetime, fail_safe) {
            Ok(r) => Ok(r),
            Err(e) => Err(to_pyerr(e)),
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
    fn set_config(
        &mut self,
        tday_shift: NaiveTime,
        night_begin: NaiveTime,
        night_end: NaiveTime,
        day_begin: NaiveTime,
        day_end: NaiveTime,
    ) -> PyResult<()> {
        match self
            .entity
            .set_config(&tday_shift, &night_begin, &night_end, &day_begin, &day_end)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(to_pyerr(e)),
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn tradecalendarpy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(load_buildin_calendar, m)?)?;
    m.add_function(wrap_pyfunction!(load_csv_calendar, m)?)?;
    m.add_function(wrap_pyfunction!(load_calendar, m)?)?;
    m.add_class::<TradeCalendar>()?;
    Ok(())
}

// Define a function to gather stub information.
define_stub_info_gatherer!(stub_info);
