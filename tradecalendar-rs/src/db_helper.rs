use anyhow::{anyhow, Result};
use arrow::array::{Array, BooleanArray, Date32Array, Int64Array};
use arrow::datatypes::DataType;
use clickhouse::Client;
use clickhouse::Row;
use connectorx::{get_arrow::get_arrow, source_router::SourceConn, sql::CXQuery};
use serde::Deserialize;
use std::convert::TryFrom;
use std::sync::Arc;

use crate::jcswitch::date_from_days_since_epoch;
use crate::tradecalendar::Tradingday;

/// load Tradingday from db
/// conn format:  
/// 1) postgres://user:passwd@localhost:5432/dbname
/// 2) mysql://user:passwd@localhost:3306/dbname
///
/// query: 5 fields required, keep the order of feilds,
/// select date,morning,trading,night,next from your_table where date>'yyyy-mm-dd' order by date
///
/// for clickhouse
/// conn  -> "clickhouse://readonly:readonly@192.168.9.122:8123/futuredb"
/// query -> "SELECT ?fields FROM futuredb.calendar WHERE date>'2024-01-01' ORDER BY date"
pub fn load_tradingdays_from_db(
    conn: &str,
    query: &str,
    proto: Option<String>,
) -> Result<Vec<Tradingday>> {
    if conn.is_empty() || query.is_empty() {
        return Err(anyhow!("connection string or query is empty"));
    }

    if conn.to_lowercase().starts_with("clickhouse://") {
        return load_tradingdays_from_clickhouse(conn, query);
    }

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
                    date: date_from_days_since_epoch(days),
                    morning: morning.value(idx),
                    trading: trading.value(idx),
                    night: night.value(idx),
                    next: date_from_days_since_epoch(nxdays),
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
                    date: date_from_days_since_epoch(days),
                    morning: morning.value(idx) > 0,
                    trading: trading.value(idx) > 0,
                    night: night.value(idx) > 0,
                    next: date_from_days_since_epoch(nxdays),
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

#[allow(dead_code)]
#[derive(Row, Deserialize)]
struct TradingDayRow {
    pub date: u16,
    pub morning: bool,
    pub trading: bool,
    pub night: bool,
    pub next: u16,
}

fn to_tradingday(td: &TradingDayRow) -> Tradingday {
    Tradingday {
        date: date_from_days_since_epoch(td.date as i32),
        morning: td.morning,
        trading: td.trading,
        night: td.night,
        next: date_from_days_since_epoch(td.next as i32),
    }
}

/// conn -> clickhouse://user:passwd@localhost:8123/dbname
/// http client, so port must be 8123
/// query -> "SELECT ?fields FROM futuredb.calendar WHERE date>'yyyy-mm-dd' ORDER BY date"
/// https://github.com/ClickHouse/clickhouse-rs
pub fn load_tradingdays_from_clickhouse(conn: &str, query: &str) -> Result<Vec<Tradingday>> {
    // conn -> user:passwd@localhost:8123/dbname
    let connvec = conn
        .split("://")
        .last()
        .and_then(|s| Some(s.split('@').collect::<Vec<&str>>()))
        .ok_or_else(|| anyhow!("parse connection string failed"))?;
    // url_db -> ["localhost:8123", "dbname"]
    let url_db = connvec
        .last()
        .and_then(|s| Some(s.split("/").collect::<Vec<&str>>()))
        .ok_or_else(|| anyhow!("connection string has no url"))?;
    if url_db.len() != 2 {
        return Err(anyhow!("bad format for db url and database"));
    }

    let rt = tokio::runtime::Runtime::new()?;

    let res = rt.block_on(async {
        let mut client = Client::default()
            .with_url(format!("http://{}", url_db[0]))
            .with_database(url_db[1]);
        if connvec.len() > 1 {
            // may has user and password
            let up: Vec<_> = connvec[0].split(':').collect();
            if up.len() != 2 {
                return Err(anyhow!("bad format for db user and passwd"));
            }
            client = client.with_user(up[0]).with_password(up[1]);
        }
        let rows = client.query(query).fetch_all::<TradingDayRow>().await?;
        let result: Vec<_> = rows.iter().map(to_tradingday).collect();
        Ok(result)
    })?;

    return Ok(res);
}
