//! detection/physical.rs

use std::sync::atomic::{AtomicI32, Ordering};
use windows::Win32::Foundation::{HINSTANCE, HMODULE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, HHOOK, MSLLHOOKSTRUCT, SetWindowsHookExW, UnhookWindowsHookEx, WH_MOUSE_LL,
    WM_MOUSEWHEEL,
};

use super::synthetic::Enqueue;

static LAST_DIR: AtomicI32 = AtomicI32::new(0);
static STREAK_COUNT: AtomicI32 = AtomicI32::new(0);

const WATCH_THRESHOLD: i32 = 3;
const LLMHF_INJECTED: u32 = 0x1;

fn pass_through(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { CallNextHookEx(HHOOK(std::ptr::null_mut()), code, wparam, lparam) }
}

// 1. Actualiza la racha de ticks consecutivos en una misma dirección. Si dir coincide con la última dirección vista, suma uno;
// si no, la racha arranca de nuevo en 1 y se reinicia el contador de inyecciones (en synthetic).
fn update_streak(dir: i32) -> i32 {
    let last = LAST_DIR.load(Ordering::Relaxed);
    if dir == last {
        let count = STREAK_COUNT.load(Ordering::Relaxed) + 1;
        STREAK_COUNT.store(count, Ordering::Relaxed);
        return count;
    }
    LAST_DIR.store(dir, Ordering::Relaxed);
    STREAK_COUNT.store(1, Ordering::Relaxed);
    super::synthetic::reset();
    1
}

// 2. Callback de bajo nivel que Windows invoca en cada evento de mouse del sistema. Es unsafe porque Windows entrega los datos
// vía puntero crudo (lparam), que hay que reinterpretar como MSLLHOOKSTRUCT.
unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // 2.1. Guarda: solo intervienen los eventos de rueda; el resto pasa directo.
    if code < 0 || wparam.0 as u32 != WM_MOUSEWHEEL { return pass_through(code, wparam, lparam); }

    let data = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };

    // 2.2. Guarda: un evento que inyectamos nosotros pasa sin re-procesarse, o el hook se dispararía a sí mismo en bucle.
    if data.flags & LLMHF_INJECTED != 0 { return pass_through(code, wparam, lparam); }

    // 2.3. Dirección de este tick (tarea 2) y racha acumulada (tarea 1).
    let delta = ((data.mouseData >> 16) & 0xFFFF) as i16;
    let dir: i32 = if delta > 0 { 1 } else { -1 };
    let streak = update_streak(dir);

    // 2.4. Silencio inicial (tarea 3): racha por debajo del umbral -> se bloquea el tick, sin inyectar todavía.
    if streak < WATCH_THRESHOLD {
        println!("[BLOQUEADO] dir={dir} streak={streak}/{WATCH_THRESHOLD}");
        return LRESULT(1);
    }

    // 2.5. Umbral alcanzado (tarea 4): se bloquea el tick físico y se le pide a synthetic que encole un sintético.
    match super::synthetic::enqueue(dir) {
        Enqueue::Encolada(ok) => println!("[VIGILANCIA] dir={dir} streak={streak} inyeccion_encolada={ok}"),
        Enqueue::Tope => {
            println!("[VIGILANCIA] dir={dir} streak={streak} inyeccion_encolada=false");
            super::core::request_quit();
        }
    }
    LRESULT(1)
}

// 3. Handle del propio ejecutable + instala el hook de bajo nivel apuntando a hook_proc. Devuelve el handle para desinstalarlo
// después.
pub(super) fn install() -> windows::core::Result<HHOOK> {
    unsafe {
        let hmodule: HMODULE = GetModuleHandleW(None)?;
        let hinstance: HINSTANCE = hmodule.into();
        SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), hinstance, 0)
    }
}

// 4. Al salir, desinstala el hook limpiamente.
pub(super) fn uninstall(hook: HHOOK) -> windows::core::Result<()> { unsafe { UnhookWindowsHookEx(hook) } }
