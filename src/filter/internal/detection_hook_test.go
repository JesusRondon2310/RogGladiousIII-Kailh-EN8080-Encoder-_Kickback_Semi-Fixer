package filter

// El flujo de mouseWheelCatcherHook. passThrough se sustituye para no disparar el syscall real.
//
//	inyectado / no-rueda         -> passThrough, racha intacta
//	racha en silencio (1..3)     -> BLOCK, nada encolado
//	racha en compensación (4..7) -> BLOCK, 1 sintético encolado
//	racha confirmada (8+)        -> passThrough (no BLOCK)

import (
	"testing"
	"unsafe"

	"Kickback_Fix/src/helpers"
)

// wheelEvent arma un MSLLHOOKSTRUCT como el que entrega Windows: dirección en el HIWORD de mouseData.
func wheelEvent(direction int32, injected bool) msllHookStruct {
	var ev msllHookStruct
	ev.mouseData = uint32(direction*helpers.WHEEL_TICK_UNIT) << 16
	if injected {
		ev.flags = helpers.SELF_INJECTED
	}
	return ev
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
		mouseWheelCatcherHook(0, uintptr(helpers.WHEEL_EVENT), unsafe.Pointer(&ev))

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

	t.Run("racha en silencio: bloquea sin inyectar", func(t *testing.T) {
		resetDetectionState() // lastDir=0, streak=0; el primer tick lleva la racha a 1

		ev := wheelEvent(helpers.WHEEL_DOWN, false)
		ret := mouseWheelCatcherHook(0, uintptr(helpers.WHEEL_EVENT), unsafe.Pointer(&ev))

		if ret != helpers.BLOCK {
			t.Errorf("ret = %d, quiero BLOCK (%d)", ret, helpers.BLOCK)
		}
		if len(injectorCh) != 0 {
			t.Errorf("encoló %d sintéticos en silencio, quiero 0", len(injectorCh))
		}
	})

	t.Run("racha en compensación: bloquea y encola un sintético", func(t *testing.T) {
		resetDetectionState()
		lastDir.Store(helpers.WHEEL_DOWN)
		streakCount.Store(helpers.SILENCE_TICKS) // 3; el tick lleva la racha a 4

		ev := wheelEvent(helpers.WHEEL_DOWN, false)
		ret := mouseWheelCatcherHook(0, uintptr(helpers.WHEEL_EVENT), unsafe.Pointer(&ev))

		if ret != helpers.BLOCK {
			t.Errorf("ret = %d, quiero BLOCK", ret)
		}
		if len(injectorCh) != 1 {
			t.Errorf("encoló %d sintéticos, quiero 1", len(injectorCh))
		}
	})

	t.Run("racha confirmada: deja pasar", func(t *testing.T) {
		resetDetectionState()
		lastDir.Store(helpers.WHEEL_DOWN)
		streakCount.Store(helpers.TRUST_TICKS) // 7; el tick lleva la racha a 8
		var passed bool
		passThrough = func(_, _, _ uintptr) uintptr { passed = true; return 42 }

		ev := wheelEvent(helpers.WHEEL_DOWN, false)
		ret := mouseWheelCatcherHook(0, uintptr(helpers.WHEEL_EVENT), unsafe.Pointer(&ev))

		if !passed {
			t.Error("no llamó a passThrough")
		}
		if ret != 42 {
			t.Errorf("ret = %d, quiero 42 (lo que devolvió passThrough)", ret)
		}
	})

	// Secuencia real por el hook: acumula la racha tick a tick. Un updateStreak de más (racha +2 por tick) rompe estos números.
	t.Run("secuencia real: silencio, compensación, pase", func(t *testing.T) {
		resetDetectionState()
		passThrough = func(_, _, _ uintptr) uintptr { return 0 }
		ev := wheelEvent(helpers.WHEEL_DOWN, false)
		fire := func() uintptr {
			return mouseWheelCatcherHook(0, uintptr(helpers.WHEEL_EVENT), unsafe.Pointer(&ev))
		}

		for i := int32(1); i <= helpers.SILENCE_TICKS; i++ {
			fire()
		}
		if len(injectorCh) != 0 {
			t.Fatalf("tras %d ticks de silencio, encoló %d, quiero 0", helpers.SILENCE_TICKS, len(injectorCh))
		}

		for i := helpers.SILENCE_TICKS + 1; i <= helpers.TRUST_TICKS; i++ {
			fire()
		}
		want := int(helpers.TRUST_TICKS - helpers.SILENCE_TICKS)
		if len(injectorCh) != want {
			t.Fatalf("tras la compensación, encoló %d, quiero %d", len(injectorCh), want)
		}

		if fire() == helpers.BLOCK {
			t.Fatalf("el tick %d debería pasar, no BLOCK", helpers.TRUST_TICKS+1)
		}
	})
}
