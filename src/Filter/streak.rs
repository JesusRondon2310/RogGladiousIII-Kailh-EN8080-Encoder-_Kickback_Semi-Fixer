//! streak.rs


use std::sync::atomic::{AtomicI32, Ordering};

/// Última dirección de giro vista (1 = arriba, -1 = abajo).
static LAST_SEEN_DIRECTION: AtomicI32 = AtomicI32::new(0);

/// Conteo de ticks consecutivos en la misma dirección (racha actual).
static DIRECTION_STREAK_COUNT: AtomicI32 = AtomicI32::new(0);

/// 1. Actualiza la racha de ticks consecutivos en una misma dirección.
/// Si `scroll_direction` coincide con la última dirección vista, suma uno a la racha;
/// si no coincide, la racha arranca de nuevo en 1 para la nueva dirección.
pub fn streak_manager(scroll_direction: i32) -> i32 {
    let last = LAST_SEEN_DIRECTION.load(Ordering::Relaxed);
    if scroll_direction == last {
        let count = DIRECTION_STREAK_COUNT.load(Ordering::Relaxed) + 1;
        DIRECTION_STREAK_COUNT.store(count, Ordering::Relaxed);
        count
    } else {
        LAST_SEEN_DIRECTION.store(scroll_direction, Ordering::Relaxed);
        DIRECTION_STREAK_COUNT.store(1, Ordering::Relaxed);
        1
    }
}

/// Reinicia el estado de la racha (usado en cortes o reinicios).
pub fn reset_streak() {
    LAST_SEEN_DIRECTION.store(0, Ordering::Relaxed);
    DIRECTION_STREAK_COUNT.store(0, Ordering::Relaxed);
}