package filter

// Run no es testeable unitariamente: instala un hook real y corre el bombeo de mensajes. Acá: layout de MSG y el ctrl handler
// con requestQuit sustituido.

import (
	"testing"
	"unsafe"

	"Kickback_Fix/src/helpers"
)

func TestMsgLayout(t *testing.T) {
	if got := unsafe.Sizeof(msg{}); got != 48 {
		t.Errorf("msg = %d bytes, quiero 48", got)
	}
}

func TestConsoleCtrlHandler(t *testing.T) {
	orig := requestQuit
	t.Cleanup(func() { requestQuit = orig })

	t.Run("Ctrl+C: maneja y pide salir", func(t *testing.T) {
		var asked bool
		requestQuit = func() { asked = true }

		if got := consoleCtrlHandler(uintptr(helpers.CTRL_C_EVENT)); got != uintptr(helpers.HANDLED) {
			t.Errorf("ret = %d, quiero HANDLED (%d)", got, helpers.HANDLED)
		}
		if !asked {
			t.Error("no llamó a requestQuit")
		}
	})

	t.Run("evento desconocido: no maneja", func(t *testing.T) {
		var asked bool
		requestQuit = func() { asked = true }

		if got := consoleCtrlHandler(999); got != uintptr(helpers.NOT_HANDLED) {
			t.Errorf("ret = %d, quiero NOT_HANDLED (%d)", got, helpers.NOT_HANDLED)
		}
		if asked {
			t.Error("llamó a requestQuit para un evento que no maneja")
		}
	})
}
