package filter

import (
	"fmt"
	"runtime"
	"unsafe"

	"Kickback_Fix/src/helpers"
	"golang.org/x/sys/windows"
)

// injectorCh lleva las direcciones a inyectar desde el hook hasta la goroutine inyectora. El hook nunca llama a SendInput directamente:
// hacerlo dentro del callback bloquea el raw input thread contra sí mismo = deadlock.
var injectorCh = make(chan int32, 16)
var user32 = windows.NewLazySystemDLL("user32.dll")
var procSendInput = user32.NewProc("SendInput")

type mouseInput struct {
	dx          int32
	dy          int32
	mouseData   uint32
	dwFlags     uint32
	time        uint32
	dwExtraInfo uintptr
}

type mouseInputEvent struct {
	inputType uint32
	mi        mouseInput
}

// 1. Encola un tick sintético para el hilo inyector sin bloquear el hook. Devuelve false si el buffer está lleno (el sintético
// se pierde; con buffer 16 y compensación de ~4 ticks no debería pasar).
func enqueueManager(direction int32) bool {
	select {
	case injectorCh <- direction:
		return true
	default:
		return false
	}
}

// buildInput arma el evento de rueda para `direction`. Puro: sin efectos, testeable sin tocar SendInput.
func buildInput(direction int32) mouseInputEvent {
	return mouseInputEvent{
		inputType: helpers.MOUSE_INPUT,
		mi: mouseInput{
			mouseData: uint32(direction * helpers.WHEEL_TICK_UNIT),
			dwFlags:   helpers.WHEEL_MOVE,
		},
	}
}

// 2. Construye e inyecta un tick de rueda sintético en `direction`. Corre siempre en el hilo inyector, nunca dentro del hook.
func execute(direction int32) {
	if direction != helpers.WHEEL_UP && direction != helpers.WHEEL_DOWN {
		fmt.Printf("[INYECCIÓN] dirección inválida: %d\n", direction)
		return
	}

	in := buildInput(direction)

	sent, _, callErr := procSendInput.Call(
		uintptr(1),
		uintptr(unsafe.Pointer(&in)),
		unsafe.Sizeof(in),
	)
	runtime.KeepAlive(in)

	if uint32(sent) == helpers.ONE_EVENT {
		fmt.Printf("[INYECTADO] direction=%d\n", direction)
		return
	}
	fmt.Printf("[INYECCIÓN FALLÓ] direction=%d sent=%d err=%v\n", direction, sent, callErr)
}

// 3. Recibe del canal y delega en execute.
func injectorLoop() {
	fmt.Println("[INYECTOR] goroutine arrancada")
	for direction := range injectorCh {
		fmt.Printf("[INYECTOR] recibido direction=%d\n", direction)
		execute(direction)
	}
	fmt.Println("[INYECTOR] canal cerrado, goroutine termina")
}

// 4. Arranca el hilo inyector: espera direcciones por el canal y ejecuta la inyección fuera del contexto del hook.
func StartInjector() {
	go injectorLoop()
}
