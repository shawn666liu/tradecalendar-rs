import typing
import datetime as dt
from typing import List

class TradeCalendar:
    def reload(
        self, db_conn, query, proto=..., csv_file=..., start_date=...
    ) -> None: ...
    def is_trading_day(self, date) -> bool: ...
    def get_next_trading_day(self, date, num) -> dt.date: ...
    def get_prev_trading_day(self, date, num) -> dt.date: ...
    def get_trading_days_count(self, start_dt, end_dt) -> int: ...
    def get_trading_days_list(self, start_dt, end_dt) -> List[dt.date]: ...
    def get_date_detail(self, theday) -> dict: ...
    def trading_day_from_datetime(
        self,
        input: dt.datetime,
        for_next: bool,
        is_finance_item: bool,
    ) -> dt.date: ...
    def max_date(self) -> dt.date: ...
    def min_date(self) -> dt.date: ...
    def reset(self, start_time=...) -> None: ...
    def is_trading(self) -> bool: ...
    def time_changed(
        self,
        datetime: dt.datetime,
        fail_safe: bool,
    ): ...
    def set_config(
        self,
        tday_shift: dt.datetime,
        night_begin: dt.datetime,
        night_end: dt.datetime,
        day_begin: dt.datetime,
        day_end: dt.datetime,
    ):
        """重置日期边界的一些配置,
        调用此函数之后，可以调用time_changed()刷新状态
        tday_shift: 交易日切换的时间点，缺省值 19:30:00, 影响trading_day()/prev_tday()/next_tday()
        以下4个配置影响 is_trading()
        night_begin: 缺省值 20:30:00
        night_end: 缺省值 2:31:00
        day_begin: 缺省值 8:30:00
        day_end: 缺省值 15:30:00
        """
        ...

    def get_config(self): ...
    def prev_tday(self): ...
    def current_tday(self): ...
    def next_tday(self): ...

def get_buildin_calendar(start_date=...) -> TradeCalendar:
    r"""
    get_buildin_calendar
    """
    ...

def get_csv_calendar(csv_file, start_date=...) -> TradeCalendar:
    r"""
    get_csv_calendar
    """
    ...

def get_calendar(
    db_conn, query, proto=..., csv_file=..., start_date=...
) -> TradeCalendar:
    r"""
    get_calendar
    """
    ...
