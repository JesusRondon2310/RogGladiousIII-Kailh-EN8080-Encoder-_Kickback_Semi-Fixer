package server

import (
	"Kickback_Fix/src/config"
	"encoding/json"
	"fmt"
	"net/http"
)

// Cuerpo JSON de GET/PUT /config.
type configDTO struct {
	Silence int32 `json:"silence"`
	Trust   int32 `json:"trust"`
	Enabled bool  `json:"enabled"`
}

// Cuanto dura el Silencio
const MIN_SILENCE_TICKS int32 = 2
const MAX_SILENCE_TICKS int32 = 5

// Cuanto dura la inyeccion + bloqueo
const MIN_TRUST_TICKS int32 = 6
const MAX_TRUST_TICKS int32 = 10

func validateConfig(c configDTO) error {
	if c.Silence < MIN_SILENCE_TICKS || c.Silence > MAX_SILENCE_TICKS {
		return fmt.Errorf("silence fuera de rango [%d,%d]: %d", MIN_SILENCE_TICKS, MAX_SILENCE_TICKS, c.Silence)
	}
	if c.Trust < MIN_TRUST_TICKS || c.Trust > MAX_TRUST_TICKS {
		return fmt.Errorf("trust fuera de rango [%d,%d]: %d", MIN_TRUST_TICKS, MAX_TRUST_TICKS, c.Trust)
	}
	if c.Silence >= c.Trust {
		return fmt.Errorf("silence (%d) debe ser menor que trust (%d)", c.Silence, c.Trust)
	}
	return nil
}

func handleGetConfig(w http.ResponseWriter, r *http.Request) {
	dto := configDTO{Silence: config.Silence(), Trust: config.Trust(), Enabled: config.Enabled()}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(dto)
}

func handlePutConfig(w http.ResponseWriter, r *http.Request) {
	var dto configDTO
	if err := json.NewDecoder(r.Body).Decode(&dto); err != nil {
		http.Error(w, "JSON inválido: "+err.Error(), http.StatusBadRequest)
		return
	}
	if err := validateConfig(dto); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	config.SetSilence(dto.Silence)
	config.SetTrust(dto.Trust)
	config.SetEnabled(dto.Enabled)
	w.WriteHeader(http.StatusNoContent)
}

// routes registra los handlers y devuelve el mux. Separado de serve para poder
// probar el enrutado con httptest.NewServer (sin abrir un puerto real).
func routes() *http.ServeMux {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /config", handleGetConfig)
	mux.HandleFunc("PUT /config", handlePutConfig)
	return mux
}

func serve(addr string) error {
	return http.ListenAndServe(addr, routes())
}
