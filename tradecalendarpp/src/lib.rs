use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate};

use tradecalendar::jcswitch::date_from_i32;
use tradecalendar::{self, TradingdayCache};
// pub fn new_calendar(weixin_srv_url: String, appid: String, hostid: String) -> Box<NotifySender> {
//     Box::new(NotifySender {
//         sender: notifyz::NotifySender::new(Some(&weixin_srv_url), Some(&appid), Some(&hostid)),
//     })
// }

pub struct TradeCalendar {
    mgr: tradecalendar::TradeCalendar,
}
impl TradeCalendar {
    pub fn is_trading_day(self: &TradeCalendar, days_since_epoch: i32) -> Result<bool> {
        let date = date_from_i32(days_since_epoch);
        self.mgr.is_trading_day(&date)
    }
}

#[cxx::bridge(namespace = "tradecalendarpp")]
mod ffi {

    extern "Rust" {
        type TradeCalendar;
        fn is_trading_day(self: &TradeCalendar, days_since_epoch: i32) -> Result<bool>;

    }
}
