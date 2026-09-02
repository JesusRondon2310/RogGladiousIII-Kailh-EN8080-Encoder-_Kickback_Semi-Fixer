//! main.rs

mod filter;
mod helpers;

fn main() -> windows::core::Result<()> {
    filter::run()
}
