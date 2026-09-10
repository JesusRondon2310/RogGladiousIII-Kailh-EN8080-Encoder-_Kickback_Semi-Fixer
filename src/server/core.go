package server

import "fmt"

// Puerto local fijo donde la GUI encuentra la API de config. Solo 127.0.0.1:
// nada de esto sale de la máquina.
const addr = "127.0.0.1:47800"

// 1. Arranca la API HTTP local en una goroutine y vuelve de inmediato. serve bloquea (ListenAndServe), por eso corre aparte — igual que el inyector. Si el puerto está ocupado serve devuelve el error y se imprime; el filtro sigue funcionando, solo que sin GUI.
func StartServer() {
	go func() {
		if err := serve(addr); err != nil {
			fmt.Println("API de config no disponible:", err)
		}
	}()
}
