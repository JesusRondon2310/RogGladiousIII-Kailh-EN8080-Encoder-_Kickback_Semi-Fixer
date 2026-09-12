use std::sync::atomic::{AtomicU32, Ordering};

use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PostThreadMessageW, TranslateMessage,
};

use crate::helpers::constants as consts;

static MAIN_THREAD_ID: AtomicU32 = AtomicU32::new(0);

// 1. Engancha el handler de cierre (Ctrl+C / cierre de ventana) en este hilo.
pub(super) fn install_quit_handler() -> std::io::Result<()> {
    MAIN_THREAD_ID.store(unsafe { GetCurrentThreadId() }, Ordering::SeqCst);
    if unsafe { SetConsoleCtrlHandler(Some(console_ctrl_handler), 1) } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

// 2. Bombeo de mensajes: sin esto el hook deja de recibir eventos. GetMessageW devuelve 0 en QUIT_MESSAGE, -1 en error.
pub(super) fn pump_messages() {
    let mut msg: MSG = unsafe { std::mem::zeroed() };
    loop {
        let ret = unsafe { GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) };
        if ret <= 0 {
            break;
        }
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

// 3. Postea el mensaje de salida al hilo del bombeo. Lo llama console_ctrl_handler en Ctrl+C / cierre.
fn request_quit() {
    unsafe {
        PostThreadMessageW(MAIN_THREAD_ID.load(Ordering::SeqCst), consts::QUIT_MESSAGE, 0, 0);
    }
}

// 4. Windows lo llama en Ctrl+C / cierre de ventana.
unsafe extern "system" fn console_ctrl_handler(ctrl_type: u32) -> i32 {
    match ctrl_type {
        consts::CTRL_C_EVENT | consts::CTRL_BREAK_EVENT | consts::CTRL_CLOSE_EVENT => {
            request_quit();
            consts::HANDLED
        }
        _ => consts::NOT_HANDLED,
    }
}
