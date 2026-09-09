package filter

import "Kickback_Fix/src/helpers"

// action es qué hacer con un tick físico que llegó al hook.
type action int

const (
	pass           action = iota // dejarlo pasar a la aplicación
	block                        // bloquearlo, sin más
	blockAndInject               // bloquearlo y encolar un tick sintético en su dirección
)

// decide devuelve la acción para un tick, según la racha acumulada en su dirección (lo que devuelve updateStreak).
//
// Reglas (ver Redesing.md y el trace acordado):
//   - racha dentro del silencio inicial      -> block
//   - racha en la ventana de compensación    -> blockAndInject
//   - racha ya confirmada como dirección real -> pass
//
// Los límites son helpers.SILENCE_TICKS y helpers.TRUST_TICKS.
func decide(streak int32) action {
	if streak <= helpers.SILENCE_TICKS {
		return block
	}

	if streak <= helpers.TRUST_TICKS {
		return blockAndInject
	}

	return pass
}
