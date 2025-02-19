#### TradeCalendar
中国股票/期货交易日历管理  
#cargo run --example ex1

### Python绑定
- 进入子目录  
cd tradecalendarpy
- 切换到需要的虚拟环境  
conda activate your-env-name
- 安装maturin  
conda install maturin  
或者 pip install maturin  
参看 https://github.com/PyO3/maturin
- 编译该虚拟环境对应python版本的whl包,用以分发然后手动安装  
maturin build --release
- 或者,直接为当前虚拟环境安装whl包  
maturin develop --release
### C++绑定
- 编译通过
- 复制target/cxxbridge/{rust, tradecalendarpp}及之下的所有.h和.cc文件  
  包括cxx.h, ???.rs.h, ???.rs.cc  
- 下载cxx.cc文件,   
  https://raw.githubusercontent.com/dtolnay/cxx/refs/heads/master/src/cxx.cc
- 复制target/release下面的tradecalendarpp.{dll,lib}文件, linux下则为libtradecalendarpp.so
- todo: 封装文件