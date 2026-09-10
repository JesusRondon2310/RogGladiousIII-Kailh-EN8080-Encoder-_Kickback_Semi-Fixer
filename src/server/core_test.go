package server

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

// El enrutado: que los patrones "GET /config" / "PUT /config" lleguen a sus
// handlers. mux.ServeHTTP despacha por patrón sin abrir un puerto real.
func TestRoutes(t *testing.T) {
	resetConfig()
	defer resetConfig()

	mux := routes()

	get := httptest.NewRequest(http.MethodGet, "/config", nil)
	getRec := httptest.NewRecorder()
	mux.ServeHTTP(getRec, get)
	if getRec.Code != http.StatusOK {
		t.Errorf("GET /config no enruta: código %d", getRec.Code)
	}

	put := httptest.NewRequest(http.MethodPut, "/config", strings.NewReader(`{"silence":4,"trust":8,"enabled":true}`))
	putRec := httptest.NewRecorder()
	mux.ServeHTTP(putRec, put)
	if putRec.Code != http.StatusNoContent {
		t.Errorf("PUT /config no enruta: código %d", putRec.Code)
	}
}
