fn main() {
    cxx_build::bridge("src/tradecalendarpp.rs")
        .std("c++14")
        .compile("tradecalendarpp");

    println!("cargo:rerun-if-changed=src/tradecalendarpp.rs");
}
