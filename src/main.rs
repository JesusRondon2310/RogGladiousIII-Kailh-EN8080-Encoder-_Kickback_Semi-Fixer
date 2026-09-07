//! main.rs
mod filter;

fn main() -> windows::core::Result<()> {
    println!("Iniciando Kickback_Fix...");
    // Llamar al orquestador del módulo filter.
    filter::run()
}