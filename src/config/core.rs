//!config/core.rs

use super::store;

pub fn init() {
    store::init();
}

pub fn filtro() -> bool {
    store::filtro()
}

pub fn silencio_inicial() -> i32 {
    store::silencio_inicial()
}

pub fn techo_kickback() -> i32 {
    store::techo_kickback()
}
