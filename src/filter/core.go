package filter

import (
	"fmt"
	"sync/atomic"
	"unsafe"

	"Kickback_Fix/src/exception"
	inner "Kickback_Fix/src/filter/internal"
	"Kickback_Fix/src/helpers"
	"golang.org/x/sys/windows"
)

var kernel32 = windows.NewLazySystemDLL("kernel32.dll")
var user32 = windows.NewLazySystemDLL("user32.dll")
var procSetConsoleCtrlHandler = kernel32.NewProc("SetConsoleCtrlHandler")
var procPostThreadMessageW = user32.NewProc("PostThreadMessageW")
var procGetMessageW = user32.NewProc("GetMessageW")
var procTranslateMessage = user32.NewProc("TranslateMessage")
var procDispatchMessageW = user32.NewProc("DispatchMessageW")

// NewCallback asigna un trampolín C->Go: se crea UNA sola vez.
var ctrlHandlerCallback = windows.NewCallback(consoleCtrlHandler)
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
func Run() (err error) {
	defer exception.Catch(&err)

	// 1.1. Este hilo recibe el mensaje de salida; engancha el handler de cierre.
	mainThreadID.Store(windows.GetCurrentThreadId())
	if r, _, e := procSetConsoleCtrlHandler.Call(ctrlHandlerCallback, 1); r == 0 {
		return e
	}

	// 1.2. Arranca el hilo inyector antes de la detección.
	inner.StartInjector()

	// 1.3. Arranca la detección de ticks: requestQuit va como callback de tope.
	hook := exception.Try(inner.StartHook(requestQuit))
	fmt.Println("Filtro activo (v2 hasta tarea 4). Ctrl+C para salir.")

	// 1.4. Bombeo de mensajes: sin esto el hook deja de recibir eventos. GetMessageW devuelve 0 en QUIT_MESSAGE, -1 en error.
	var m msg
	for {
		if r, _, _ := procGetMessageW.Call(uintptr(unsafe.Pointer(&m)), 0, 0, 0); int32(r) <= 0 {
			break
		}
		procTranslateMessage.Call(uintptr(unsafe.Pointer(&m)))
		procDispatchMessageW.Call(uintptr(unsafe.Pointer(&m)))
	}

	// 1.5. Al salir, para la detección limpiamente.
	exception.TryWithErrorReturnFunc(inner.StopHook(hook))
	fmt.Println("Hook desinstalado. Saliendo.")
	return
}

// 2. Postea el mensaje de salida al hilo del bombeo. detection lo recibe como onCap al llegar al tope. Es var para que los
// tests del ctrl handler lo sustituyan.
var requestQuit = func() {
	procPostThreadMessageW.Call(uintptr(mainThreadID.Load()), uintptr(helpers.QUIT_MESSAGE), 0, 0)
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
