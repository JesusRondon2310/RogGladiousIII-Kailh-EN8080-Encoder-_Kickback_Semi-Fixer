// ---- Dirección de la rueda ----
pub const WHEEL_UP: i32 = 1;
pub const WHEEL_DOWN: i32 = -1;

// ---- Rueda / hook de bajo nivel ----
pub const WHEEL_TICK_UNIT: i32 = 120; // un "click" de rueda (WHEEL_DELTA)
pub const SELF_INJECTED: u32 = 0x1; // MSLLHOOKSTRUCT.flags: el evento lo inyectamos nosotros (LLMHF_INJECTED)
pub const MOUSE_HOOK: i32 = 14; // SetWindowsHookExW: hook de mouse de bajo nivel (WH_MOUSE_LL)
pub const WHEEL_EVENT: usize = 0x020A; // wParam del hook para un evento de rueda (WM_MOUSEWHEEL)

// ---- Retornos de callbacks ----
pub const BLOCK: isize = 1; // retorno del hook: no pasar el evento al siguiente (LRESULT 1)
pub const HANDLED: i32 = 1; // console ctrl handler: lo manejamos nosotros (BOOL)
pub const NOT_HANDLED: i32 = 0; // console ctrl handler: que siga el default (BOOL)

// ---- Message pump / consola ----
pub const QUIT_MESSAGE: u32 = 0x0012; // mensaje de salida del bombeo (WM_QUIT)
pub const CTRL_C_EVENT: u32 = 0;
pub const CTRL_BREAK_EVENT: u32 = 1;
pub const CTRL_CLOSE_EVENT: u32 = 2;

// ---- SendInput ----
pub const MOUSE_INPUT: u32 = 0; // INPUT.type = mouse (INPUT_MOUSE)
pub const WHEEL_MOVE: u32 = 0x0800; // MOUSEINPUT.dwFlags: movimiento de rueda (MOUSEEVENTF_WHEEL)
pub const ONE_EVENT: u32 = 1; // SendInput debe insertar exactamente 1

// ---- Config del filtro (tuning; manipulables en runtime más adelante) ----
pub const SILENCE_TICKS: i32 = 3; // racha que se bloquea en silencio antes de arrancar la compensación
pub const TRUST_TICKS: i32 = 7; // racha total a la que la dirección se da por confirmada; el tick 8+ pasa
