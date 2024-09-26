use anyhow::{anyhow, Result};
use arrow::array::{Array, BooleanArray, Date32Array, Int64Array};
use arrow::datatypes::DataType;
// use arrow_array::{Array, BooleanArray, Date32Array, Int64Array};
// use arrow_schema::DataType;
use connectorx::{get_arrow::get_arrow, source_router::SourceConn, sql::CXQuery};
use std::convert::TryFrom;
use std::sync::Arc;

use crate::time_helper::date_from_i32;
use crate::tradecalendar::Tradingday;

/// load Tradingday from db
/// conn format:  
/// 1) postgres://user:passwd@localhost:5432/dbname
/// 2) mysql://user:passwd@localhost:3306/dbname
///
/// query: 5 fields required, keep the order of feilds,
/// select date,morning,trading,night,next from your_table where xxx order by date;
pub fn load_tradingdays(conn: &str, query: &str, proto: Option<String>) -> Result<Vec<Tradingday>> {
    let mut source_conn = SourceConn::try_from(conn)?;
    if let Some(mode) = proto {
        source_conn.proto = mode;
    }
    let queries = &[CXQuery::from(query)];
    let destination = get_arrow(&source_conn, None, queries)?;
    let data = destination.arrow()?;
    let total = data.iter().fold(0, |acc, x| acc + x.num_rows());
    let mut res = Vec::with_capacity(total);
    for chunk in data.iter() {
        let schema = chunk.schema();
        println!("{:#?}", schema);

        // clickhouse没有bool类型,读取到的是i64,所以这里要判断是否boolean类型
        if schema.fields.len() != 5 {
            return Err(anyhow!("sql查询结果必须是5个字段,且保证顺序"));
        };
        let morning_fld = &schema.fields[1];
        let is_bool = morning_fld.data_type() == &DataType::Boolean;
        println!("is_bool: {}", is_bool);

        let arrarys = chunk.columns();
        let tdays = cast_to_concret_array::<Date32Array>(&arrarys[0], 0)?;
        let next = cast_to_concret_array::<Date32Array>(&arrarys[4], 4)?;

        let len = chunk.num_rows();
        if is_bool {
            let morning = cast_to_concret_array::<BooleanArray>(&arrarys[1], 1)?;
            let trading = cast_to_concret_array::<BooleanArray>(&arrarys[2], 2)?;
            let night = cast_to_concret_array::<BooleanArray>(&arrarys[3], 3)?;

            for idx in 0..len {
                let days = tdays.value(idx);
                let nxdays = next.value(idx);
                let rec = Tradingday {
                    date: date_from_i32(days),
                    morning: morning.value(idx),
                    trading: trading.value(idx),
                    night: night.value(idx),
                    next: date_from_i32(nxdays),
                };
                res.push(rec);
            }
        } else {
            // let type_: &DataType = schema.fields[1].data_type();
            let morning = cast_to_concret_array::<Int64Array>(&arrarys[1], 1)?;
            let trading = cast_to_concret_array::<Int64Array>(&arrarys[2], 2)?;
            let night = cast_to_concret_array::<Int64Array>(&arrarys[3], 3)?;

            for idx in 0..len {
                let days = tdays.value(idx);
                let nxdays = next.value(idx);
                let rec = Tradingday {
                    date: date_from_i32(days),
                    morning: morning.value(idx) > 0,
                    trading: trading.value(idx) > 0,
                    night: night.value(idx) > 0,
                    next: date_from_i32(nxdays),
                };
                res.push(rec);
            }
        }
    }

    Ok(res)
}

fn cast_to_concret_array<T: 'static>(arrary: &Arc<dyn Array>, index: usize) -> Result<&T> {
    let concret_array = arrary.as_any().downcast_ref::<T>().ok_or_else(|| {
        anyhow!(
            "cannot cast col_{} to {}",
            index,
            std::any::type_name::<T>()
        )
    })?;
    Ok(concret_array)
}

// fn to_bool_array(array: Arc<dyn Array>) -> Result<BooleanArray> {
//     let data_type = array.data_type();
//     match data_type {
//         DataType::Boolean => {
//             let x = Arc::get_mut(array);
//             let array_ = array.as_any().downcast_ref::<BooleanArray>().unwrap();
//             return Ok(array_);
//         }
//         DataType::Int64 => {
//             let values = array
//                 .as_any()
//                 .downcast_ref::<Int64Array>()
//                 .unwrap()
//                 .values()
//                 .iter()
//                 .map(|v| *v > 0)
//                 .collect::<Vec<_>>();
//             let res = BooleanArray::from(values);
//             return Ok(res);
//         }
//         _ => return Err(anyhow!("")),
//     }
// }
