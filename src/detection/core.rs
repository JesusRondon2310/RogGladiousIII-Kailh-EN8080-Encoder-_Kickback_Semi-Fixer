//! detection/core.rs

use std::sync::atomic::{AtomicU32, Ordering};
use windows::Win32::Foundation::{BOOL, LPARAM, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::System::Console::{
    CTRL_BREAK_EVENT, CTRL_C_EVENT, CTRL_CLOSE_EVENT, SetConsoleCtrlHandler,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PostThreadMessageW, TranslateMessage, WM_QUIT,
};

// Hilo que corre el bombeo de mensajes; destino del WM_QUIT de salida.
static MAIN_THREAD_ID: AtomicU32 = AtomicU32::new(0);

// 1. Punto de entrada del módulo. Instala el hook, arranca el inyector, corre el bombeo de mensajes hasta un cierre y limpia.
pub fn run() -> windows::core::Result<()> {
    unsafe {
        // 1.1. Registra este hilo como destino del WM_QUIT y engancha el handler de cierre de consola.
        MAIN_THREAD_ID.store(GetCurrentThreadId(), Ordering::Relaxed);
        SetConsoleCtrlHandler(Some(console_ctrl_handler), true)?;

        // 1.2. Arranca el hilo inyector antes de instalar el hook.
        super::synthetic::start();

        // 1.3. Instala el hook de bajo nivel (lo gestiona `physical`).
        let hook = super::physical::install()?;
        println!("Filtro activo (v2 hasta tarea 4). Ctrl+C para salir.");

        // 1.4. Bombeo de mensajes: sin esto el hook deja de recibir eventos. Sale cuando el handler postea WM_QUIT.
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // 1.5. Al salir, desinstala el hook limpiamente.
        super::physical::uninstall(hook)?;
        println!("Hook desinstalado. Saliendo.");
    }

    Ok(())
}

// 2. Pide al hilo del bombeo que salga: postea WM_QUIT a su cola de mensajes.
pub(super) fn request_quit() {
    let tid = MAIN_THREAD_ID.load(Ordering::Relaxed);
    let _ = unsafe { PostThreadMessageW(tid, WM_QUIT, WPARAM(0), LPARAM(0)) };
}

// 3. Handler de cierre de consola.
unsafe extern "system" fn console_ctrl_handler(ctrl_type: u32) -> BOOL {
    match ctrl_type {
        CTRL_C_EVENT | CTRL_BREAK_EVENT | CTRL_CLOSE_EVENT => {
            request_quit();
            BOOL(1)
        }
        _ => BOOL(0),
    }
}
