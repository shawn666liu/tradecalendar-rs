#### TradeCalendar
中国股票/期货交易日历管理  
`cargo run --example ex1`

### Python绑定
注意，不同的python版本需要生成单独的wheel包  
- 生成/更新stub  
`cargo run --bin stub_gen`  
- 进入子目录  
`cd tradecalendarpy`
- 切换到需要的虚拟环境  
`conda activate your-env-name`
- 安装maturin  
`conda install maturin`  
或者 `pip install maturin`  
参看 https://github.com/PyO3/maturin
- 编译该虚拟环境对应python版本的whl包,用以分发然后手动安装  
`maturin build --release`  
生成的包在target/wheels目录下面,带有python版本号  
- 或者,直接为当前虚拟环境安装whl包  
`maturin develop --release`
### C++绑定
- 编译通过
- 复制target/cxxbridge/{rust, tradecalendarpp}及之下的所有.h和.cc文件  
  包括cxx.h, ???.rs.h, ???.rs.cc  
- 下载cxx.cc文件,   
  https://raw.githubusercontent.com/dtolnay/cxx/refs/heads/master/src/cxx.cc
- 复制target/release下面的tradecalendarpp.{dll,lib}文件, linux下则为libtradecalendarpp.so
- todo: 封装文件


#### 生成新的交易日历文件
1. 每年年底, 国务院办公厅发布放假安排后, 手工编辑holidays.csv文件, 注意周六周日都移除
2. 在顶层目录下, 执行 `cargo run --example calendar-update -- -i "./holidays.csv"`, 将在output目录下,生成相应文件
3. 复制output/calendar_part.csv的内容, 到calendar.csv末尾, 注意, 如果边界上有重叠, 用新文件的日期数据覆盖旧的   
4. 重新编译和发布项目
   
#### 更新交易日历数据
1. 用sql文件更新相关数据库
2. 更新各程序的calendar.csv配置文件
3. 如果是使用库文件内置的日历数据,则必须更新库文件本身
    - 编译并发布rust包的新版本, ./calendar.csv会自动include到程序内, 用户端需要更新这个包
    - python和c++版本, 也需要重新编译发布
4. 建议使用1和2的模式,实际上get_calendar()函数会尝试读取数据库、csv文件和内置数据,然后使用最后日期最大的那个


### 从数据库加载交易日历
支持postgres, mysql, odbc, clickhouse, 连接字符串如下:  
1. `postgres://user:passwd@localhost:5432/dbname`
2. `mysql://user:passwd@localhost:3306/dbname`
3. `clickhouse://user:passwd@localhost:8123/dbname`
4. odbc: `Driver={PostgreSQL Unicode};Server=localhost;PORT=5432;UID=user;PWD=passwd;Database=dbname`

- query: 5 fields required, keep the order of fields,
- `select date,morning,trading,night,next from your_table where date>='yyyy-mm-dd' order by date`

### stub_gen链接错误时
是因为找不到libpython3.12.so.1.0之类的.so或者.dll文件，  
需要把LD_LIBARY_PATH指向你虚拟环境env所在的lib目录  
LD_LIBARY_PATH=???env/lib  cargo run --bin stub_gen