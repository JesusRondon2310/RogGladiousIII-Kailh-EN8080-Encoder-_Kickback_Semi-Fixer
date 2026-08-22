#![allow(dead_code, unused_imports)]
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use windows::Win32::Foundation::{HINSTANCE, HMODULE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, HHOOK, MSG, MSLLHOOKSTRUCT, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, WH_MOUSE_LL, WM_MOUSEWHEEL,
};

const CONFIRM_WINDOW_MS: u64 = 400;
const REQUIRED_CONFIRMATIONS: i32 = 3;

static CONFIRMED_DIR: AtomicI32 = AtomicI32::new(0);
static PENDING_DIR: AtomicI32 = AtomicI32::new(0);
static PENDING_COUNT: AtomicI32 = AtomicI32::new(0);
static LAST_TICK_TIME_MS: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && wparam.0 as u32 == WM_MOUSEWHEEL {
        let data = &*(lparam.0 as *const MSLLHOOKSTRUCT);

        let delta = ((data.mouseData >> 16) & 0xFFFF) as i16;
        let dir: i32 = if delta > 0 { 1 } else { -1 };

        let raw_now = now_ms();
        let raw_last = LAST_TICK_TIME_MS.swap(raw_now, Ordering::Relaxed);
        let raw_elapsed = if raw_last == 0 {
            0
        } else {
            raw_now.saturating_sub(raw_last)
        };
        println!("tick: delta={delta} dir={dir} elapsed_desde_ultimo_tick={raw_elapsed}ms");

        let confirmed = CONFIRMED_DIR.load(Ordering::Relaxed);
        let pending_dir = PENDING_DIR.load(Ordering::Relaxed);

        if confirmed == 0 || dir == confirmed {
            CONFIRMED_DIR.store(dir, Ordering::Relaxed);
            PENDING_DIR.store(0, Ordering::Relaxed);
        } else {
            if dir == pending_dir {
                let count = PENDING_COUNT.load(Ordering::Relaxed) + 1;

                if count >= REQUIRED_CONFIRMATIONS {
                    println!("-> CONFIRMADO cambio a dir={dir} (tras {count} ticks)");
                    CONFIRMED_DIR.store(dir, Ordering::Relaxed);
                    PENDING_DIR.store(0, Ordering::Relaxed);
                    PENDING_COUNT.store(0, Ordering::Relaxed);
                } else {
                    println!(
                        "-> BLOQUEADO, esperando más confirmaciones (dir={dir}, count={count}/{REQUIRED_CONFIRMATIONS})"
                    );
                    PENDING_COUNT.store(count, Ordering::Relaxed);
                    return LRESULT(1);
                }
            } else {
                println!("-> BLOQUEADO (nuevo candidato: dir={dir}, pending_dir={pending_dir})");
                PENDING_DIR.store(dir, Ordering::Relaxed);
                PENDING_COUNT.store(0, Ordering::Relaxed);
                return LRESULT(1);
            }
        }
    }

    CallNextHookEx(HHOOK(std::ptr::null_mut()), code, wparam, lparam)
}

fn main() -> windows::core::Result<()> {
    unsafe {
        let hmodule: HMODULE = GetModuleHandleW(None)?;
        let hinstance: HINSTANCE = hmodule.into();
        let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), hinstance, 0)?;

        println!("Filtro activo. Deja esta ventana abierta.");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnhookWindowsHookEx(hook)?;
    }

    Ok(())
}
