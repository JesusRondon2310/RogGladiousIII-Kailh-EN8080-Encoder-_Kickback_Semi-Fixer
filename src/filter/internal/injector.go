package filter

import (
	"fmt"
	"runtime"
	"sync/atomic"
	"syscall"
	"unsafe"

	"Kickback_Fix/src/helpers"
)

// injectorCh lleva las direcciones a inyectar desde el hook hasta la goroutine inyectora. El hook nunca llama a SendInput directamente:
// hacerlo dentro del callback bloquea el raw input thread contra sí mismo = deadlock.
var injectorCh = make(chan int32, 16)
var injectionsSinceReset atomic.Int32
var user32 = syscall.NewLazyDLL("user32.dll")
var procSendInput = user32.NewProc("SendInput")

type mouseInput struct {
	dx          int32
	dy          int32
	mouseData   uint32
	dwFlags     uint32
	time        uint32
	dwExtraInfo uintptr
}

type input struct {
	inputType uint32
	mi        mouseInput
}

type enqueueResult int32

const (
	enqueued enqueueResult = iota
	queueFull
	atCap
)

// 1. Cuenta y encola una inyección para el hilo inyector. Al llegar al tope diagnóstico devuelve Tope sin encolar.
func enqueueManager(direction int32) enqueueResult {
	if injectionsSinceReset.Load() >= helpers.DIAG_INJECTION_LIMIT {
		fmt.Printf("[STOP DIAGNÓSTICO] tope %d alcanzado\n", helpers.DIAG_INJECTION_LIMIT)
		return atCap
	}

	injectionsSinceReset.Add(1)
	select {
	case injectorCh <- direction:
		return enqueued
	default:
		return queueFull
	}
}

// 2. Reinicia el contador de inyecciones. detection lo llama al cambiar la dirección de la racha.
func resetInjectionsCounter() {
	injectionsSinceReset.Store(0)
}

// testeable sin tocar SendInput.
func buildInput(direction int32) input {
	return input{
		inputType: helpers.INPUT_MOUSE,
		mi: mouseInput{
			mouseData: uint32(direction * helpers.WHEEL_TICK_UNIT),
			dwFlags:   helpers.MOUSEEVENTF_WHEEL,
		},
	}
}

// 3. Construye e inyecta un tick de rueda sintético en `direction`. Corre siempre en el hilo inyector, nunca dentro del hook.
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

// 4. recibe del canal y delega en execute
func injectorLoop() {
	fmt.Println("[INYECTOR] goroutine arrancada")
	for direction := range injectorCh {
		fmt.Printf("[INYECTOR] recibido direction=%d\n", direction)
		execute(direction)
	}
	fmt.Println("[INYECTOR] canal cerrado, goroutine termina")
}

// 5. Arranca el hilo inyector: espera direcciones por el canal y ejecuta la inyección fuera del contexto del hook.
func StartInjector() {
	go injectorLoop()
}
