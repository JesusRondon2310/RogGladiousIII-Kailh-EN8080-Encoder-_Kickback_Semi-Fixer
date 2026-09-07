//! injector.rs
//! Hilo inyector: encola peticiones y ejecuta SendInput en un hilo separado.

use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::sync::OnceLock;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_WHEEL, MOUSEINPUT,
};

static INJECTOR_SENDER: OnceLock<Sender<i32>> = OnceLock::new();

fn get_sender() -> &'static Sender<i32> {
    INJECTOR_SENDER.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<i32>();
        thread::spawn(move || injector_loop(rx));
        tx
    })
}

fn injector_loop(rx: Receiver<i32>) {
    for dir in rx {
        inject_synthetic_tick(dir);
    }
}

pub fn enqueue_manager(direction: i32) {
    let _ = get_sender().send(direction);
}

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
    unsafe {
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

pub fn shutdown_injector() {
   
}