//! The `mdeezl` command line: everything lives in the library (`src/lib.rs`).
fn main() {
    std::process::exit(mdeezl::cli(std::env::args().skip(1)))
}
