//! main.rs

mod detection;

fn main() -> windows::core::Result<()> {
    detection::run()
}
