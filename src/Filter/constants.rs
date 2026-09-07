//! constants.rs
///Número de ticks consecutivos para activar la vigilancia.
pub const WATCH_THRESHOLD: i32 = 3;
/// Techo mínimo de racha para considerar la dirección como "confirmada".
pub const KICKBACK_CEILING: i32 = 7;
/// Objetivo de compensación: número total de ticks (reales + sintéticos) que deben acumularse para dar por finalizada la compensación.
/// Depende de KICKBACK_CEILING para evitar saltos de fase.
pub const COMPENSATION_TARGET: i32 = KICKBACK_CEILING; 
/// Número de ticks "limpios" que deben pasar tras un corte antes de reactivar la vigilancia.
pub const BLOCK_NEW_ATTEMPT: i32 = 2;