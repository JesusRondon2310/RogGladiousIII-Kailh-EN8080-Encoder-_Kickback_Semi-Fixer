package filter

// Tests de la lógica de injector. El SendInput real y el ciclo de vida de la goroutine (injectorLoop / Start) necesitan el hook
// corriendo — van en un test de integración aparte, no acá.

import (
	"testing"
	"unsafe"

	"Kickback_Fix/src/helpers"
)

// resetInjectorState deja el estado de paquete en cero: contador a 0 y canal vacío. Los tests tocan globales, así que corren en serie
// y arrancan cada uno con esto.
func resetInjectorState() {
	resetInjectionsCounter()
	for {
		select {
		case <-injectorCh:
		default:
			return
		}
	}
}

// TestMouseInputEventLayout: si el struct que va a SendInput no mide lo que Win32 espera, le mandamos basura y el layout está mal. Los
// tamaños (32 y 40 en x64) salen de la doc de MOUSEINPUT e INPUT.
func TestMouseInputEventLayout(t *testing.T) {
	if got := unsafe.Sizeof(mouseInput{}); got != 32 {
		t.Errorf("mouseInput = %d bytes, quiero 32", got)
	}
	if got := unsafe.Sizeof(mouseInputEvent{}); got != 40 {
		t.Errorf("mouseInputEvent = %d bytes, quiero 40", got)
	}
}

// TestBuildInput: la lógica real de execute que se puede aislar — que la dirección se traduzca al INPUT correcto. El caso que
// importa es WHEEL_DOWN: uint32(-120) tiene que dar el patrón de bits que Win32 lee de vuelta como -120.
func TestBuildInput(t *testing.T) {
	cases := []struct {
		name          string
		direction     int32
		wantMouseData uint32
	}{
		{"arriba", helpers.WHEEL_UP, 120},
		{"abajo", helpers.WHEEL_DOWN, 0xFFFFFF88}, // -120 en complemento a dos
	}

	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			got := buildInput(c.direction)

			if got.inputType != helpers.MOUSE_INPUT {
				t.Errorf("inputType = %d, quiero %d", got.inputType, helpers.MOUSE_INPUT)
			}
			if got.mi.dwFlags != helpers.WHEEL_MOVE {
				t.Errorf("dwFlags = %#x, quiero %#x", got.mi.dwFlags, helpers.WHEEL_MOVE)
			}
			if got.mi.mouseData != c.wantMouseData {
				t.Errorf("mouseData = %#x, quiero %#x", got.mi.mouseData, c.wantMouseData)
			}
		})
	}
}

// TestEnqueueManager cubre la máquina chica de enqueue: el tope diagnóstico, que el reset lo reabra, y el envío no bloqueante
// cuando el canal está lleno.
func TestEnqueueManager(t *testing.T) {
	t.Run("encola hasta el tope y luego corta", func(t *testing.T) {
		resetInjectorState()

		for i := int32(0); i < helpers.DIAG_INJECTION_LIMIT; i++ {
			if got := enqueueManager(helpers.WHEEL_UP); got != enqueued {
				t.Fatalf("llamada %d: got %d, quiero enqueued", i, got)
			}
		}
		if got := injectionsSinceReset.Load(); got != helpers.DIAG_INJECTION_LIMIT {
			t.Fatalf("contador = %d, quiero %d", got, helpers.DIAG_INJECTION_LIMIT)
		}

		if got := enqueueManager(helpers.WHEEL_UP); got != atCap {
			t.Fatalf("sobre el tope: got %d, quiero atCap", got)
		}
		if got := injectionsSinceReset.Load(); got != helpers.DIAG_INJECTION_LIMIT {
			t.Fatalf("el contador subió pasado el tope: %d", got)
		}
	})

	t.Run("reset reabre el tope", func(t *testing.T) {
		resetInjectorState()

		for i := int32(0); i <= helpers.DIAG_INJECTION_LIMIT; i++ {
			enqueueManager(helpers.WHEEL_UP)
		}
		resetInjectionsCounter()

		if got := enqueueManager(helpers.WHEEL_DOWN); got != enqueued {
			t.Fatalf("tras reset: got %d, quiero enqueued", got)
		}
	})

	t.Run("canal lleno devuelve queueFull sin bloquear", func(t *testing.T) {
		resetInjectorState()

		for i := int32(0); i < int32(cap(injectorCh)); i++ {
			injectorCh <- helpers.WHEEL_UP
		}
		resetInjectionsCounter() // que no corte por tope antes de llegar al select

		if got := enqueueManager(helpers.WHEEL_UP); got != queueFull {
			t.Fatalf("canal lleno: got %d, quiero queueFull", got)
		}
	})
}
