use anyhow::{anyhow, Result};
// use connectorx::get_arrow2;
use arrow2::array::{Array, PrimitiveArray};
use arrow2::types::NativeType;
use connectorx::{get_arrow2::get_arrow2, source_router::SourceConn, sql::CXQuery};
use std::any::type_name;
use std::convert::TryFrom;

use crate::common::MyDateType;
use crate::tradecalendar::Tradingday;

#[cfg(feature = "with-chrono")]
pub fn date_from_i32(days: i32) -> MyDateType {
    return arrow2::temporal_conversions::date32_to_date(days);
}

#[cfg(feature = "with-jiff")]
pub fn date_from_i32(days: i32) -> MyDateType {
    use jiff::civil::Date;
    // Date::from_unix_epoch_days(days)

    const DAYS_FROM_0000_01_01_TO_1970_01_01: i64 = 719_468;
    const DAYS_IN_ERA: i64 = 146_097;
    let days: i64 = days.into();

    let days = days + DAYS_FROM_0000_01_01_TO_1970_01_01;
    let era = days / DAYS_IN_ERA;
    let day_of_era = days % DAYS_IN_ERA;
    let year_of_era = (day_of_era - day_of_era / (1_460) + day_of_era / (36_524)
        - day_of_era / (DAYS_IN_ERA - (1)))
        / (365);
    let year = year_of_era + era * (400);
    let day_of_year = day_of_era - ((365) * year_of_era + year_of_era / (4) - year_of_era / (100));
    let month = (day_of_year * (5) + (2)) / (153);
    let day = day_of_year - ((153) * month + (2)) / (5) + (1);

    let month = if month < 10 { month + (3) } else { month - (9) };
    let year = if month <= 2 { year + (1) } else { year };

    Date::constant(year as i16, month as i8, day as i8)
}

/// load Tradingday from db
/// conn format:  
/// 1) postgresql://user:passwd@localhost:5432/dbname
/// 2) clickhouse://user:passwd@localhost:5432/dbname
///
/// query: 5 fields required:
/// select date,morning,trading,night,next from your_table where xxx order by date;
pub fn load_calendar(conn: &str, query: &str, proto: Option<String>) -> Result<Vec<Tradingday>> {
    let mut source_conn = SourceConn::try_from(conn)?;
    if let Some(mode) = proto {
        source_conn.proto = mode;
    }
    let queries = &[CXQuery::from(query)];
    let destination = get_arrow2(&source_conn, None, queries)?;
    let (data, schema) = destination.arrow()?;
    print!("schema {:?}\n", schema);
    let total = data.iter().fold(0, |acc, x| acc + x.len());
    let mut res = Vec::with_capacity(total);
    for chunk in data.iter() {
        let len = chunk.len();
        let arrarys = chunk.arrays();
        if arrarys.len() != 5 {
            return Err(anyhow!("sql查询结果必须是5个字段,且保证顺序"));
        }
        let tdays = cast_to_primitive::<i32>(arrarys, 0)?;
        let morning = cast_to_primitive::<i64>(arrarys, 1)?;
        let trading = cast_to_primitive::<i64>(arrarys, 2)?;
        let night = cast_to_primitive::<i64>(arrarys, 3)?;
        let next = cast_to_primitive::<i32>(arrarys, 4)?;

        for idx in 0..len {
            let days = tdays.get(idx).ok_or_else(|| anyhow!("cast date"))?;
            let nxdays = next.get(idx).ok_or_else(|| anyhow!("cast nex"))?;
            let rec = Tradingday {
                date: date_from_i32(days),
                morning: morning.get(idx).ok_or_else(|| anyhow!("cast morning"))? > 0,
                trading: trading.get(idx).ok_or_else(|| anyhow!("cast trading"))? > 0,
                night: night.get(idx).ok_or_else(|| anyhow!("cast night"))? > 0,
                next: date_from_i32(nxdays),
            };
            res.push(rec);
        }
    }

    Ok(res)
}

pub fn cast_to_primitive<T: NativeType>(
    arrarys: &[Box<dyn Array>],
    index: usize,
) -> Result<&PrimitiveArray<T>> {
    let dyn_any = &arrarys[index];
    let primitive_array = dyn_any
        .as_any()
        .downcast_ref::<PrimitiveArray<T>>()
        .ok_or_else(|| anyhow!("cannot cast field_0 to {}", std::any::type_name::<T>()))?;
    Ok(primitive_array)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_read() -> Result<()> {
        // let conn = "postgresql://admin:Intel%40123@129.211.121.22:5432/futurebars";
        // let conn = "mysql://readonly:readonly@192.168.9.122:9004/futuredb";

        // let conn = "postgres://readonly:readonly@192.168.9.122:9005/futuredb";
        let conn = "mysql://readonly:readonly@192.168.100.208:9004/futuredb";
        let query = "select * from calendar limit 10";
        let res = load_calendar(conn, query, Some("text".into()))?;
        for td in res.iter() {
            print!("{:?}\n", td);
        }
        Ok(())
    }
}
