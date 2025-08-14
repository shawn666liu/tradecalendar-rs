use anyhow::Result;
use odbc_api::{ConnectionOptions, Cursor, Environment, sys::Date};

use crate::Tradingday;
use crate::jcswitch::{MyDateType, make_date};

/// connection string example:
/// Driver={PostgreSQL Unicode};Server=localhost;PORT=5432;UID=username;PWD=password;Database=dbname

/// query: 5 fields required, keep the order of feilds,
/// select date,morning,trading,night,next from your_table where date>'yyyy-mm-dd' order by date
pub fn load_tradingdays_from_odbc(conn_string: &str, query: &str) -> Result<Vec<Tradingday>> {
    let env = Environment::new()?;

    let mut tradingdays = Vec::with_capacity(1024);
    let conn = env.connect_with_connection_string(conn_string, ConnectionOptions::default())?;

    let parameters = (); // This query does not use any parameters.
    let timeout_sec = None;

    if let Some(mut cursor) = conn.execute(query, parameters, timeout_sec)? {
        // Use cursor to process query results.
        let mut odbc_date = Date {
            year: 0,
            month: 0,
            day: 0,
        };

        // !!!注意: 索引是从1开始的，不是从0开始

        while let Some(mut row) = cursor.next_row()? {
            row.get_data(1, &mut odbc_date)?;

            let date = convert_odbc_date(&odbc_date);

            let mut morning = 0_i16;
            row.get_data(2, &mut morning)?;

            let mut trading = 0_i16;
            row.get_data(3, &mut trading)?;

            let mut night = 0_i16;
            row.get_data(4, &mut night)?;

            row.get_data(5, &mut odbc_date)?;

            let next = convert_odbc_date(&odbc_date);

            tradingdays.push(Tradingday {
                date,
                morning: morning != 0,
                trading: trading != 0,
                night: night != 0,
                next,
            });
        }
    }
    Ok(tradingdays)
}

fn convert_odbc_date(odbc_date: &Date) -> MyDateType {
    make_date(
        odbc_date.year as i32,
        odbc_date.month as u32,
        odbc_date.day as u32,
    )
}
