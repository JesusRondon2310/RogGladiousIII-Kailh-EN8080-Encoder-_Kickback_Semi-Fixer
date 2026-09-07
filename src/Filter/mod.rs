//! filter/mod.rs
//! Orquestador del módulo de filtro.

mod constants;
mod streak;
mod machine;
mod injector;
mod detection;

use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, TranslateMessage, DispatchMessageW, MSG};

/// Punto de entrada principal del filtro.
pub fn run() -> windows::core::Result<()> {
    // Instalar el hook.
    let hook = detection::install_hook()?;
    println!("[filter] Hook instalado. Esperando eventos...");

    // Bucle de mensajes (necesario para que el hook funcione).
    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    // Al salir, desinstalar el hook.
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::UnhookWindowsHookEx;
        UnhookWindowsHookEx(hook)?;
    }
    println!("[filter] Hook desinstalado.");

    // Apagar el inyector (opcional).
    injector::shutdown_injector();

    Ok(())
}