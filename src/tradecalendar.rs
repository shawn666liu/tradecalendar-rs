use anyhow::{anyhow, Context, Result};
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Weekday};
use csv::*;
use encoding_rs_io::DecodeReaderBytes;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::Display;
use std::fs::File;
use std::path::Path;
use std::result::Result::Ok;

/// 如果搜索的时间点“不在”交易时段内, 如何返回交易日:
///
/// 如果周五有夜盘,则周六周日对应的交易日一定是下周一,跟Method项无关,
/// 因为这个时间段不是NotTrading, 它是trading的持续; 交易日的午休时段也是trading,
///
/// 如果是在下午16:00收盘后到夜里19:00之间, 则跟Method选项有关, 因为这个时段是NotTrading;
/// 节假日时段由于放假之前最后一天是没有夜盘的, 所以也是NotTrading,
///
/// 返回前一日(Prev)还是后一日(Next)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotTradingSearchMethod {
    Next,
    Prev,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tradingday {
    pub date: NaiveDate,
    pub morning: bool,
    pub trading: bool,
    pub night: bool,
    pub next: NaiveDate,
}

impl Display for Tradingday {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dw = self.date.weekday();
        write!(
            f,
            "{}, {}, {}, {}, {}, {}",
            self.date,
            dw,
            if self.morning { 1 } else { 0 },
            if self.trading { 1 } else { 0 },
            if self.night { 1 } else { 0 },
            self.next,
        )
    }
}

impl Tradingday {
    /// create a dummy object
    pub fn new_dummy(date: &NaiveDate) -> Self {
        Self {
            date: date.clone(),
            morning: false,
            trading: false,
            night: false,
            next: next_working_day(date, 1),
        }
    }

    /// 文件头：date,morning,trading,night
    ///
    /// 行格式：2009-01-03,false,false,false,2009-01-04
    pub fn load_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Tradingday>> {
        let path = path.as_ref();
        let file = File::open(path).with_context(|| path.display().to_string())?;
        let v = Self::load_csv_read(DecodeReaderBytes::new(file))
            .with_context(|| path.display().to_string())?;
        Ok(v)
    }

    /// 可以直接从字符串加载, str.as_bytes()
    pub fn load_csv_read<R: std::io::Read>(read: R) -> Result<Vec<Tradingday>> {
        let mut rdr = Reader::from_reader(read);
        let mut v = vec![];
        for result in rdr.deserialize::<Tradingday>() {
            let record = result?;
            v.push(record);
        }
        Ok(v)
    }
}

/// 查找the_day在list(已排序)中的索引，及其左右值的索引，如果该索引无效，则为-1
pub fn search_days(list: &[Tradingday], the_day: &NaiveDate) -> (isize, isize, isize) {
    if list.is_empty() {
        return (-1, -1, -1);
    }

    let mut found = false;
    let mut first = 0_usize;
    let mut len = list.len();
    while len > 0 {
        let half = len >> 1;
        let mid = first + half;
        unsafe {
            let mid_item = list.get_unchecked(mid);
            // 仅比较日期部分
            match mid_item.date.cmp(the_day) {
                Ordering::Less => {
                    first = mid + 1;
                    len = len - half - 1;
                }
                Ordering::Equal => {
                    len = half;
                    found = true;
                }
                _ => len = half,
            }
        }
    }
    let mut _mid = first as isize;
    let mut _left = _mid - 1;
    let mut _right = -1_isize;
    match found {
        true => {
            if _mid < list.len() as isize - 1 {
                _right = _mid + 1;
            }
        }
        false => {
            if _mid < list.len() as isize {
                _right = _mid;
            }
            _mid = -1;
        }
    }
    return (_left, _mid, _right);
}

/// 获取下一(num)个工作日,即非周六周日的日期
/// 用于get_next_trading_day()失败之后，强制取工作日
pub fn next_working_day(the_day: &NaiveDate, num: usize) -> NaiveDate {
    assert!(num > 0);

    let mut next = the_day.clone();
    let mut numday = num;

    while numday > 0 {
        next = next.succ_opt().expect("succ_opt");
        if is_working_day(&next) {
            numday -= 1;
        }
    }
    return next;
}

/// 获取前一(num)个工作日,即非周六周日的日期
/// 用于get_prev_trading_day()失败之后，强制取工作日
pub fn prev_working_day(the_day: &NaiveDate, num: usize) -> NaiveDate {
    assert!(num > 0);

    let mut prev = the_day.clone();
    let mut numday = num;

    while numday > 0 {
        prev = prev.pred_opt().expect("pred_opt");
        if is_working_day(&prev) {
            numday -= 1;
        }
    }
    return prev;
}

/// the_day是否为工作日，非周六、周日
/// 用于is_trading_day()失败后，判断是否工作日
/// fail_safe一般都发生在新年初,年末忘记了更新calendar,导致取新一年的交易日失败
/// 而我们知道，一月一号元旦，一定是休假的，所以这里可以把元旦避开
fn is_working_day(the_day: &NaiveDate) -> bool {
    let week_day = the_day.weekday();
    let isfirst = the_day.month() == 1 && the_day.day() == 1;
    return week_day != Weekday::Sat && week_day != Weekday::Sun && !isfirst;
}

/// 内部是无状态的
pub trait TradingdayCache {
    /// 获取原始的日期列表(含非交易日)
    /// 主要用于期货，比如周六非交易日，但实际上夜盘会持续到周六凌晨
    fn get_full_day_list(&self) -> &Vec<Tradingday>;

    /// 获取交易日列表(仅含交易日)，主要用于股票，不含夜盘
    fn get_trading_day_list(&self) -> &Vec<Tradingday>;

    /// 获取两个日期之间的交易日的slice, 包含这两个交易日, 超出范围的部分将被忽略
    fn get_trading_day_slice(&self, start_dt: &NaiveDate, end_dt: &NaiveDate) -> &[Tradingday] {
        if start_dt > end_dt {
            panic!("start_dt {} needs less than end_dt {}", start_dt, end_dt)
        };
        let list = self.get_trading_day_list();
        let (_, mid, right) = search_days(list, start_dt);
        let istart = if mid >= 0 { mid } else { right };
        if istart < 0 {
            // empty slice
            return &list[0..0];
        }
        let (left, mid, _) = search_days(list, end_dt);
        // println!("end_dt is {}, left {}, mid {}", end_dt, left, mid);

        let iend = std::cmp::max(left, mid);
        if iend < 0 || istart > iend {
            return &list[0..0];
        };
        return &list[(istart as usize)..((iend + 1) as usize)];
    }

    /// 获取两个日期之间的所有日期(含非交易日)的slice, 包含这两个日期, 超出范围的部分将被忽略
    fn get_full_day_slice(&self, start_dt: &NaiveDate, end_dt: &NaiveDate) -> &[Tradingday] {
        if start_dt > end_dt {
            panic!("start_dt {} needs less than end_dt {}", start_dt, end_dt)
        };
        let list = self.get_full_day_list();
        let (_, mid, right) = search_days(list, start_dt);
        let istart = if mid >= 0 && right >= 0 {
            mid
        } else {
            std::cmp::max(mid, right)
        };
        if istart < 0 {
            return &list[0..0];
        }
        let (left, mid, _) = search_days(list, end_dt);
        let iend = std::cmp::max(left, mid);
        if iend < 0 || istart > iend {
            return &list[0..0];
        };
        return &list[(istart as usize)..((iend + 1) as usize)];
    }

    /// trade_day是否交易日
    fn is_trading_day(&self, trade_day: &NaiveDate) -> Result<bool> {
        let list = self.get_trading_day_list();
        let (_, mid, _) = search_days(list, trade_day);
        if mid >= 0 {
            return Ok(list[mid as usize].trading);
        }
        return Err(anyhow!("out of range"));
    }

    /// 获取后续第num个交易日, 要求num大于零
    fn get_next_trading_day(&self, the_day: &NaiveDate, num: usize) -> anyhow::Result<&Tradingday> {
        assert!(num > 0);

        // 由于trading_day_list数据较少，比直接查询full_day_list更快
        let list = self.get_trading_day_list();
        let (_, _, right) = search_days(list, the_day);
        if right >= 0 {
            let index = right as usize + num - 1;
            if index < list.len() {
                return Ok(&list[index]);
            }
        }
        return Err(anyhow!("out of range"));
    }

    /// 获取之前的第num个交易日，要求num大于零
    fn get_prev_trading_day(&self, the_day: &NaiveDate, num: usize) -> Result<&Tradingday> {
        assert!(num > 0);

        let list = self.get_trading_day_list();
        let (left, _, _) = search_days(list, the_day);
        if left >= 0 {
            let index = left + 1 - (num as isize);
            if index >= 0 {
                return Ok(&list[index as usize]);
            }
        }
        return Err(anyhow!("out of range"));
    }

    /// 计算从start_date(含)到end_date(含)之间交易日的个数, 超出范围的部分将被忽略
    fn get_trading_days_count(&self, start_dt: &NaiveDate, end_dt: &NaiveDate) -> usize {
        if start_dt > end_dt {
            panic!("start_dt {} needs less than end_dt {}", start_dt, end_dt)
        };
        let list = self.get_trading_day_list();
        let (_, mid, right) = search_days(list, start_dt);
        let start_index = if mid >= 0 { mid } else { right };
        if start_index < 0 {
            return 0;
        };
        let (left, mid, _) = search_days(list, end_dt);
        let end_index = if mid >= 0 { mid } else { left };
        if end_index < 0 {
            return 0;
        }
        return (end_index - start_index + 1) as usize;
    }

    /// 根据输入时间获取交易日,
    ///
    /// 如果输入的时间点是非交易时段, 则利用method确定是取前一个交易日, 还是后一交易日,
    ///
    /// 交易时段内的时间点不受影响
    ///
    /// is_finance_item, 金融期货的下午收盘时间点为15:15, 其他商品15:00
    fn trading_day_by_datetime(
        &self,
        input: &NaiveDateTime,
        method: NotTradingSearchMethod,
        is_finance_item: bool,
    ) -> Result<NaiveDate> {
        let list = self.get_full_day_list();
        let date = input.date();
        let (_, index, _) = search_days(list, &date);
        if index < 0 {
            return Err(anyhow!("out of range"));
        };

        let index = index as usize;
        let tday = &list[index];
        assert!(tday.date == date);
        // 如果是金融期货，下午收盘时间15：15，否则收盘时间15：00
        let day_after = if is_finance_item {
            15 * 3600 + 15 * 60
        } else {
            15 * 3600
        };

        let time = input.time();
        let secs = time.num_seconds_from_midnight();
        if secs < 3600 * 9 {
            // [0:00, 09:00)
            if tday.morning {
                if tday.trading {
                    // 周二~周五，既有凌晨盘，也有日盘
                    return Ok(tday.date);
                } else {
                    // 周六早上, 有凌晨盘, 但无日盘
                    return Ok(tday.next);
                };
            } else {
                // 1） trading==true, 无凌晨盘， 有日盘, 周一早上或假期结束早上, 取决于上一交易日是否有夜盘
                // 2） trading==false, 无凌晨盘，也无日盘, 周日，节假日, 取决于上一交易日是否有夜盘
                return Ok(self.__by_prev_tday(index, tday, method));
            }
        } else if secs <= day_after {
            // [9:00, 15:00] 或者 [9:00, 15:15]
            if tday.trading {
                // 有日盘
                return Ok(tday.date);
            } else if tday.morning {
                // 周六早上, 有凌晨盘, 但无日盘
                return Ok(tday.next);
            } else {
                return Ok(self.__by_prev_tday(index, tday, method));
            }
        } else if secs < 21 * 3600 {
            // (15:00/15:15, 21:00)
            if tday.trading {
                return Ok(match method {
                    NotTradingSearchMethod::Next => tday.next,
                    NotTradingSearchMethod::Prev => tday.date,
                });
            } else {
                return Ok(self.__by_prev_tday(index, tday, method));
            }
        } else {
            // [21:00, 24:00)
            if tday.trading {
                if tday.night {
                    // 有日盘，有夜盘，一定是周一至周五夜里
                    return Ok(tday.next);
                } else {
                    // 有日盘, 无夜盘, 一定是放假前夜
                    return Ok(match method {
                        NotTradingSearchMethod::Next => tday.next,
                        NotTradingSearchMethod::Prev => tday.date,
                    });
                }
            } else {
                assert_eq!(tday.night, false);
                // 无日盘则必然无夜盘，周六或者周日夜里或者节假日
                // 向前回退到最近交易日来判断
                return Ok(self.__by_prev_tday(index, tday, method));
            }
        }
    }

    /// 在full_day_list里面，快速找到上一个交易日, 调用get_prev_trading_day()开销太大
    ///
    /// full_list_idx开始的下标(不含),向前搜索
    fn __fast_prev_trading_day(&self, full_list_idx: usize) -> Option<&Tradingday> {
        let list = self.get_full_day_list();
        let mut index = full_list_idx;
        while index > 0 {
            index -= 1;
            let tmp = &list[index];
            if tmp.trading {
                return Some(tmp);
            }
        }
        None
    }

    /// 从index向前找上一个交易日, 视其夜盘的情况，确定交易日信息
    fn __by_prev_tday(
        &self,
        index: usize,
        tday: &Tradingday,
        method: NotTradingSearchMethod,
    ) -> NaiveDate {
        let res = match self.__fast_prev_trading_day(index) {
            Some(prev_tday) => {
                if prev_tday.night {
                    prev_tday.next
                } else {
                    match method {
                        NotTradingSearchMethod::Next => prev_tday.next,
                        NotTradingSearchMethod::Prev => prev_tday.date,
                    }
                }
            }
            None => {
                // out of range, not possible here
                if tday.trading {
                    tday.date
                } else {
                    tday.next
                }
            }
        };
        return res;
    }
}

/// 用来检测当前时间点是否交易, 及交易日切换的一些配置项
struct TradingCheckConfig {
    /// 夜盘属于下一个交易日，这个变量指示什么时间点进行切换，一般是夜里19:00~20点，缺省19:30
    pub tday_shift: NaiveTime,

    //-------------------- begin 以下几个字段用来判断接口是否应该处于连接状态
    /// 缺省夜里 20:30
    pub _night_begin: NaiveTime,
    /// 缺省凌晨 2:31
    pub _night_end: NaiveTime,
    /// 缺省早上 8:30
    pub _day_begin: NaiveTime,
    /// 缺省下午 15:30
    pub _day_end: NaiveTime,
    //-------------------- end
}
impl Default for TradingCheckConfig {
    fn default() -> Self {
        Self {
            tday_shift: NaiveTime::from_hms_opt(19, 30, 0).expect("fromhms"),
            _night_begin: NaiveTime::from_hms_opt(20, 30, 0).expect("fromhms"),
            _night_end: NaiveTime::from_hms_opt(2, 31, 0).expect("fromhms"),
            _day_begin: NaiveTime::from_hms_opt(8, 30, 0).expect("fromhms"),
            _day_end: NaiveTime::from_hms_opt(15, 30, 0).expect("fromhms"),
        }
    }
}

/// 内部是有状态的，维护着当前自然日，交易日等信息
/// 如果交易日当天有夜盘，则self.cfg._night_begin作为下一个TradingDay的开始
/// 如果交易日当天没有夜盘，则夜里23:59:59之后的0点作为下一个TradingDay的开始
/// 非交易日，直接取next做为Tradingday
///
/// 外部触发trading状态切换、交易日更改的函数为 time_changed()，返回值：tuple(自然日是否改变，交易日是否改变)
/// 若返回值中含有true, 则有状态改变，调用方可采取相应动作
pub struct TradeCalendar {
    full_day_list: Vec<Tradingday>,
    trading_day_list: Vec<Tradingday>,

    /// 当前自然日及时间
    current_time: NaiveDateTime,
    /// 当前时间点, 交易接口是否可连接（CTP服务器开放时段）
    is_trading: bool,
    cfg: TradingCheckConfig,

    curr_tday: NaiveDate,
    next_tday: NaiveDate,
    prev_tday: NaiveDate,
}

impl Default for TradeCalendar {
    fn default() -> Self {
        // trading_day_list列表不能为空，创建一个远古的日期
        let dummy = Tradingday::new_dummy(&NaiveDate::default());

        // 将当前交易日设置为无效值的意义:
        // 在time_changed()里面，与实际交易日比较时，才不会相同，才能被重新赋值
        // current_time 同理
        Self {
            curr_tday: NaiveDate::MIN,
            current_time: NaiveDateTime::MIN,
            is_trading: false,
            full_day_list: vec![dummy],
            trading_day_list: Default::default(),
            next_tday: Default::default(),
            prev_tday: Default::default(),
            cfg: Default::default(),
        }
    }
}

impl TradingdayCache for TradeCalendar {
    fn get_full_day_list(&self) -> &Vec<Tradingday> {
        return &self.full_day_list;
    }

    fn get_trading_day_list(&self) -> &Vec<Tradingday> {
        return &self.trading_day_list;
    }
}

impl TradeCalendar {
    /// 使用new创建之后，紧接着调用reload()进行初始化
    pub fn new() -> Self {
        Default::default()
    }

    /// 当前时间是否在CTP服务器可连接时段
    pub fn is_trading(&self) -> bool {
        self.is_trading
    }

    /// 前一交易日
    pub fn prev_tday(&self) -> &NaiveDate {
        &self.prev_tday
    }

    /// 获取当前交易日
    pub fn current_tday(&self) -> &NaiveDate {
        &self.curr_tday
    }

    /// 后一交易日
    pub fn next_tday(&self) -> &NaiveDate {
        &self.next_tday
    }

    /// 获取最近设置的自然日(区别于交易日)及其时间，可用于回溯模式
    pub fn current_time(&self) -> &NaiveDateTime {
        &self.current_time
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
    pub fn set_config(
        &mut self,
        tday_shift: &NaiveTime,
        night_begin: &NaiveTime,
        night_end: &NaiveTime,
        day_begin: &NaiveTime,
        day_end: &NaiveTime,
    ) -> Result<()> {
        if tday_shift >= &NaiveTime::from_hms_opt(21, 0, 0).expect("fromhms")
            || tday_shift <= &NaiveTime::from_hms_opt(16, 0, 0).expect("fromhms")
        {
            return Err(anyhow!("TradingdayManager: `tday_shift`一般在夜里19~20."));
        }
        if night_begin < day_end {
            return Err(anyhow!(
                "TradingdayManager: `night_begin` should big than `day_end`."
            ));
        }
        if day_end <= day_begin {
            return Err(anyhow!(
                "TradingdayManager: `day_end` should big than `day_begin`."
            ));
        }
        if day_begin <= night_end {
            return Err(anyhow!(
                "TradingdayManager: `day_begin` should big than `night_end`."
            ));
        }

        self.cfg.tday_shift = tday_shift.clone();
        self.cfg._night_begin = night_begin.clone();
        self.cfg._night_end = night_end.clone();
        self.cfg._day_begin = day_begin.clone();
        self.cfg._day_end = day_end.clone();
        Ok(())
    }

    /// 重新加载交易日历列表，年末时交易日历需更新，使用此函数日常重新加载
    /// 调用此函数之后，可以调用time_changed()刷新状态
    pub fn reload(&mut self, full_list: Vec<Tradingday>) -> Result<()> {
        if full_list.is_empty() {
            return Err(anyhow!("TradingdayManager: full_list can't be empty."));
        }
        self.trading_day_list = full_list
            .iter()
            .filter(|td| td.trading)
            .map(|x| x.clone())
            .collect();
        self.full_day_list = full_list;
        Ok(())
    }

    /// 仅用于回溯模式
    ///
    /// 重置内部状态，以便重新开始
    pub fn reset(&mut self, start_time: Option<&NaiveDateTime>) -> Result<()> {
        let td = &self.full_day_list[0];
        let current_time = td.date.and_hms_opt(0, 0, 0).expect("andhms");
        self.curr_tday = NaiveDate::MIN;
        self.time_changed(start_time.unwrap_or(&current_time), true)?;
        Ok(())
    }

    /// 时间改变，重新计算内部状态
    ///
    /// fail_safe: 在失败时(主要是calendar没有及时更新的情况)尝试补救?
    ///
    /// 返回值: tuple(上个交易日, 当前交易日, 上个自然日, 当前自然日, Option<Error_Message>)
    ///    
    pub fn time_changed(
        &mut self,
        datetime: &NaiveDateTime,
        fail_safe: bool,
    ) -> Result<(NaiveDate, NaiveDate, NaiveDate, NaiveDate, Option<String>)> {
        // println!("time_changed() called.");
        let curr_date = datetime.date();
        let old_date = self.current_time.date();
        if old_date != curr_date {
            log::trace!("自然日改变: {} => {}", old_date, curr_date);
        }

        let mut error_msg: Option<String> = None;

        let calendar: Tradingday;
        let (_, index, _) = search_days(&self.full_day_list, &curr_date);
        if index >= 0 {
            calendar = (self.full_day_list[index as usize]).clone();
        } else {
            let min_dt = &self.full_day_list[0].date;
            let max_dt = &self.full_day_list.last().expect("no fail").date;
            if &curr_date > min_dt && &curr_date < max_dt {
                error_msg = Some(format!(
                    "TradingdayManager: full_days_list ({} ~ {}), 缺少数据 {}",
                    min_dt, max_dt, &curr_date,
                ));
            } else {
                error_msg = Some(format!(
                    "TradingdayManager: full_days_list ({} ~ {}), out of range for {}",
                    min_dt, max_dt, &curr_date,
                ));
            }
            if fail_safe {
                calendar = self.fail_safe_tradingday(&curr_date);
            } else {
                return Err(anyhow!(error_msg.expect("no fail")));
            }
        }

        self.current_time = datetime.clone();
        let time = datetime.time();

        // 如果交易日当天有夜盘，则self.cfg.tday_shift作为下一个TradingDay的开始
        // 如果交易日当天没有夜盘，则夜里23:59:59之后的0点作为下一个TradingDay的开始
        // 非交易日，直接取next做为Tradingday
        let current_tday = if calendar.trading {
            if calendar.night && time >= self.cfg.tday_shift {
                calendar.next
            } else {
                calendar.date
            }
        } else {
            // 非交易日
            calendar.next
        };
        let trading = self.check_is_trading(&time, &calendar);
        self.set_is_trading(trading);

        let old_tday = self.curr_tday;
        if old_tday != current_tday {
            self.curr_tday = current_tday;
            // get_prev_trading_day()的错误无需汇报，因为我们time_changed总是向前推进
            self.prev_tday = match self.get_prev_trading_day(&current_tday, 1) {
                Ok(pretday) => pretday.date,
                Err(_) => prev_working_day(&current_tday, 1),
            };
            match self.get_next_trading_day(&current_tday, 1) {
                Ok(day) => {
                    self.next_tday = day.date;
                }
                Err(_) => {
                    // 这个可能会发生在年末岁初,calendar没有及时更新的情况下
                    if fail_safe {
                        error_msg = Some(format!(
                            "out of range when get next for {}. 请更新交易日历",
                            &current_tday
                        ));
                        self.next_tday = next_working_day(&current_tday, 1);
                    } else {
                        return Err(anyhow!(
                            "TradingdayManager::time_change(), out of range for full_days_list"
                        ));
                    }
                }
            };
            log::info!(
                "交易日改变: {} => {}, prev {}, next {}, shift point {}",
                old_tday,
                self.curr_tday,
                self.prev_tday,
                self.next_tday,
                self.cfg.tday_shift
            );
        }
        Ok((old_tday, current_tday, old_date, curr_date, error_msg))
    }

    /// 重算is_trading变量, 当前Tradingday已知
    fn check_is_trading(&self, time: &NaiveTime, tday: &Tradingday) -> bool {
        if time >= &self.cfg._night_begin {
            return tday.night;
        }
        if time <= &self.cfg._night_end {
            return tday.morning;
        }
        if time >= &self.cfg._day_begin && time <= &self.cfg._day_end {
            return tday.trading;
        }
        return false;
    }

    fn set_is_trading(&mut self, trading: bool) {
        if self.is_trading != trading {
            log::info!("is_trading changed: {} -> {}", self.is_trading, trading);
            self.is_trading = trading;
        }
    }

    /// 已经超出了full_day_list的范围, 只能按照working day的方式, 构造一个范围外的Tradingday
    fn fail_safe_tradingday(&mut self, input: &NaiveDate) -> Tradingday {
        // 需要构造出一个Tradingday对象出来
        let weekday = input.weekday();
        let mut calendar = Tradingday::new_dummy(&input);
        calendar.trading = is_working_day(&input);

        // 如果白天没有交易的话，则一定没有夜盘
        // 如果白天有交易，则夜盘取决于后续是否有公共假期，但是这里显然无法获取假期数据
        calendar.night = calendar.trading;

        // 一般白天有交易时都会有早盘，除了以下三种情况
        calendar.morning = calendar.trading;
        // 1) 周五的夜盘持续到周六早上，但周六白天不交易
        if weekday == Weekday::Sat {
            calendar.morning = true;
        }
        // 2) 周一白天有交易，但显然没有早盘
        else if weekday == Weekday::Mon {
            calendar.morning = false;
        }
        // 3) 从公共假期到input之间，没有工作日的话，则没有早盘， 因为放假前一天没有夜盘的。
        // 这里能确定的假日就是元旦，五一国庆当然也能确定，但五一国庆的时候肯定已经更新交易日历了吧，
        // 所以这里只检查元旦就OK了
        let first_day = NaiveDate::from_ymd_opt(input.year(), 1, 1).unwrap();
        let mut theday = input.clone() - chrono::Duration::days(1);
        let mut has_working_day = false;
        while theday > first_day {
            if is_working_day(&theday) {
                has_working_day = true;
                break;
            }
            theday -= chrono::Duration::days(1);
        }
        if !has_working_day {
            calendar.morning = false;
        }

        // println!("{}", calendar);

        return calendar;
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    // use crate::CSV;

    #[test]
    #[allow(unused_variables)]
    fn test_calendar() -> anyhow::Result<()> {
        let buf = "date,morning,trading,night,next
2021-01-01,false,false,false,2021-01-04
2021-01-02,false,false,false,2021-01-04
2021-01-03,false,false,false,2021-01-04
2021-01-04,false,true,true,2021-01-05
2021-01-05,true,true,true,2021-01-06
2021-01-06,true,true,true,2021-01-07
2021-01-07,true,true,true,2021-01-08
2021-01-08,true,true,true,2021-01-11
2021-01-09,true,false,false,2021-01-11
2021-01-10,false,false,false,2021-01-11
2021-01-11,false,true,true,2021-01-12
2021-01-12,true,true,true,2021-01-13
2021-01-13,true,true,true,2021-01-14
2021-01-14,true,true,true,2021-01-15
2021-01-15,true,true,true,2021-01-18
2021-01-16,true,false,false,2021-01-18
2021-01-17,false,false,false,2021-01-18
2021-01-18,false,true,true,2021-01-19
2021-01-19,true,true,true,2021-01-20
2021-01-20,true,true,true,2021-01-21
2021-01-21,true,true,true,2021-01-22
2021-01-22,true,true,true,2021-01-25
2021-01-23,true,false,false,2021-01-25
2021-01-24,false,false,false,2021-01-25
2021-01-25,false,true,true,2021-01-26
2021-01-26,true,true,true,2021-01-27
2021-01-27,true,true,true,2021-01-28
2021-01-28,true,true,true,2021-01-29
2021-01-29,true,true,true,2021-02-01
2021-01-30,true,false,false,2021-02-01
2021-01-31,false,false,false,2021-02-01
2021-02-01,false,true,true,2021-02-02
2021-12-29,true,true,true,2021-12-30
2021-12-30,true,true,true,2021-12-31
2021-12-31,true,true,false,2022-01-03";

        // 以上数据:
        // 1） 从 2021-02-01 后开始缺失，
        // 2） 由于 2021-12-31 这条数据实际是在2020年12月左右生成的, 其后一交易日当时只能推断出是2022-01-03,
        // 实际情况是2022年的公共假日在2021年12月左右公布，2022-01-03是节假日, 这个位置正确的值是2022-01-04
        // 不更新交易日历，仅通过fail_safe，是无法修正这个数据的

        let y20201230 = NaiveDate::from_ymd_opt(2020, 12, 30).expect("chrono");
        let y20201231 = NaiveDate::from_ymd_opt(2020, 12, 31).expect("chrono");
        let y20210101 = NaiveDate::from_ymd_opt(2021, 1, 1).expect("chrono");
        let y20210102 = NaiveDate::from_ymd_opt(2021, 1, 2).expect("chrono");
        let y20210104 = NaiveDate::from_ymd_opt(2021, 1, 4).expect("chrono");
        let y20210105 = NaiveDate::from_ymd_opt(2021, 1, 5).expect("chrono");
        let y20210108 = NaiveDate::from_ymd_opt(2021, 1, 8).expect("chrono");
        let y20210111 = NaiveDate::from_ymd_opt(2021, 1, 11).expect("chrono");
        let y20210120 = NaiveDate::from_ymd_opt(2021, 1, 20).expect("chrono");
        let y20210121 = NaiveDate::from_ymd_opt(2021, 1, 21).expect("chrono");
        let y20210202 = NaiveDate::from_ymd_opt(2021, 2, 2).expect("chrono");
        let y20211231 = NaiveDate::from_ymd_opt(2021, 12, 31).expect("chrono");
        let y20220101 = NaiveDate::from_ymd_opt(2022, 1, 1).expect("chrono");
        let y20220102 = NaiveDate::from_ymd_opt(2022, 1, 2).expect("chrono");
        let y20220103 = NaiveDate::from_ymd_opt(2022, 1, 3).expect("chrono");
        let y20220104 = NaiveDate::from_ymd_opt(2022, 1, 4).expect("chrono");
        let y20240101 = NaiveDate::from_ymd_opt(2024, 1, 1).expect("chrono");

        let list = Tradingday::load_csv_read(buf.as_bytes())?;
        // println!("{:?}", list);

        // let wtr = CSV::save_csv_write(Vec::new(), list)?;
        // let data = String::from_utf8(wtr.into_inner()?)?;
        // println!("\ncsv result is\n{}", data);

        let mut mgr = TradeCalendar::new();
        mgr.set_config(
            &NaiveTime::from_hms_opt(19, 30, 0).expect("fromhms"),
            &NaiveTime::from_hms_opt(20, 30, 0).expect("fromhms"),
            &NaiveTime::from_hms_opt(2, 31, 0).expect("fromhms"),
            &NaiveTime::from_hms_opt(8, 30, 0).expect("fromhms"),
            &NaiveTime::from_hms_opt(15, 30, 0).expect("fromhms"),
        )?;

        mgr.reload(list)?;
        let td = mgr.get_next_trading_day(&y20210101, 1)?;
        assert_eq!(td.date, y20210104,);
        let td = mgr.get_next_trading_day(&y20210102, 1)?;
        assert_eq!(td.date, y20210104,);
        let td = mgr.get_next_trading_day(&y20210104, 1)?;
        assert_eq!(td.date, y20210105);
        let td = mgr.get_next_trading_day(&y20210104, 4)?;
        assert_eq!(td.date, y20210108);
        let td = mgr.get_next_trading_day(&y20210104, 5)?;
        assert_eq!(td.date, y20210111);
        let td = mgr.get_next_trading_day(&y20210108, 1)?;
        assert_eq!(td.date, y20210111);
        let td = mgr.get_prev_trading_day(&y20210108, 3)?;
        assert_eq!(td.date, y20210105);

        let datetime = y20210105.and_hms_opt(9, 10, 5).unwrap();
        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, false)?;
        assert_ne!(old_date, curr_date);
        assert_ne!(old_tday, curr_tday);
        assert!(opt_err.is_none());
        assert_eq!(mgr.is_trading(), true);

        let datetime = y20210108.and_hms_opt(19, 28, 30).unwrap();
        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, false)?;
        assert_ne!(old_tday, curr_tday);
        assert_eq!(mgr.current_tday(), &y20210108);
        assert_eq!(mgr.is_trading(), false);

        let datetime = y20210108.and_hms_opt(19, 29, 30).unwrap();
        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, false)?;
        assert_eq!(old_date, curr_date);
        assert_eq!(old_tday, curr_tday);
        assert_eq!(mgr.current_tday(), &y20210108);
        assert_eq!(mgr.is_trading(), false);

        let datetime = y20210108.and_hms_opt(19, 30, 0).unwrap();
        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, false)?;
        assert_ne!(old_tday, curr_tday);
        assert_eq!(mgr.current_tday(), &y20210111);
        assert_eq!(mgr.is_trading(), false);

        let datetime = y20210108.and_hms_opt(20, 30, 0).unwrap();
        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, false)?;
        assert_eq!(old_tday, curr_tday);
        assert_eq!(mgr.current_tday(), &y20210111);
        assert_eq!(mgr.is_trading(), true);

        // 中间数据缺失
        let datetime = y20210202.and_hms_opt(0, 0, 0).unwrap();
        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, true)?;
        assert!(opt_err.is_some());
        // println!("{}", opt_err.unwrap());
        assert_eq!(mgr.current_tday(), &y20210202);

        // fail_safe

        // missing before, 这种情况在实盘一般是不会出现的，因为日期总是向后推进，不会向前
        let datetime = y20201230.and_hms_opt(20, 30, 0).unwrap();
        let res = mgr.time_changed(&datetime, false);
        assert!(res.is_err());

        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, true)?;
        assert_ne!(old_tday, curr_tday);
        assert!(opt_err.is_some());
        assert_eq!(mgr.current_tday(), &y20201231);
        assert_eq!(mgr.is_trading(), true);

        // missing after, 实盘可能遭遇这种情况,
        let datetime = y20211231.and_hms_opt(10, 30, 0).unwrap();
        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, true)?;
        // println!(
        //     "({}) => {}, {}, {}, {}",
        //     datetime, old_tday, curr_tday, old_date, curr_date
        // );

        // 由于12月31日后面是元旦，是假期， 所以12月31日没有夜盘，交易日是不会切换的
        let datetime = y20211231.and_hms_opt(21, 30, 0).unwrap();
        let (old_tday, curr_tday, old_date, curr_date, opt_err) =
            mgr.time_changed(&datetime, true)?;
        // println!(
        //     "({}) => {}, {}, {}, {}",
        //     datetime, old_tday, curr_tday, old_date, curr_date
        // );
        assert!(opt_err.is_none());
        assert_eq!(mgr.current_tday(), &y20211231);
        assert_eq!(mgr.is_trading(), false);

        // let mut datetime = y20240101.and_hms_opt(0, 30, 0).unwrap();
        // for idx in 1..=50 {
        //     datetime += chrono::Duration::hours(3);
        //     let (old_tday, curr_tday, old_date, curr_date, opt_err) =
        //         mgr.time_changed(&datetime, true)?;
        //     println!(
        //         "{:>2}: ({} {:>}) => {}, {}, {}, {}, trading? {}",
        //         idx,
        //         datetime,
        //         datetime.date().weekday(),
        //         old_tday,
        //         curr_tday,
        //         old_date,
        //         curr_date,
        //         mgr.is_trading()
        //     );
        // }

        // 仅能判断出2024-01-02没有凌晨盘，但无法确定2024-01-02是否节假日
        let tday = mgr.fail_safe_tradingday(&(y20240101 + chrono::Duration::days(1)));
        assert!(!tday.morning);

        let start = y20210120.and_hms_opt(18, 22, 0).expect("chrono");
        mgr.reset(Some(&start))?;
        assert_eq!(mgr.current_tday(), &y20210120);
        let start = NaiveDateTime::from_str("2021-01-20T20:22:00").expect("chrono");
        mgr.reset(Some(&start))?;
        assert_eq!(mgr.current_tday(), &y20210121);
        Ok(())
    }
}
