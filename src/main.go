package main

import (
	"os"

	"Kickback_Fix/src/filter"
)

func main() {
	var err error = filter.Run()
	if err != nil {
		os.Exit(1)
	}
}
