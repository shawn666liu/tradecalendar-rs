// https://crates.io/crates/pyo3-stub-gen

use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    // `stub_info` is a function defined by `define_stub_info_gatherer!` macro.
    let stub = tradecalendarpy::stub_info()?;
    stub.generate()?;
    Ok(())
}
