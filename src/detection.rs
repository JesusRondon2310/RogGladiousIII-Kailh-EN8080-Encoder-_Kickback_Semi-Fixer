//! detection.rs

use std::sync::atomic::{AtomicI32, Ordering};
use windows::Win32::Foundation::{HINSTANCE, HMODULE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, HHOOK, MSG, MSLLHOOKSTRUCT, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, WH_MOUSE_LL, WM_MOUSEWHEEL,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_WHEEL, MOUSEINPUT,
};

static LAST_SEEN_DIRECTION: AtomicI32 = AtomicI32::new(0);
static DIRECTION_STREAK_COUNT: AtomicI32 = AtomicI32::new(0);
static PHASE: AtomicI32 = AtomicI32::new(0);
static COMPENSATION_COUNT: AtomicI32 = AtomicI32::new(0);


const WATCH_THRESHOLD: i32 = 3;
const KICKBACK_CEILING: i32 = 7;
const COMPENSATION_TARGET: i32 = 7;


fn inject_synthetic_tick(dir: i32) {
    let mouse_data = (dir * 120) as u32; // 120 = unidad estándar de Windows para un "click" de rueda
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: mouse_data,
                dwFlags: MOUSEEVENTF_WHEEL,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

fn forward_to_next_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { CallNextHookEx(HHOOK(std::ptr::null_mut()), code, wparam, lparam) }
}

/// 1. Actualiza la racha de ticks consecutivos en una misma dirección.
/// Si `scroll_direction` coincide con la última dirección vista, suma uno a la racha;
/// si no coincide, la racha arranca de nuevo en 1 para la nueva dirección.
fn streak_manager(scroll_direction: i32) -> i32 {
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


/// 2. Callback de bajo nivel que Windows invoca en cada evento de mouse del sistema.
/// Es `unsafe` porque el propio callback de Windows entrega datos vía puntero crudo (`lparam`),
/// que hay que interpretar manualmente como `MSLLHOOKSTRUCT` antes de poder leerlo con seguridad.
unsafe extern "system" fn mouse_wheel_hook_callback(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // 2.1. Si no es un evento de rueda, no interviene: pasa directo.
    if code < 0 || wparam.0 as u32 != WM_MOUSEWHEEL {
        return forward_to_next_hook(code, wparam, lparam);
    }

    // 2.2. Lee el evento crudo que entrega Windows y extrae la dirección del giro.
    let data = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
    
    // 2.2b. Si este evento fue inyectado por nosotros mismos (vía SendInput), lo dejamos pasar sin re-procesarlo — si no, el hook 
    // se dispararía a sí mismo en bucle.
    const LLMHF_INJECTED: u32 = 0x1;
    if data.flags & LLMHF_INJECTED != 0 {
        return forward_to_next_hook(code, wparam, lparam);
    }

    // 2.2c. Extrae la dirección del giro a partir del delta.
    let delta = ((data.mouseData >> 16) & 0xFFFF) as i16;
    let scroll_direction: i32 = if delta > 0 { 1 } else { -1 };

    // 2.3. Actualiza la racha con este tick, sin excepciones para ninguna dirección (Partes 1 y 2).
    let streak = streak_manager(scroll_direction);

    // ========================================================================
    // Manejo de fases 
    // ========================================================================
    let phase = PHASE.load(Ordering::Relaxed);

    // --- Flujo normal ---
    if phase == 3 {
        println!("-> FLUJO NORMAL (Paso 8)");
        return forward_to_next_hook(code, wparam, lparam);
    }

    // --- Compensando  ---
    if phase == 2 {
        inject_synthetic_tick(scroll_direction);
        let comp_count = COMPENSATION_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        println!("-> COMPENSANDO (Paso 7): inyectado {}/{} sintéticos", comp_count, COMPENSATION_TARGET);
        if comp_count >= COMPENSATION_TARGET {
            PHASE.store(3, Ordering::Relaxed);
            println!("-> OBJETIVO ALCANZADO, FLUJO NORMAL (Paso 8)");
        }
        return LRESULT(1);   // Bloqueamos el real
    }

    // --- Corte antes del techo ---
    // Si estamos en Vigilancia (fase 1) y la racha se reinició a 1 (cambio de dirección)
    if streak == 1 && phase == 1 {
        PHASE.store(0, Ordering::Relaxed);
        COMPENSATION_COUNT.store(0, Ordering::Relaxed);
        println!("-> CORTE ANTES DEL TECHO (Paso 5): reinicio total");
        return LRESULT(1);   // Bloqueamos sin inyectar
    }

    // --- Transición de fase 0 a 1 (primera vez que se supera el umbral) ---
    if phase == 0 && streak >= WATCH_THRESHOLD {
        PHASE.store(1, Ordering::Relaxed);
        COMPENSATION_COUNT.store(0, Ordering::Relaxed);
        println!("-> INICIO DE VIGILANCIA (Paso 4)");
    }

    // 2.4. Racha aún insuficiente: se bloquea sin compensación (Paso 3).
    if streak < WATCH_THRESHOLD {
        println!("-> BLOQUEADO, silencio inicial (scroll_direction={scroll_direction}, streak={streak}/{WATCH_THRESHOLD})");
        return LRESULT(1);
    }

    // 2.5. Racha alcanzó UMBRAL_VIGILANCIA: arranca vigilancia + compensación simultánea (Paso 4).
    println!("-> VIGILANCIA (scroll_direction={scroll_direction}, streak={streak}, techo={KICKBACK_CEILING})");
    inject_synthetic_tick(scroll_direction);
    

    let comp_count = COMPENSATION_COUNT.fetch_add(1, Ordering::Relaxed) + 1;

    //  Confirmación
    if streak >= KICKBACK_CEILING && PHASE.load(Ordering::Relaxed) == 1 {
        PHASE.store(2, Ordering::Relaxed);
        println!("-> CONFIRMADO (Paso 6): dirección oficial {}, streak={}", scroll_direction, streak);
    }

    // si ya completamos el objetivo durante la vigilancia, pasamos a Normal
    if comp_count >= COMPENSATION_TARGET {
        PHASE.store(3, Ordering::Relaxed);
        println!("-> OBJETIVO DE COMPENSACIÓN ALCANZADO (Paso 7), FLUJO NORMAL (Paso 8)");
    }

    // Bloqueamos el tick real (no llega a la aplicación)
    LRESULT(1)
}

// ============================================================================
//  PUNTO DE ENTRADA DEL MÓDULO
// ============================================================================

/// 3. Instala el hook de mouse a nivel de sistema y mantiene el programa corriendo hasta que se cierre.
pub fn run() -> windows::core::Result<()> {
    unsafe {
        // 3.1. Obtiene el handle del propio ejecutable, requerido por SetWindowsHookExW.
        let hmodule: HMODULE = GetModuleHandleW(None)?;
        let hinstance: HINSTANCE = hmodule.into();

        // 3.2. Instala el hook de bajo nivel de mouse, apuntando a mouse_wheel_hook_callback.
        let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_wheel_hook_callback), hinstance, 0)?;
        println!("Filtro activo. Deja esta ventana abierta.");

        // 3.3. Mantiene el programa vivo escuchando eventos; sin esto, el hook dejaría de recibir nada.
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // 3.4. Al salir del loop, desinstala el hook limpiamente.
        UnhookWindowsHookEx(hook)?;
    }

    Ok(())
}