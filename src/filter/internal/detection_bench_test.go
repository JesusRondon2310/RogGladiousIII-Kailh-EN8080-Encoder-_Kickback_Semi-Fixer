package filter

// Benchmarks del hot path: mouseWheelCatcherHook corre en el hilo de input en cada evento de mouse. Objetivo: 0 allocs/op.

import (
	"testing"
	"unsafe"

	"Kickback_Fix/src/helpers"
)

// BenchmarkHookForwardMove: evento que no es de rueda (un mouse move) — cae en el primer guard y se reenvía con passThrough.
// Es el camino más frecuente: el LL hook ve todos los moves y clicks.
func BenchmarkHookForwardMove(b *testing.B) {
	var ev msllHookStruct
	p := unsafe.Pointer(&ev)

	b.ReportAllocs()
	for b.Loop() {
		mouseWheelCatcherHook(0, 0x0200, p) // 0x0200 = WM_MOUSEMOVE
	}
}

// BenchmarkHookWheelSilence: tick de rueda con racha en silencio (bloquea, no inyecta).
func BenchmarkHookWheelSilence(b *testing.B) {
	resetDetectionState()
	lastDir.Store(helpers.WHEEL_DOWN)
	var ev msllHookStruct
	ev.mouseData = 0xFF880000 // -120 en el HIWORD
	p := unsafe.Pointer(&ev)

	b.ReportAllocs()
	for b.Loop() {
		streakCount.Store(1)
		mouseWheelCatcherHook(0, uintptr(helpers.WHEEL_EVENT), p)
	}
}
