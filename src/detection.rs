//! detection.rs

use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use windows::Win32::Foundation::{HINSTANCE, HMODULE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, HHOOK, MSG, MSLLHOOKSTRUCT, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, WH_MOUSE_LL, WM_MOUSEWHEEL,
};

static CONFIRMED_DIR: AtomicI32 = AtomicI32::new(0);
static PENDING_DIR: AtomicI32 = AtomicI32::new(0);
static PENDING_COUNT: AtomicI32 = AtomicI32::new(0);
static LAST_TICK_TIME_MS: AtomicU64 = AtomicU64::new(0);

const REQUIRED_CONFIRMATIONS: i32 = 3;

fn now_ms() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 }

fn pass_through(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { CallNextHookEx(HHOOK(std::ptr::null_mut()), code, wparam, lparam) }
}

fn log_tick(delta: i16, dir: i32) {
    let now = now_ms();
    let last = LAST_TICK_TIME_MS.swap(now, Ordering::Relaxed);
    let elapsed = if last == 0 { 0 } else { now.saturating_sub(last) };
    println!("tick: delta={delta} dir={dir} elapsed_desde_ultimo_tick={elapsed}ms");
}

// 1. Fija dir como la dirección confirmada y descarta el candidato que se estuviera observando, si había uno.
fn confirm_direction(dir: i32) {
    CONFIRMED_DIR.store(dir, Ordering::Relaxed);
    PENDING_DIR.store(0, Ordering::Relaxed);
    PENDING_COUNT.store(0, Ordering::Relaxed);
}

// 2. Registra dir como el nuevo candidato a vigilar y arranca su conteo de confirmaciones en cero.
fn start_candidate(dir: i32) {
    PENDING_DIR.store(dir, Ordering::Relaxed);
    PENDING_COUNT.store(0, Ordering::Relaxed);
}

// 3. Callback de bajo nivel que Windows invoca en cada evento de mouse del sistema. Es unsafe
//    porque el propio callback de Windows entrega datos vía puntero crudo (lparam), que hay
//    que interpretar manualmente como MSLLHOOKSTRUCT antes de poder leerlo con seguridad.
unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // 3.1. Si no es un evento de rueda, no interviene: pasa directo.
    if code < 0 || wparam.0 as u32 != WM_MOUSEWHEEL { return pass_through(code, wparam, lparam); }

    // 3.2. Lee el evento crudo que entrega Windows y extrae la dirección del giro.
    let data = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
    let delta = ((data.mouseData >> 16) & 0xFFFF) as i16;
    let dir: i32 = if delta > 0 { 1 } else { -1 };
    log_tick(delta, dir);

    // 3.3. Lee el estado actual del filtro.
    let confirmed = CONFIRMED_DIR.load(Ordering::Relaxed);
    let pending_dir = PENDING_DIR.load(Ordering::Relaxed);

    // 3.4. Primer tick de siempre, o coincide con la dirección ya confirmada: pasa directo.
    if confirmed == 0 || dir == confirmed {
        confirm_direction(dir);
        return pass_through(code, wparam, lparam);
    }

    // 3.5. No coincide con el candidato en observación: arranca uno nuevo, se bloquea.
    if dir != pending_dir {
        start_candidate(dir);
        println!("-> BLOQUEADO (nuevo candidato: dir={dir}, pending_dir={pending_dir})");
        return LRESULT(1);
    }

    // 3.6. Coincide con el candidato en observación: suma una confirmación.
    let count = PENDING_COUNT.load(Ordering::Relaxed) + 1;

    // 3.7. Aún no hay suficiente evidencia de que sea un cambio real: se sigue bloqueando.
    if count < REQUIRED_CONFIRMATIONS {
        PENDING_COUNT.store(count, Ordering::Relaxed);
        println!("-> BLOQUEADO, esperando más confirmaciones (dir={dir}, count={count}/{REQUIRED_CONFIRMATIONS})");
        return LRESULT(1);
    }

    // 3.8. Se alcanzó el número de confirmaciones requerido: se acepta como cambio real.
    println!("-> CONFIRMADO cambio a dir={dir} (tras {count} ticks)");
    confirm_direction(dir);
    pass_through(code, wparam, lparam)
}

// 4. Instala el hook de mouse a nivel de sistema y mantiene el programa corriendo hasta que se cierre.
pub fn run() -> windows::core::Result<()> {
    unsafe {
        // 4.1. Obtiene el handle del propio ejecutable, requerido por SetWindowsHookExW.
        let hmodule: HMODULE = GetModuleHandleW(None)?;
        let hinstance: HINSTANCE = hmodule.into();

        // 4.2. Instala el hook de bajo nivel de mouse, apuntando a hook_proc.
        let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), hinstance, 0)?;
        println!("Filtro activo. Deja esta ventana abierta.");

        // 4.3. Mantiene el programa vivo escuchando eventos; sin esto, el hook dejaría de recibir nada.
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // 4.4. Al salir del loop, desinstala el hook limpiamente.
        UnhookWindowsHookEx(hook)?;
    }

    Ok(())
}
