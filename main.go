package main

import (
	"os"

	"Kickback_Fix/src/filter"
	"Kickback_Fix/src/server"
)

func main() {
	// El servidor arranca primero (no bloquea, corre en su goroutine); luego
	// filter.Run bombea mensajes y se queda hasta el cierre.
	server.StartServer()

	var err error = filter.Run()
	if err != nil {
		os.Exit(1)
	}
}
