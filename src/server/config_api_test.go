package server

import (
	"Kickback_Fix/src/config"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

// resetConfig deja config en los valores por defecto (3, 7). Se llama antes y
// después de cada test que los cambia — los tests comparten el estado global.
func resetConfig() {
	config.SetSilence(3)
	config.SetTrust(7)
}

// validateConfig como unidad. El caso silence >= trust no se prueba: con
// silence en [2,5] y trust en [7,10] es inalcanzable (rangos sin solapamiento).
func TestValidateConfig(t *testing.T) {
	casos := []struct {
		nombre  string
		in      configDTO
		quiereN bool // true = espera error
	}{
		{"válido mínimo", configDTO{Silence: 2, Trust: 6}, false},
		{"válido máximo", configDTO{Silence: 5, Trust: 10}, false},
		{"silence bajo mínimo", configDTO{Silence: 1, Trust: 7}, true},
		{"silence sobre máximo", configDTO{Silence: 6, Trust: 7}, true},
		{"trust bajo mínimo", configDTO{Silence: 3, Trust: 5}, true},
		{"trust sobre máximo", configDTO{Silence: 3, Trust: 11}, true},
	}
	for _, c := range casos {
		t.Run(c.nombre, func(t *testing.T) {
			err := validateConfig(c.in)
			if (err != nil) != c.quiereN {
				t.Errorf("validateConfig(%+v) err = %v, quería error=%v", c.in, err, c.quiereN)
			}
		})
	}
}

// GET /config devuelve el estado actual como JSON.
func TestHandleGetConfig(t *testing.T) {
	resetConfig()

	req := httptest.NewRequest(http.MethodGet, "/config", nil)
	rec := httptest.NewRecorder()
	handleGetConfig(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("código = %d, quería 200", rec.Code)
	}
	if ct := rec.Header().Get("Content-Type"); ct != "application/json" {
		t.Errorf("Content-Type = %q, quería application/json", ct)
	}
	var got configDTO
	if err := json.NewDecoder(rec.Body).Decode(&got); err != nil {
		t.Fatalf("cuerpo no es JSON válido: %v", err)
	}
	if got.Silence != 3 || got.Trust != 7 {
		t.Errorf("dto = %+v, quería {Silence:3 Trust:7}", got)
	}
}

// PUT /config válido: aplica el par y responde 204.
func TestHandlePutConfigValido(t *testing.T) {
	resetConfig()
	defer resetConfig()

	req := httptest.NewRequest(http.MethodPut, "/config", strings.NewReader(`{"silence":2,"trust":9}`))
	rec := httptest.NewRecorder()
	handlePutConfig(rec, req)

	if rec.Code != http.StatusNoContent {
		t.Fatalf("código = %d, quería 204. cuerpo: %s", rec.Code, rec.Body)
	}
	if config.Silence() != 2 || config.Trust() != 9 {
		t.Errorf("config = (%d,%d), quería (2,9)", config.Silence(), config.Trust())
	}
}

// PUT /config con cuerpo inválido: 400 y config sin tocar.
func TestHandlePutConfigRechazos(t *testing.T) {
	casos := []struct {
		nombre string
		body   string
	}{
		{"silence fuera de rango", `{"silence":6,"trust":9}`},
		{"trust fuera de rango", `{"silence":3,"trust":11}`},
		{"JSON roto", `{`},
	}
	for _, c := range casos {
		t.Run(c.nombre, func(t *testing.T) {
			resetConfig()
			defer resetConfig()

			req := httptest.NewRequest(http.MethodPut, "/config", strings.NewReader(c.body))
			rec := httptest.NewRecorder()
			handlePutConfig(rec, req)

			if rec.Code != http.StatusBadRequest {
				t.Errorf("código = %d, quería 400", rec.Code)
			}
			if config.Silence() != 3 || config.Trust() != 7 {
				t.Errorf("config cambió a (%d,%d) tras un rechazo", config.Silence(), config.Trust())
			}
		})
	}
}
