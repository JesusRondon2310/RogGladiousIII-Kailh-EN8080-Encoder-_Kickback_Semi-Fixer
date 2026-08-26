//! detection.rs

use std::sync::atomic::{AtomicI32, Ordering};
use windows::Win32::Foundation::{HINSTANCE, HMODULE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, HHOOK, MSG, MSLLHOOKSTRUCT, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, WH_MOUSE_LL, WM_MOUSEWHEEL,
};

static LAST_DIR: AtomicI32 = AtomicI32::new(0);
static STREAK_COUNT: AtomicI32 = AtomicI32::new(0);

const REQUIRED_CONFIRMATIONS: i32 = 4;

fn pass_through(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { CallNextHookEx(HHOOK(std::ptr::null_mut()), code, wparam, lparam) }
}

// 1. Actualiza la racha de ticks consecutivos en una misma dirección. Si dir coincide con la última dirección vista, 
// suma uno a la racha; si no coincide, la racha arranca de nuevo en 1 para la nueva dirección. 
fn update_streak(dir: i32) -> i32 {
    let last = LAST_DIR.load(Ordering::Relaxed);
    if dir == last {
        let count = STREAK_COUNT.load(Ordering::Relaxed) + 1;
        STREAK_COUNT.store(count, Ordering::Relaxed);
        count
    } else {
        LAST_DIR.store(dir, Ordering::Relaxed);
        STREAK_COUNT.store(1, Ordering::Relaxed);
        1
    }
}

// 2. Callback de bajo nivel que Windows invoca en cada evento de mouse del sistema. Es unsafe porque el propio callback de Windows 
// entrega datos vía puntero crudo (lparam), que hay que interpretar manualmente como MSLLHOOKSTRUCT antes de poder leerlo con seguridad.
unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // 2.1. Si no es un evento de rueda, no interviene: pasa directo.
    if code < 0 || wparam.0 as u32 != WM_MOUSEWHEEL { return pass_through(code, wparam, lparam); }

    // 2.2. Lee el evento crudo que entrega Windows y extrae la dirección del giro.
    let data = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
    let delta = ((data.mouseData >> 16) & 0xFFFF) as i16;
    let dir: i32 = if delta > 0 { 1 } else { -1 };
    println!("tick: delta={delta} dir={dir}");

    // 2.3. Actualiza la racha con este tick, sin excepciones para ninguna dirección.
    let count = update_streak(dir);

    // 2.4. Racha aún insuficiente: se bloquea, sea cual sea la dirección.
    if count < REQUIRED_CONFIRMATIONS {
        println!("-> BLOQUEADO (dir={dir}, streak={count}/{REQUIRED_CONFIRMATIONS})");
        return LRESULT(1);
    }

    // 2.5. La racha alcanzó el umbral: esta dirección queda validada, el tick pasa.
    if count == REQUIRED_CONFIRMATIONS { println!("-> CONFIRMADO dir={dir} (streak={count})"); }
    pass_through(code, wparam, lparam)
}

// 3. Instala el hook de mouse a nivel de sistema y mantiene el programa corriendo hasta que se cierre.
pub fn run() -> windows::core::Result<()> {
    unsafe {
        // 3.1. Obtiene el handle del propio ejecutable, requerido por SetWindowsHookExW.
        let hmodule: HMODULE = GetModuleHandleW(None)?;
        let hinstance: HINSTANCE = hmodule.into();

        // 3.2. Instala el hook de bajo nivel de mouse, apuntando a hook_proc.
        let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), hinstance, 0)?;
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
