use std::sync::mpsc::{self, SyncSender};
use std::sync::OnceLock;
use std::thread;

use windows_sys::Win32::UI::Input::KeyboardAndMouse::{INPUT, INPUT_0, MOUSEINPUT, SendInput};

use crate::helpers::constants as consts;
use crate::trace;

// injector_tx lleva las direcciones a inyectar desde el hook hasta el hilo inyector. El hook nunca llama a SendInput directamente:
// hacerlo dentro del callback bloquea el raw input thread contra sí mismo = deadlock.
static INJECTOR_TX: OnceLock<SyncSender<i32>> = OnceLock::new();

// 1. Encola un tick sintético para el hilo inyector sin bloquear el hook.
pub(super) fn enqueue_manager(direction: i32) -> bool {
    match INJECTOR_TX.get() {
        Some(tx) => tx.try_send(direction).is_ok(),
        None => false,
    }
}

// 2. Construye e inyecta un tick de rueda sintético en `direction`. Corre siempre en el hilo inyector, nunca dentro del hook.
fn execute(direction: i32) {
    if direction != consts::WHEEL_UP && direction != consts::WHEEL_DOWN {
        trace!("[INYECCIÓN] dirección inválida: {}\n", direction);
        return;
    }

    let input = INPUT {
        r#type: consts::MOUSE_INPUT,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: (direction * consts::WHEEL_TICK_UNIT) as u32,
                dwFlags: consts::WHEEL_MOVE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let sent = unsafe { SendInput(1, &input, size_of::<INPUT>() as i32) };

    if sent == consts::ONE_EVENT {
        trace!("[INYECTADO] direction={}\n", direction);
        return;
    }
    let err = std::io::Error::last_os_error();
    trace!("[INYECCIÓN FALLÓ] direction={} sent={} err={}\n", direction, sent, err);
}

// 3. Recibe del canal y delega en execute.
fn injector_loop(rx: mpsc::Receiver<i32>) {
    trace!("[INYECTOR] hilo arrancado\n");
    for direction in rx {
        trace!("[INYECTOR] recibido direction={}\n", direction);
        execute(direction);
    }
    trace!("[INYECTOR] canal cerrado, hilo termina\n");
}

// 4. Arranca el hilo inyector: espera direcciones por el canal y ejecuta la inyección fuera del contexto del hook.
pub(super) fn start_injector() {
    let (tx, rx) = mpsc::sync_channel::<i32>(16);
    let _ = INJECTOR_TX.set(tx);
    thread::spawn(move || injector_loop(rx));
}
