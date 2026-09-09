package filter

// Tests de la lógica de detection. SetWindowsHookExW / UnhookWindowsHookEx (InstallHook / UninstallHook) necesitan un escritorio
// real y un message loop — van en integración. Acá: layout del struct, decodificación de dirección, racha, y el flujo del hook
// proc con passThrough sustituido.

import (
	"testing"
	"unsafe"

	"Kickback_Fix/src/helpers"
)

// resetDetectionState: racha en cero, contador de inyecciones en cero, canal drenado, sin callback de tope.
func resetDetectionState() {
	lastDir.Store(0)
	streakCount.Store(0)
	resetInjectorState()
	onDiagnosticCap = nil
}

// wheelEvent arma un MSLLHOOKSTRUCT como el que entrega Windows: dirección en el HIWORD de mouseData.
func wheelEvent(direction int32, injected bool) msllHookStruct {
	var ev msllHookStruct
	ev.mouseData = uint32(direction*helpers.WHEEL_TICK_UNIT) << 16
	if injected {
		ev.flags = helpers.LLMHF_INJECTED
	}
	return ev
}

func TestMouseHookLayout(t *testing.T) {
	if got := unsafe.Sizeof(point{}); got != 8 {
		t.Errorf("point = %d bytes, quiero 8", got)
	}
	if got := unsafe.Sizeof(msllHookStruct{}); got != 32 {
		t.Errorf("msllHookStruct = %d bytes, quiero 32", got)
	}
}

// TestWheelDirection: el HIWORD de mouseData es un short con signo. -120 (rueda hacia el usuario) tiene que leerse como WHEEL_DOWN,
// no como un positivo enorme.
func TestWheelDirection(t *testing.T) {
	cases := []struct {
		name      string
		mouseData uint32
		want      int32
	}{
		{"arriba +120", 0x00780000, helpers.WHEEL_UP},
		{"abajo -120", 0xFF880000, helpers.WHEEL_DOWN},
		{"abajo -1", 0xFFFF0000, helpers.WHEEL_DOWN},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			if got := wheelDirection(c.mouseData); got != c.want {
				t.Errorf("wheelDirection(%#x) = %d, quiero %d", c.mouseData, got, c.want)
			}
		})
	}
}

func TestUpdateStreak(t *testing.T) {
	t.Run("misma dirección suma", func(t *testing.T) {
		resetDetectionState()
		for i := int32(1); i <= 3; i++ {
			if got := updateStreak(helpers.WHEEL_UP); got != i {
				t.Fatalf("tick %d: streak = %d, quiero %d", i, got, i)
			}
		}
	})

	t.Run("cambio de dirección reinicia y limpia inyecciones", func(t *testing.T) {
		resetDetectionState()
		updateStreak(helpers.WHEEL_UP)
		updateStreak(helpers.WHEEL_UP)
		injectionsSinceReset.Store(2) // simula inyecciones en curso

		if got := updateStreak(helpers.WHEEL_DOWN); got != 1 {
			t.Fatalf("tras el cambio: streak = %d, quiero 1", got)
		}
		if got := injectionsSinceReset.Load(); got != 0 {
			t.Fatalf("el contador de inyecciones no se reinició: %d", got)
		}
		if got := lastDir.Load(); got != helpers.WHEEL_DOWN {
			t.Fatalf("lastDir = %d, quiero %d", got, helpers.WHEEL_DOWN)
		}
	})
}

func TestMouseWheelCatcherHook(t *testing.T) {
	origPassThrough := passThrough
	t.Cleanup(func() { passThrough = origPassThrough })

	t.Run("evento inyectado pasa directo y no toca la racha", func(t *testing.T) {
		resetDetectionState()
		streakCount.Store(5)
		var passed bool
		passThrough = func(_, _, _ uintptr) uintptr { passed = true; return 0 }

		ev := wheelEvent(helpers.WHEEL_UP, true)
		mouseWheelCatcherHook(0, uintptr(helpers.WM_MOUSEWHEEL), unsafe.Pointer(&ev))

		if !passed {
			t.Error("no llamó a passThrough")
		}
		if got := streakCount.Load(); got != 5 {
			t.Errorf("la racha cambió: %d, quiero 5", got)
		}
	})

	t.Run("evento que no es de rueda pasa directo", func(t *testing.T) {
		resetDetectionState()
		streakCount.Store(5)
		var passed bool
		passThrough = func(_, _, _ uintptr) uintptr { passed = true; return 0 }

		ev := wheelEvent(helpers.WHEEL_UP, false)
		mouseWheelCatcherHook(0, 0x0200 /* WM_MOUSEMOVE */, unsafe.Pointer(&ev))

		if !passed {
			t.Error("no llamó a passThrough")
		}
		if got := streakCount.Load(); got != 5 {
			t.Errorf("la racha cambió: %d, quiero 5", got)
		}
	})

	t.Run("bajo el umbral bloquea el tick", func(t *testing.T) {
		resetDetectionState()
		passThrough = func(_, _, _ uintptr) uintptr { return 99 }

		ev := wheelEvent(helpers.WHEEL_DOWN, false)
		ret := mouseWheelCatcherHook(0, uintptr(helpers.WM_MOUSEWHEEL), unsafe.Pointer(&ev))

		if ret != helpers.BLOCK {
			t.Errorf("ret = %d, quiero BLOCK (%d)", ret, helpers.BLOCK)
		}
		if got := streakCount.Load(); got != 1 {
			t.Errorf("streak = %d, quiero 1", got)
		}
	})

	t.Run("al llegar al tope diagnóstico dispara onDiagnosticCap", func(t *testing.T) {
		resetDetectionState()
		passThrough = func(_, _, _ uintptr) uintptr { return 0 }
		var fired bool
		onDiagnosticCap = func() { fired = true }

		// vigilancia arranca en WATCH_THRESHOLD; el tope corta tras DIAG_INJECTION_LIMIT inyecciones.
		total := helpers.WATCH_THRESHOLD + helpers.DIAG_INJECTION_LIMIT + 1
		ev := wheelEvent(helpers.WHEEL_UP, false)
		for i := int32(0); i < total; i++ {
			mouseWheelCatcherHook(0, uintptr(helpers.WM_MOUSEWHEEL), unsafe.Pointer(&ev))
		}

		if !fired {
			t.Error("no se disparó onDiagnosticCap al llegar al tope")
		}
	})
}
