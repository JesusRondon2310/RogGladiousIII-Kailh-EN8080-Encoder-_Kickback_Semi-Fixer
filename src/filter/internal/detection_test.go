package filter

// Tests de detection: layout del struct, decodificación de dirección, racha. El flujo del hook proc va en detection_hook_test.go.
// StartHook / StopHook necesitan un escritorio real y un message loop — van en integración.

import (
	"testing"
	"unsafe"

	"Kickback_Fix/src/helpers"
)

// resetDetectionState: racha en cero y canal drenado. Compartido con detection_hook_test.go.
func resetDetectionState() {
	lastDir.Store(0)
	streakCount.Store(0)
	resetInjectorState()
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

	t.Run("cambio de dirección reinicia la racha", func(t *testing.T) {
		resetDetectionState()
		updateStreak(helpers.WHEEL_UP)
		updateStreak(helpers.WHEEL_UP)

		if got := updateStreak(helpers.WHEEL_DOWN); got != 1 {
			t.Fatalf("tras el cambio: streak = %d, quiero 1", got)
		}
		if got := lastDir.Load(); got != helpers.WHEEL_DOWN {
			t.Fatalf("lastDir = %d, quiero %d", got, helpers.WHEEL_DOWN)
		}
	})

	t.Run("capea en TRUST_TICKS+1 y no sube más", func(t *testing.T) {
		resetDetectionState()
		var last int32
		for i := 0; i < 20; i++ {
			last = updateStreak(helpers.WHEEL_UP)
		}
		if last != helpers.TRUST_TICKS+1 {
			t.Fatalf("tras 20 ticks: streak = %d, quiero %d (capeado)", last, helpers.TRUST_TICKS+1)
		}
	})
}

// TestDecide: la spec de decide(racha) -> acción.
//
//	racha 1..SILENCE_TICKS (3)          -> block          (silencio inicial, sin inyectar)
//	racha SILENCE_TICKS+1..TRUST_TICKS  -> blockAndInject  (compensación simultánea)
//	racha > TRUST_TICKS (7)             -> pass            (dirección confirmada)
func TestDecide(t *testing.T) {
	cases := []struct {
		streak int32
		want   action
	}{
		{1, block},
		{2, block},
		{3, block},
		{4, blockAndInject},
		{5, blockAndInject},
		{6, blockAndInject},
		{7, blockAndInject},
		{8, pass},
		{20, pass},
	}

	for _, c := range cases {
		if got := decide(c.streak); got != c.want {
			t.Errorf("decide(%d) = %v, quiero %v", c.streak, got, c.want)
		}
	}
}

// Los límites salen de las constantes, no de los literales de arriba: si mañana SILENCE_TICKS o TRUST_TICKS cambian, decide
// tiene que seguir bien.
func TestDecideRespetaLasConstantes(t *testing.T) {
	if decide(helpers.SILENCE_TICKS) != block {
		t.Errorf("racha == SILENCE_TICKS debe ser block")
	}
	if decide(helpers.SILENCE_TICKS+1) != blockAndInject {
		t.Errorf("racha == SILENCE_TICKS+1 debe ser blockAndInject")
	}
	if decide(helpers.TRUST_TICKS) != blockAndInject {
		t.Errorf("racha == TRUST_TICKS debe ser blockAndInject (es el 7º bloqueado)")
	}
	if decide(helpers.TRUST_TICKS+1) != pass {
		t.Errorf("racha == TRUST_TICKS+1 debe ser pass")
	}
}
