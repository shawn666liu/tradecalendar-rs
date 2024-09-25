import tradecalendarpy as calendar
import datetime as dt


def test_calendar():
    cal = calendar.load_buildin_calendar()

    # 无状态
    date = dt.date(2024, 10, 11)
    istday = cal.is_trading_day(date)
    print(f"{date} is tradingday? {istday}")
    next = cal.get_next_trading_day(date, 6)
    print(f"next tradingday for {date} is {next}, type {type(next)}")
    tdlist = cal.get_trading_days_list(date, date + dt.timedelta(days=20))
    print(f"tdlist length is {len(tdlist)}\n{tdlist}")
    detail = cal.get_date_detail(date)
    print(f"detail is {detail}")
    pass

    # 有状态
    now = dt.datetime.now()
    changeinfo = cal.time_changed(now, False)
    print(f"change info: {changeinfo}")


test_calendar()
