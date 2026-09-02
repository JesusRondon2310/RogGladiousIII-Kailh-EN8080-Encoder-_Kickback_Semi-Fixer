//! filter/injector.rs

use std::sync::OnceLock;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc::{self, Sender};
use windows::Win32::Foundation::GetLastError;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_WHEEL, MOUSEINPUT, SendInput,
};

use crate::helpers::constants::{DIAG_INJECTION_LIMIT, ONE_EVENT, WHEEL_DOWN, WHEEL_TICK_UNIT, WHEEL_UP};

// Canal hacia el hilo inyector. detection solo pide encolar por aquí; nunca se llama a SendInput desde el hook: SendInput dentro
// del callback bloquea el raw input thread contra sí mismo = deadlock (todos los eventos se quedan esperando).
static INJECTOR: OnceLock<Sender<i32>> = OnceLock::new();

// Inyecciones desde el último cambio de dirección de la racha. Lo reinicia detection vía reset_injections_counter().
static INJECTIONS_SINCE_RESET: AtomicI32 = AtomicI32::new(0);

// Resultado de pedir una inyección. detection actúa según la variante.
pub(super) enum EnqueueResult {
    Encolada(bool),
    Tope,
}

// 1. Cuenta y encola una inyección para el hilo inyector. Al llegar al tope diagnóstico devuelve Tope sin encolar.
pub(super) fn enqueue_manager(dir: i32) -> EnqueueResult {
    let count = INJECTIONS_SINCE_RESET.load(Ordering::Relaxed);
    if count >= DIAG_INJECTION_LIMIT {
        println!("[STOP DIAGNÓSTICO] tope {DIAG_INJECTION_LIMIT} alcanzado");
        return EnqueueResult::Tope;
    }
    INJECTIONS_SINCE_RESET.store(count + 1, Ordering::Relaxed);
    match INJECTOR.get() {
        Some(tx) => EnqueueResult::Encolada(tx.send(dir).is_ok()),
        None => EnqueueResult::Encolada(false),
    }
}

// 2. Reinicia el contador de inyecciones. detection lo llama al cambiar la dirección de la racha.
pub(super) fn reset_injections_counter() { INJECTIONS_SINCE_RESET.store(0, Ordering::Relaxed); }

// 3. Construye e inyecta un tick de rueda sintético en `dir`. Corre siempre en el hilo inyector, nunca dentro del hook.
// En vez de asertar, reporta el resultado de SendInput (sent + GetLastError) para diagnosticar.
fn execute(dir: i32) {
    debug_assert!(dir == WHEEL_UP || dir == WHEEL_DOWN, "direccion de inyeccion invalida: {dir}");

    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: (dir * WHEEL_TICK_UNIT) as u32,
                dwFlags: MOUSEEVENTF_WHEEL,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let sent = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
    if sent == ONE_EVENT {
        println!("[INYECTADO] dir={dir}");
        return;
    }
    let err = unsafe { GetLastError() };
    println!("[INYECCION FALLÓ] dir={dir} sent={sent} GetLastError={err:?}");
}

// 4. Arranca el hilo inyector: espera direcciones por el canal y ejecuta la inyección fuera del contexto del hook.
pub(super) fn start() {
    let (tx, rx) = mpsc::channel::<i32>();
    let _ = INJECTOR.set(tx);
    std::thread::spawn(move || {
        println!("[INYECTOR] hilo arrancado");
        while let Ok(dir) = rx.recv() {
            println!("[INYECTOR] recibido dir={dir}");
            execute(dir);
        }
        println!("[INYECTOR] canal cerrado, hilo termina");
    });
}
