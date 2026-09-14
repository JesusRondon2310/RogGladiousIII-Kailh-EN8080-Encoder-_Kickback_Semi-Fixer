//!main.rs

mod config;
mod filter;
mod helpers;

fn main() {
    config::init();

    if !config::filtro() {
        println!("Filtro desactivado en config.toml.");
        return;
    }

    if filter::run().is_err() {
        std::process::exit(1);
    }
}
