use std::sync::atomic::{AtomicI32, Ordering};

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, HHOOK, MSLLHOOKSTRUCT, SetWindowsHookExW, UnhookWindowsHookEx,
};

use super::injector::enqueue_manager;
use crate::helpers::constants as consts;
use crate::trace;

static LAST_DIR: AtomicI32 = AtomicI32::new(0);
static STREAK_COUNT: AtomicI32 = AtomicI32::new(0);

enum Action {
    Pass,
    Block,
    BlockAndInject,
}

fn pass_through(code: i32, wparam: usize, lparam: isize) -> isize {
    unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
}

fn wheel_direction(mouse_data: u32) -> i32 {
    if (mouse_data >> 16) as i16 > 0 { return consts::WHEEL_UP; }
    consts::WHEEL_DOWN
}

fn update_streak(direction: i32) -> i32 {
    if direction != LAST_DIR.load(Ordering::SeqCst) {
        LAST_DIR.store(direction, Ordering::SeqCst);
        STREAK_COUNT.store(1, Ordering::SeqCst);
        return 1;
    }
    let n = STREAK_COUNT.load(Ordering::SeqCst);
    if n > consts::TRUST_TICKS { return n; }
    STREAK_COUNT.fetch_add(1, Ordering::SeqCst) + 1
}

fn decide(streak: i32) -> Action {
    if streak <= consts::SILENCE_TICKS { return Action::Block; }
    
    if streak <= consts::TRUST_TICKS { return Action::BlockAndInject; }
    Action::Pass
}

// 1. Windows lo llama en cada evento de mouse, en el hilo que instaló el hook. lParam apunta a un MSLLHOOKSTRUCT válido solo durante la llamada.
unsafe extern "system" fn mouse_wheel_catcher_hook(code: i32, wparam: usize, lparam: isize) -> isize {
    // 1.1. Solo eventos de rueda; el resto pasa directo.
    if code < 0 || wparam != consts::WHEEL_EVENT { return pass_through(code, wparam, lparam); }
    let event = unsafe { &*(lparam as *const MSLLHOOKSTRUCT) };

    // 1.2. Un evento inyectado por nosotros pasa sin re-procesar, o el hook se dispara a sí mismo en bucle.
    if event.flags & consts::SELF_INJECTED != 0 { return pass_through(code, wparam, lparam); }

    // 1.3. Dirección de este tick + racha acumulada.
    let direction = wheel_direction(event.mouseData);
    let streak = update_streak(direction);

    // 1.4. Decidir según la racha
    match decide(streak) {
        Action::Pass => pass_through(code, wparam, lparam),
        Action::BlockAndInject => {
            trace!("[COMPENSA] dir={} racha={}\n", direction, streak);
            enqueue_manager(direction);
            consts::BLOCK
        }
        Action::Block => {
            trace!("[SILENCIO] dir={} racha={}\n", direction, streak);
            consts::BLOCK
        }
    }
}

// 2. Arranca la detección apuntando a mouse_wheel_catcher_hook. Devuelve el handle para pararlo.
pub(super) fn start_hook() -> std::io::Result<HHOOK> {
    let hmod = unsafe { GetModuleHandleW(std::ptr::null()) };
    let hook = unsafe { SetWindowsHookExW(consts::MOUSE_HOOK, Some(mouse_wheel_catcher_hook), hmod, 0) };
    if hook.is_null() { return Err(std::io::Error::last_os_error()); }
    Ok(hook)
}

// 3. Para la detección.
pub(super) fn stop_hook(hook: HHOOK) -> std::io::Result<()> {
    if unsafe { UnhookWindowsHookEx(hook) } == 0 { return Err(std::io::Error::last_os_error()); }
    Ok(())
}
