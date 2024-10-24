import tradecalendarpy as calendar
import datetime as dt


def test_calendar():
    dburl = "postgresql://readonly:readonly@192.168.9.122:5432/future_info"
    proto = None
    # 旧版的clickhouse server支持mysql text protocol
    # dburl = "mysql://readonly:readonly@192.168.100.208:9005/futuredb"
    # proto = "text" # access clickhouse via mysql `text` protocol
    sql = "select date,morning,trading,night,next from calendar where date>'2020-01-01' order by date"

    # 新版直接使用clickhouse接口读取,不用connectorx
    dburl = "clickhouse://readonly:readonly@192.168.9.122:8123/futuredb"
    sql = "SELECT ?fields FROM futuredb.calendar WHERE date>='2024-01-01' ORDER BY date"
    cal = calendar.get_calendar(
        dburl, sql, proto, csv_file=None, start_date=dt.date(2024, 1, 1)
    )
    # cal = calendar.get_buildin_calendar()
    print(f"internal list is from {cal.min_date()} to {cal.max_date()}")

    # 无状态
    date = dt.date(2024, 10, 11)
    istday = cal.is_trading_day(date)
    print(f"{date} is tradingday? {istday}")
    num = 6
    next = cal.get_next_trading_day(date, num)
    print(f"next {num} tradingday for {date} is {next}, type {type(next)}")
    tdlist = cal.get_trading_days_list(date, date + dt.timedelta(days=20))
    print(f"tdlist length is {len(tdlist)}\n{tdlist}")
    detail = cal.get_date_detail(date)
    print(f"detail is {detail}")
    pass

    # 有状态
    now = dt.datetime.now()
    changeinfo = cal.time_changed(now, False)
    print(
        "change info:  (previous_tradingday, current_tradingday, previous_date, current_date, option<error_msg>)"
    )
    print(f"change info1: {changeinfo}")
    changeinfo = cal.time_changed(now, False)
    print(f"change info2: {changeinfo}")


test_calendar()
