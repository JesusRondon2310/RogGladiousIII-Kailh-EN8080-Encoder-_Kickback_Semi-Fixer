package filter

import (
	"fmt"
	"sync/atomic"
	"unsafe"

	"Kickback_Fix/src/helpers"

	"golang.org/x/sys/windows"
)

var kernel32 = windows.NewLazySystemDLL("kernel32.dll")
var procGetModuleHandleW = kernel32.NewProc("GetModuleHandleW")
var procSetWindowsHookExW = user32.NewProc("SetWindowsHookExW")
var procCallNextHookEx = user32.NewProc("CallNextHookEx")
var procUnhookWindowsHookEx = user32.NewProc("UnhookWindowsHookEx")

// NewCallback asigna un trampolín C->Go y hay un tope global: se crea UNA sola vez.
var mouseHookCallback = windows.NewCallback(mouseWheelCatcherHook)

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

// 1. Racha de ticks consecutivos en la misma dirección. Si coincide con la última, suma uno; si no, arranca de nuevo en 1.
func updateStreak(direction int32) int32 {
	if direction == lastDir.Load() {
		return streakCount.Add(1)
	}
	lastDir.Store(direction)
	streakCount.Store(1)
	return 1
}

// 2. Windows lo llama en cada evento de mouse, en el hilo que instaló el hook. lParam apunta a un MSLLHOOKSTRUCT válido solo
// durante la llamada.
func mouseWheelCatcherHook(nCode, wParam uintptr, lParam unsafe.Pointer) uintptr {
	// 2.1. Solo eventos de rueda; el resto pasa directo. nCode llega como uintptr: se lee con signo.
	if int32(nCode) < 0 || uint32(wParam) != helpers.WHEEL_EVENT {
		return passThrough(nCode, wParam, uintptr(lParam))
	}
	event := (*msllHookStruct)(lParam)

	// 2.2. Un evento inyectado por nosotros pasa sin re-procesar, o el hook se dispara a sí mismo en bucle.
	if event.flags&helpers.SELF_INJECTED != 0 {
		return passThrough(nCode, wParam, uintptr(lParam))
	}

	// 2.3. Dirección de este tick + racha acumulada.
	direction := wheelDirection(event.mouseData)
	racha := updateStreak(direction)

	// 2.4. Decidir según la racha
	switch decide(racha) {
	case pass:
		fmt.Printf("[PASA] dir=%d racha=%d\n", direction, racha)
		return passThrough(nCode, wParam, uintptr(lParam))
	case blockAndInject:
		fmt.Printf("[COMPENSA] dir=%d racha=%d\n", direction, racha)
		enqueueManager(direction)
		return helpers.BLOCK
	default:
		fmt.Printf("[SILENCIO] dir=%d racha=%d\n", direction, racha)
		return helpers.BLOCK
	}
}

// 3. Arranca la detección apuntando a mouseWheelCatcherHook. Devuelve el handle para pararlo.
func StartHook() (windows.Handle, error) {
	hmod, _, _ := procGetModuleHandleW.Call(0)
	hook, _, err := procSetWindowsHookExW.Call(uintptr(helpers.MOUSE_HOOK), mouseHookCallback, hmod, 0)
	if hook == 0 {
		return 0, err
	}
	return windows.Handle(hook), nil
}

// 4. Para la detección.
func StopHook(hook windows.Handle) error {
	if ret, _, err := procUnhookWindowsHookEx.Call(uintptr(hook)); ret == 0 {
		return err
	}
	return nil
}
