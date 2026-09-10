// Package exception colapsa el manejo de errores repetitivo con panic/recover, contenido.
//
// Uso: la función declara named return y pone `defer exception.Catch(&err)` como primera línea.
// Dentro, exception.Try / exception.TryWithErrorReturnFunc abortan al primer error sin escribir `if err != nil`.
// Un panic que NO venga de exception (nil deref, índice fuera de rango) se re-lanza intacto.
//
//	func Load(path string) (cfg Config, err error) {
//		defer exception.Catch(&err)
//		raw := exception.Try(os.ReadFile(path))
//		exception.TryWithErrorReturnFunc(json.Unmarshal(raw, &cfg))
//		return
//	}
//
// No es Win32 ni específico de este proyecto: sirve en cualquier módulo Go.
package exception

type failure struct{ err error }

// Catch va como defer al tope de una función con named return. Convierte el panic de Try / TryWithErrorReturnFunc en *dst.
func Catch(dst *error) {
	if p := recover(); p != nil {
		if f, ours := p.(failure); ours {
			*dst = f.err
			return
		}
		panic(p)
	}
}

// Try: si err != nil, aborta la función (hasta el Catch); si no, devuelve v. Para cualquier (T, error).
func Try[T any](v T, err error) T {
	if err != nil {
		panic(failure{err})
	}
	return v
}

// TryWithErrorReturnFunc: variante para funciones que solo devuelven error.
func TryWithErrorReturnFunc(err error) {
	if err != nil {
		panic(failure{err})
	}
}
