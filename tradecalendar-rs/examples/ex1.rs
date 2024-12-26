use anyhow::Result;
use crossbeam_channel::{bounded, select, tick, Receiver};
use std::time::Duration;

use tradecalendar::{get_buildin_calendar, jcswitch::*};
use tradecalendar::{TradeCalendar, TradingdayCache};

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn main() -> Result<()> {
    let mut calendar: TradeCalendar = get_buildin_calendar(None)?;
    // optional, 设置交易日和is_trading切换边界
    // calendar.set_config(tday_shift, night_begin, night_end, day_begin, day_end);

    let now = get_now();
    let today = now.date();
    let prev_tday = calendar.get_prev_trading_day(&today, 1)?;
    let next_tday = calendar.get_next_trading_day(&today, 1)?;
    println!(
        "prev_tday was {}, next_tday will be {}",
        prev_tday.date, next_tday.date
    );
    let next_10_tday = calendar.get_next_trading_day(&today, 10)?;
    let tomorrow_ = tomorrow(&today);
    let tday_slice = calendar.get_trading_day_slice(&tomorrow_, &next_10_tday.date);
    println!("next 10 tradingdays will be");
    for tday in tday_slice {
        print!("{} ", tday.date);
    }
    println!("\nevery 60 seconds timer started, or press Ctrl+C to exit\n");

    let _ = calendar.time_changed(&now, true)?;
    let mut trading = calendar.is_trading();

    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_secs(60));

    loop {
        select! {
            recv(ticks) -> _ => {
                let now = get_now();
                let (old_tday, now_tday,old_date, now_date, opt_err) = calendar.time_changed(&now, true)?;
                if let Some(err) = opt_err {
                    println!("Error: {err}");
                }
                if old_date != now_date {
                    println!("自然日改变: {old_date} => {now_date}");
                }
                if old_tday != now_tday {
                    println!("交易日改变: {old_tday} => {now_tday}");
                }
                let now_trading = calendar.is_trading();
                if trading != now_trading{
                    println!("is_trading 改变: {trading} => {now_trading}");
                    trading = now_trading;
                    if trading {
                        // do something here, such as:
                        // check whether TraderApi is connected
                        // ...
                    }
                }
                println!("on_timer: {now}, tradingday {now_tday}, is_trading? {now_trading}\n");
            }
            recv(ctrl_c_events) -> _ => {
                println!("\nGoodbye!");
                break;
            }
        }
    }

    Ok(())
}
