package filter

import (
	"testing"

	"Kickback_Fix/src/helpers"
)

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
