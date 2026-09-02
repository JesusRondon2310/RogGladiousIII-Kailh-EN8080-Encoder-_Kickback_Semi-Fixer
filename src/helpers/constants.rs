//! helpers/constants.rs

use windows::Win32::Foundation::{BOOL, LRESULT};

// ---- Dirección de la rueda ----
pub const WHEEL_UP: i32 = 1;
pub const WHEEL_DOWN: i32 = -1;

// ---- Rueda / hook de bajo nivel (Win32) ----
pub const WHEEL_TICK_UNIT: i32 = 120;    // WHEEL_DELTA — un notch de rueda
pub const LLMHF_INJECTED: u32 = 0x1;     // flag "inyectado por nosotros" en MSLLHOOKSTRUCT

// ---- Retornos de callbacks Win32 ----
pub const BLOCK: LRESULT = LRESULT(1);   // callback del hook: no pasar el evento al siguiente
pub const HANDLED: BOOL = BOOL(1);       // console ctrl handler: lo manejamos nosotros
pub const NOT_HANDLED: BOOL = BOOL(0);   // console ctrl handler: que siga el default

// ---- SendInput ----
pub const ONE_EVENT: u32 = 1;            // esperamos que SendInput inserte exactamente 1

// ---- Config del filtro (tuning; candidatas a runtime más adelante) ----
pub const WATCH_THRESHOLD: i32 = 3;      // racha para arrancar vigilancia + compensación
pub const DIAG_INJECTION_LIMIT: i32 = 3; // tope de inyecciones por gesto (diagnóstico)
