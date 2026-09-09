package filter

import (
	"fmt"
	"sync/atomic"
	"syscall"
	"unsafe"

	inner "Kickback_Fix/src/filter/internal"
	"Kickback_Fix/src/helpers"
)

var kernel32 = syscall.NewLazyDLL("kernel32.dll")
var user32 = syscall.NewLazyDLL("user32.dll")
var procGetCurrentThreadId = kernel32.NewProc("GetCurrentThreadId")
var procSetConsoleCtrlHandler = kernel32.NewProc("SetConsoleCtrlHandler")
var procPostThreadMessageW = user32.NewProc("PostThreadMessageW")
var procGetMessageW = user32.NewProc("GetMessageW")
var procTranslateMessage = user32.NewProc("TranslateMessage")
var procDispatchMessageW = user32.NewProc("DispatchMessageW")

// NewCallback asigna un trampolín C->Go: se crea UNA sola vez.
var ctrlHandlerCallback = syscall.NewCallback(consoleCtrlHandler)
var mainThreadID atomic.Uint32

type msg struct {
	hwnd    uintptr
	message uint32
	wParam  uintptr
	lParam  uintptr
	time    uint32
	pt      struct{ x, y int32 }
	private uint32
}

// 1. Arranca injector y detección, corre el bombeo de mensajes hasta un cierre, y limpia.
func Run() error {
	// 1.1. Registra este hilo como destino del WM_QUIT y engancha el handler de cierre.
	tid, _, _ := procGetCurrentThreadId.Call()
	mainThreadID.Store(uint32(tid))
	if r, _, err := procSetConsoleCtrlHandler.Call(ctrlHandlerCallback, 1); r == 0 {
		return err
	}

	// 1.2. Arranca el hilo inyector antes de la detección.
	inner.StartInjector()

	// 1.3. Arranca la detección de ticks: requestQuit va como callback de tope.
	hook, err := inner.StartHook(requestQuit)
	if err != nil {
		return err
	}
	fmt.Println("Filtro activo (v2 hasta tarea 4). Ctrl+C para salir.")

	// 1.4. Bombeo de mensajes: sin esto el hook deja de recibir eventos. GetMessageW devuelve 0 en WM_QUIT, -1 en error.
	var m msg
	for {
		if r, _, _ := procGetMessageW.Call(uintptr(unsafe.Pointer(&m)), 0, 0, 0); int32(r) <= 0 {
			break
		}
		procTranslateMessage.Call(uintptr(unsafe.Pointer(&m)))
		procDispatchMessageW.Call(uintptr(unsafe.Pointer(&m)))
	}

	// 1.5. Al salir, para la detección limpiamente.
	if err := inner.StopHook(hook); err != nil {
		return err
	}
	fmt.Println("Hook desinstalado. Saliendo.")
	return nil
}

// 2. Postea WM_QUIT al hilo del bombeo para que salga. detection lo recibe como onCap al llegar al tope diagnóstico. Es var para
// que los tests del ctrl handler lo sustituyan.
var requestQuit = func() {
	procPostThreadMessageW.Call(uintptr(mainThreadID.Load()), uintptr(helpers.WM_QUIT), 0, 0)
}

// 3. Windows lo llama en Ctrl+C / cierre de ventana. ctrlType llega como uintptr.
func consoleCtrlHandler(ctrlType uintptr) uintptr {
	switch uint32(ctrlType) {
	case helpers.CTRL_C_EVENT, helpers.CTRL_BREAK_EVENT, helpers.CTRL_CLOSE_EVENT:
		requestQuit()
		return uintptr(helpers.HANDLED)
	default:
		return uintptr(helpers.NOT_HANDLED)
	}
}
