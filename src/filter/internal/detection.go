package filter

import (
	"fmt"
	"sync/atomic"
	"syscall"
	"unsafe"

	"Kickback_Fix/src/helpers"
)

var kernel32 = syscall.NewLazyDLL("kernel32.dll")
var procGetModuleHandleW = kernel32.NewProc("GetModuleHandleW")
var procSetWindowsHookExW = user32.NewProc("SetWindowsHookExW")
var procCallNextHookEx = user32.NewProc("CallNextHookEx")
var procUnhookWindowsHookEx = user32.NewProc("UnhookWindowsHookEx")

// NewCallback asigna un trampolín C->Go y hay un tope global: se crea UNA sola vez.
var hookCallback = syscall.NewCallback(mouseWheelCatcherHook)

// onDiagnosticCap lo setea StartHook. Windows llama al mouseWheelCatcherHook sin contexto: la única vía de avisarle al orquestador
// "se llegó al tope diagnóstico" es una variable de paquete.
var onDiagnosticCap func()

var lastDir atomic.Int32
var streakCount atomic.Int32

type point struct {
	x int32
	y int32
}

type msllHookStruct struct {
	pt          point
	mouseData   uint32
	flags       uint32
	time        uint32
	dwExtraInfo uintptr
}

var passThrough = func(nCode, wParam, lParam uintptr) uintptr {
	ret, _, _ := procCallNextHookEx.Call(0, nCode, wParam, lParam)
	return ret
}

func wheelDirection(mouseData uint32) int32 {
	if int16(mouseData>>16) > 0 {
		return helpers.WHEEL_UP
	}
	return helpers.WHEEL_DOWN
}

// 1. Racha de ticks consecutivos en la misma dirección. Si coincide con la última, suma uno; si no, arranca en 1 y reinicia
// el contador de inyecciones.
func updateStreak(direction int32) int32 {
	if direction == lastDir.Load() {
		return streakCount.Add(1)
	}
	lastDir.Store(direction)
	streakCount.Store(1)
	resetInjectionsCounter()
	return 1
}

// 2. Windows lo llama en cada evento de mouse, en el hilo que instaló el hook. lParam apunta a un MSLLHOOKSTRUCT válido solo
// durante la llamada.
func mouseWheelCatcherHook(nCode, wParam uintptr, lParam unsafe.Pointer) uintptr {
	// 2.1. Solo eventos de rueda; el resto pasa directo. nCode llega como uintptr: se lee con signo.
	if int32(nCode) < 0 || uint32(wParam) != helpers.WM_MOUSEWHEEL {
		return passThrough(nCode, wParam, uintptr(lParam))
	}
	event := (*msllHookStruct)(lParam)

	// 2.2. Un evento inyectado por nosotros pasa sin re-procesar, o el hook se dispara a sí mismo en bucle.
	if event.flags&helpers.LLMHF_INJECTED != 0 {
		return passThrough(nCode, wParam, uintptr(lParam))
	}

	// 2.3. Dirección de este tick + racha acumulada.
	direction := wheelDirection(event.mouseData)
	streak := updateStreak(direction)

	// 2.4. Silencio inicial: racha bajo el umbral -> bloquea sin inyectar.
	if streak < helpers.WATCH_THRESHOLD {
		fmt.Printf("[BLOQUEADO] dir=%d streak=%d/%d\n", direction, streak, helpers.WATCH_THRESHOLD)
		return helpers.BLOCK
	}

	// 2.5. Umbral alcanzado: bloquea el físico y pide a injector un sintético.
	result := enqueueManager(direction)
	fmt.Printf("[VIGILANCIA] dir=%d streak=%d encolada=%t\n", direction, streak, result == enqueued)
	if result == atCap && onDiagnosticCap != nil {
		onDiagnosticCap()
	}
	return helpers.BLOCK
}

// 3. Arranca la detección apuntando a mouseWheelCatcherHook. Devuelve el handle para pararlo. onCap se invoca al llegar al tope.
func StartHook(onCap func()) (uintptr, error) {
	onDiagnosticCap = onCap
	hmod, _, _ := procGetModuleHandleW.Call(0)
	hook, _, err := procSetWindowsHookExW.Call(uintptr(helpers.WH_MOUSE_LL), hookCallback, hmod, 0)
	if hook == 0 {
		return 0, err
	}
	return hook, nil
}

// 4. Para la detección.
func StopHook(hook uintptr) error {
	if ret, _, err := procUnhookWindowsHookEx.Call(hook); ret == 0 {
		return err
	}
	return nil
}
