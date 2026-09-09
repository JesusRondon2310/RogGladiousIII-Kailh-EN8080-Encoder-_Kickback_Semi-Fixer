package exception

import (
	"errors"
	"testing"
)

func TestTryDevuelveElValorSinError(t *testing.T) {
	if got := Try(42, nil); got != 42 {
		t.Errorf("Try(42, nil) = %d, quiero 42", got)
	}
}

func TestCatchCapturaElError(t *testing.T) {
	boom := errors.New("boom")
	err := under(func() { TryWithErrorReturnFunc(boom) })
	if !errors.Is(err, boom) {
		t.Errorf("err = %v, quiero boom", err)
	}
}

func TestCatchNoTocaElErrorSinPanic(t *testing.T) {
	if err := under(func() {}); err != nil {
		t.Errorf("err = %v, quiero nil", err)
	}
}

func TestCatchRelanzaPanicAjeno(t *testing.T) {
	defer func() {
		if r := recover(); r != "otro" {
			t.Errorf("recuperé %v, quiero que se re-lance \"otro\"", r)
		}
	}()
	under(func() { panic("otro") })
}

// under ejecuta f bajo Catch y devuelve el error resultante.
func under(f func()) (err error) {
	defer Catch(&err)
	f()
	return
}
