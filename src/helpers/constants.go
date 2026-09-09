package helpers

// ---- Dirección de la rueda ----
const WHEEL_UP int32 = 1
const WHEEL_DOWN int32 = -1

// ---- Rueda / hook de bajo nivel (Win32) ----
const WHEEL_TICK_UNIT int32 = 120   // WHEEL_DELTA — un notch de rueda
const LLMHF_INJECTED uint32 = 0x1   // flag "inyectado por nosotros" en MSLLHOOKSTRUCT
const WH_MOUSE_LL int32 = 14        // SetWindowsHookExW: id del hook de mouse de bajo nivel
const WM_MOUSEWHEEL uint32 = 0x020A // wparam del hook: el evento es de rueda

// ---- Retornos de callbacks Win32 ----
const BLOCK uintptr = 1     // callback del hook: no pasar el evento al siguiente (LRESULT)
const HANDLED int32 = 1     // console ctrl handler: lo manejamos nosotros (BOOL)
const NOT_HANDLED int32 = 0 // console ctrl handler: que siga el default (BOOL)

// ---- Message pump / consola (Win32) ----
const WM_QUIT uint32 = 0x0012
const CTRL_C_EVENT uint32 = 0
const CTRL_BREAK_EVENT uint32 = 1
const CTRL_CLOSE_EVENT uint32 = 2

// ---- SendInput ----
const INPUT_MOUSE uint32 = 0            // INPUT.type: el evento es de mouse
const MOUSEEVENTF_WHEEL uint32 = 0x0800 // MOUSEINPUT.dwFlags: el movimiento es de rueda
const ONE_EVENT uint32 = 1              // esperamos que SendInput inserte exactamente 1

// ---- Config del filtro (tuning; candidatas a runtime más adelante) ----
const WATCH_THRESHOLD int32 = 3      // racha para arrancar vigilancia + compensación
const DIAG_INJECTION_LIMIT int32 = 3 // tope de inyecciones por gesto (diagnóstico)
