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

const WATCH_THRESHOLD: i32 = 3;
const KICKBACK_CEILING: i32 = 7;

fn inject_synthetic_tick(dir: i32) {
    let mouse_data = (dir * 120) as u32;
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
    unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32); }
}

fn forward_to_next_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { CallNextHookEx(HHOOK(std::ptr::null_mut()), code, wparam, lparam) }
}

// 1. Actualiza la racha de ticks consecutivos en una misma dirección. Si dir coincide con la última dirección vista, 
// suma uno a la racha; si no coincide, la racha arranca de nuevo en 1 para la nueva dirección. 
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

// 2. Callback de bajo nivel que Windows invoca en cada evento de mouse del sistema. Es unsafe porque el propio callback de Windows 
// entrega datos vía puntero crudo (lparam), que hay que interpretar manualmente como MSLLHOOKSTRUCT antes de poder leerlo con seguridad.
unsafe extern "system" fn mouse_wheel_hook_callback(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // 2.1. Si no es un evento de rueda, no interviene: pasa directo.
    if code < 0 || wparam.0 as u32 != WM_MOUSEWHEEL { return forward_to_next_hook(code, wparam, lparam); }

    // 2.2. Lee el evento crudo que entrega Windows.
    let data = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
    
    // 2.2b. Si este evento fue inyectado por nosotros mismos (vía SendInput), lo dejamos pasar sin re-procesarlo — si no, el hook 
    // se dispararía a sí mismo en bucle.
    const LLMHF_INJECTED: u32 = 0x1;
    if data.flags & LLMHF_INJECTED != 0 { return forward_to_next_hook(code, wparam, lparam); }

    // 2.2c. Extrae la dirección del giro a partir del delta.
    let delta = ((data.mouseData >> 16) & 0xFFFF) as i16;
    let scroll_direction: i32 = if delta > 0 { 1 } else { -1 };

    // 2.3. Actualiza la racha con este tick, sin excepciones para ninguna dirección.
    let streak = streak_manager(scroll_direction);

    // 2.4. Racha aún insuficiente: se bloquea, sea cual sea la dirección.
    if streak < WATCH_THRESHOLD {
        println!("-> BLOQUEADO, silencio inicial (scroll_direction={scroll_direction}, streak={streak}/{WATCH_THRESHOLD})");
        return LRESULT(1);
    }

    //2.5. Racha alcanzó UMBRAL_VIGILANCIA: arranca vigilancia + compensación simultánea.
    println!("-> VIGILANCIA (scroll_direction={scroll_direction}, streak={streak}, techo={KICKBACK_CEILING})");
    inject_synthetic_tick(scroll_direction);
    LRESULT(1)
}

// 3. Instala el hook de mouse a nivel de sistema y mantiene el programa corriendo hasta que se cierre.
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
