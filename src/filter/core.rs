use super::{console, detection, injector};

// 1. Arranca injector y detección, corre el bombeo de mensajes hasta un cierre, y limpia.
pub fn run() -> std::io::Result<()> {
    // 1.1. Este hilo recibe el mensaje de salida; engancha el handler de cierre.
    console::install_quit_handler()?;

    // 1.2. Arranca el hilo inyector antes de la detección.
    injector::start_injector();

    // 1.3. Arranca la detección de ticks.
    let hook = detection::start_hook()?;
    println!("Filtro activo. Ctrl+C para salir.");

    // 1.4. Bombeo de mensajes: sin esto el hook deja de recibir eventos.
    console::pump_messages();

    // 1.5. Al salir, para la detección limpiamente.
    detection::stop_hook(hook)?;
    println!("Saliendo...");
    Ok(())
}
