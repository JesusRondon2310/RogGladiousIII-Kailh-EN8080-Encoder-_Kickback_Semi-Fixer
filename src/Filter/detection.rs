//! detection.rs
//! Hook de bajo nivel de Windows (WH_MOUSE_LL).

use std::sync::atomic::{AtomicI32, Ordering};
use windows::Win32::Foundation::{HINSTANCE, HMODULE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, HHOOK, MSLLHOOKSTRUCT, SetWindowsHookExW, WH_MOUSE_LL, WM_MOUSEWHEEL,
};

use crate::filter::{
    constants::*,
    streak::streak_manager,
    machine::{Phase, Action, decide_action},
    injector::enqueue_manager,
};

static PHASE: AtomicI32 = AtomicI32::new(Phase::Silence as i32);
static COMPENSATION_COUNT: AtomicI32 = AtomicI32::new(0);
static BLOCK_ATTEMPT_COUNT: AtomicI32 = AtomicI32::new(0);

unsafe extern "system" fn mouse_wheel_hook_callback(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // 1. Filtro de eventos no deseados
    if code < 0 || wparam.0 as u32 != WM_MOUSEWHEEL {
        return forward_to_next_hook(code, wparam, lparam);
    }

    // 2. Leer el evento crudo
    let data = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
    
    // 3. Si el evento fue inyectado por nosotros, lo dejamos pasar
    const LLMHF_INJECTED: u32 = 0x1;
    if data.flags & LLMHF_INJECTED != 0 {
        println!("[hook] Evento inyectado, pasando...");
        return forward_to_next_hook(code, wparam, lparam);
    }

    // 4. Extraer dirección y actualizar racha
    let delta = ((data.mouseData >> 16) & 0xFFFF) as i16;
    let direction: i32 = if delta > 0 { 1 } else { -1 };
    let streak = streak_manager(direction);
    println!("[tick] delta={} dir={} streak={}", delta, direction, streak);

    // 5. Obtener estado actual
    let phase_int = PHASE.load(Ordering::Relaxed);
    let phase = match phase_int {
        0 => Phase::Silence,
        1 => Phase::Vigilance,
        2 => Phase::BlockNewAttempt,
        3 => Phase::Compensating,
        4 => Phase::Normal,
        _ => Phase::Silence,
    };
    let comp_count = COMPENSATION_COUNT.load(Ordering::Relaxed);
    let block_attempt = BLOCK_ATTEMPT_COUNT.load(Ordering::Relaxed);

    println!("[hook] fase actual: {:?}, comp={}, block_attempt={}", phase, comp_count, block_attempt);

    // 6. Decidir acción
    let (new_phase, new_block_attempt, action) = decide_action(
        phase,
        streak,
        comp_count,
        block_attempt,
    );

    // 7. Actualizar estado
    PHASE.store(new_phase as i32, Ordering::Relaxed);
    BLOCK_ATTEMPT_COUNT.store(new_block_attempt, Ordering::Relaxed);

    // 8. Ejecutar acción
    match action {
        Action::Pass => {
            println!("[hook] -> PASAR (normal)");
        }
        Action::Block => {
            println!("[hook] -> BLOQUEAR sin inyectar");
        }
        Action::BlockAndCompensate => {
            println!("[hook] -> BLOQUEAR E INYECTAR sintético");
            enqueue_manager(direction);
            if new_phase == Phase::Compensating || phase == Phase::Compensating {
                let new_count = COMPENSATION_COUNT.load(Ordering::Relaxed) + 1;
                COMPENSATION_COUNT.store(new_count, Ordering::Relaxed);
                println!("[hook] contador compensación: {}", new_count);
            } else {
                COMPENSATION_COUNT.store(0, Ordering::Relaxed);
            }
        }
    }

    // 9. Retornar según acción
    match action {
        Action::Pass => forward_to_next_hook(code, wparam, lparam),
        Action::Block | Action::BlockAndCompensate => LRESULT(1),
    }
}

fn forward_to_next_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { CallNextHookEx(HHOOK(std::ptr::null_mut()), code, wparam, lparam) }
}

pub fn install_hook() -> windows::core::Result<HHOOK> {
    unsafe {
        let hmodule: HMODULE = GetModuleHandleW(None)?;
        let hinstance: HINSTANCE = hmodule.into();
        let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_wheel_hook_callback), hinstance, 0)?;
        Ok(hook)
    }
}