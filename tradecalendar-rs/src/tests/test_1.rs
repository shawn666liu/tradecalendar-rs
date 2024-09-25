#[cfg(test)]
mod tests {
    use std::fs;

    // use serde::forward_to_deserialize_any;
    use anyhow::Result;
    // use std::env;

    use crate::calendar_helper::*;
    use crate::common::MyDateType;

    #[test]
    fn gen_csv() -> Result<()> {
        let holidays = vec![
            "2022-01-03",
            "2022-01-31",
            "2022-02-01",
            "2022-02-02",
            "2022-02-03",
            "2022-02-04",
            "2022-04-04",
            "2022-04-05",
            "2022-05-02",
            "2022-05-03",
            "2022-05-04",
            "2022-06-03",
            "2022-09-12",
            "2022-10-03",
            "2022-10-04",
            "2022-10-05",
            "2022-10-06",
            "2022-10-07",
        ];

        #[cfg(feature = "with-chrono")]
        let holidays: Vec<MyDateType> = holidays
            .iter()
            .map(|&x| {
                MyDateType::parse_from_str(x, "%Y-%m-%d")
                    .expect(&format!("parse holiday error:{}", x))
            })
            .collect();

        #[cfg(feature = "with-jiff")]
        let holidays: Vec<MyDateType> = holidays
            .iter()
            .map(|&x| {
                MyDateType::strptime("%Y-%m-%d", x).expect(&format!("parse holiday error:{}", x))
            })
            .collect();

        let out_dir = "../target/tmp/sql";
        let td_lst = holidays_to_tradingdays(&holidays);
        gen_trade_day_csv(&td_lst, out_dir)?;
        let all_days = tradingdays_to_calendar(&td_lst);
        gen_calendar_csv(&all_days, out_dir)?;

        // for d in all_days.iter() {
        //     println!("{}", d);
        // }
        let path = fs::canonicalize(out_dir)?;
        println!("Finished. save to {}", path.display());
        Ok(())
    }
}
