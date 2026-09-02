//! filter/core.rs

use std::sync::atomic::{AtomicU32, Ordering};
use windows::Win32::Foundation::{BOOL, LPARAM, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::System::Console::{
    CTRL_BREAK_EVENT, CTRL_C_EVENT, CTRL_CLOSE_EVENT, SetConsoleCtrlHandler,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PostThreadMessageW, TranslateMessage, WM_QUIT,
};

use crate::helpers::constants::{HANDLED, NOT_HANDLED};

// Hilo que corre el bombeo de mensajes; destino del WM_QUIT de salida.
static MAIN_THREAD_ID: AtomicU32 = AtomicU32::new(0);

// 1. Punto de entrada del módulo. Arranca la detección y el injector, corre el bombeo de mensajes hasta un cierre, y limpia.
pub fn run() -> windows::core::Result<()> {
    // 1.1. Registra este hilo como destino del WM_QUIT y engancha el handler de cierre de consola.
    MAIN_THREAD_ID.store(unsafe { GetCurrentThreadId() }, Ordering::Relaxed);
    unsafe { SetConsoleCtrlHandler(Some(console_ctrl_handler), true) }?;

    // 1.2. Arranca el hilo inyector antes de la detección.
    super::injector::start();

    // 1.3. Arranca la detección de ticks (lo gestiona `detection`).
    let hook = super::detection::start()?;
    println!("Filtro activo (v2 hasta tarea 4). Ctrl+C para salir.");

    // 1.4. Bombeo de mensajes: sin esto el hook deja de recibir eventos. Sale cuando el handler postea WM_QUIT.
    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    // 1.5. Al salir, para la detección limpiamente.
    super::detection::stop(hook)?;
    println!("Hook desinstalado. Saliendo.");

    Ok(())
}

// 2. Pide al hilo del bombeo que salga: postea WM_QUIT a su cola de mensajes.
// pub(super) temporal: detection lo usa en el [STOP DIAGNÓSTICO]; pasa a privado con la tarea 7.
pub(super) fn request_quit() {
    let tid = MAIN_THREAD_ID.load(Ordering::Relaxed);
    let _ = unsafe { PostThreadMessageW(tid, WM_QUIT, WPARAM(0), LPARAM(0)) };
}

// 3. Handler de cierre de consola.
unsafe extern "system" fn console_ctrl_handler(ctrl_type: u32) -> BOOL {
    match ctrl_type {
        CTRL_C_EVENT | CTRL_BREAK_EVENT | CTRL_CLOSE_EVENT => {
            request_quit();
            HANDLED
        }
        _ => NOT_HANDLED,
    }
}
