use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::{PyDate, PyList};

use tradecalendar::common::{make_date, MyDateType};
use tradecalendar::{self, get_buildin_calendar};

use tradecalendar::TradeCalendar as RsCalendar;

fn pydate_to_date(d: Py<PyDate>) -> Result<MyDateType> {
    // Ok(make_date(d.get_year(), d.get_month(), d.get_day()))
    Ok(make_date(2009, 1, 1))
}

#[pyclass]
struct TradeCalendar {
    entity: RsCalendar,
}

#[pymethods]
impl TradeCalendar {
    #[new]
    #[pyo3(signature = (use_buildin, buildin_start))]
    fn new(use_buildin: Option<bool>, buildin_start: Option<Py<PyDate>>) -> Self {
        let use_bltin = match use_buildin {
            Some(use_bltin) => use_bltin,
            None => false,
        };
        if use_bltin {
            // use build in csv file to load date list
            let mut start = make_date(2009, 1, 1);
            if let Some(t) = buildin_start {
                start = pydate_to_date(t).unwrap();
            }
            let entity = get_buildin_calendar(Some(start)).unwrap();
            Self { entity }
        } else {
            Self {
                entity: RsCalendar::new(),
            }
        }
    }
    // #[pyo3(signature=(full_days))]
    fn reload(&self, full_days: &Bound<'_, PyList>) {
        for pydate in full_days.iter() {}
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn tradecalendarpy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<TradeCalendar>()?;
    Ok(())
}
