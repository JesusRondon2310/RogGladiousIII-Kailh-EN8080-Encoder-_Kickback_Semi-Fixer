// Estado de tuning del filtro, ajustable en runtime desde la GUI.
//
// Los valores por defecto siguen siendo SILENCE_TICKS / TRUST_TICKS en
// helpers/constants.go; aquí viven las copias vivas como atomic.Int32 porque
// el hook las lee en cada tick (Silence/Trust) mientras el servidor HTTP las
// escribe (SetSilence/SetTrust) — igual que lastDir/streakCount en detection.
//
// Como helpers/, este paquete no necesita orquestador: un solo archivo.
package config

import (
	"Kickback_Fix/src/helpers"
	"sync/atomic"
)

var silence atomic.Int32
var trust atomic.Int32
var enabled atomic.Bool

// init deja el estado en los valores por defecto de helpers, con el filtro
// encendido. Go lo llama solo al cargar el paquete.
func init() {
	silence.Store(helpers.SILENCE_TICKS)
	trust.Store(helpers.TRUST_TICKS)
	enabled.Store(true)
}

// Lecturas — las usa el hook en el hot path.
func Silence() int32 { return silence.Load() }
func Trust() int32   { return trust.Load() }
func Enabled() bool  { return enabled.Load() }

// Escrituras — las usa el servidor HTTP. La validación (rango, silence < trust)
// va en la capa del servidor, donde llega la petición.
func SetSilence(n int32) { silence.Store(n) }
func SetTrust(n int32)   { trust.Store(n) }
func SetEnabled(b bool)  { enabled.Store(b) }
