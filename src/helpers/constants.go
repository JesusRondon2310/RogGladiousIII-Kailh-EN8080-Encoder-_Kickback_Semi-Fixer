package helpers

// ---- Dirección de la rueda ----
const WHEEL_UP int32 = 1
const WHEEL_DOWN int32 = -1

// ---- Rueda / hook de bajo nivel ----
const WHEEL_TICK_UNIT int32 = 120 // un "click" de rueda (WHEEL_DELTA)
const SELF_INJECTED uint32 = 0x1  // MSLLHOOKSTRUCT.flags: el evento lo inyectamos nosotros (LLMHF_INJECTED)
const MOUSE_HOOK int32 = 14       // SetWindowsHookExW: hook de mouse de bajo nivel (WH_MOUSE_LL)
const WHEEL_EVENT uint32 = 0x020A // wparam del hook para un evento de rueda (WM_MOUSEWHEEL)

// ---- Retornos de callbacks ----
const BLOCK uintptr = 1     // retorno del hook: no pasar el evento al siguiente (LRESULT 1)
const HANDLED int32 = 1     // console ctrl handler: lo manejamos nosotros (BOOL)
const NOT_HANDLED int32 = 0 // console ctrl handler: que siga el default (BOOL)

// ---- Message pump / consola ----
const QUIT_MESSAGE uint32 = 0x0012 // mensaje de salida del bombeo (WM_QUIT)
const CTRL_C_EVENT uint32 = 0
const CTRL_BREAK_EVENT uint32 = 1
const CTRL_CLOSE_EVENT uint32 = 2

// ---- SendInput ----
const MOUSE_INPUT uint32 = 0     // INPUT.type = mouse (INPUT_MOUSE)
const WHEEL_MOVE uint32 = 0x0800 // MOUSEINPUT.dwFlags: movimiento de rueda (MOUSEEVENTF_WHEEL)
const ONE_EVENT uint32 = 1       // SendInput debe insertar exactamente 1

// ---- Config del filtro (tuning; manipulables en runtime más adelante) ----
const SILENCE_TICKS int32 = 3 // racha que se bloquea en silencio antes de arrancar la compensación
const TRUST_TICKS int32 = 7   // racha total a la que la dirección se da por confirmada; el tick 8+ pasa
