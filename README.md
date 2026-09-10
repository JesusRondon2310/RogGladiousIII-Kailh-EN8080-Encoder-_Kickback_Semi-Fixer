# Kickback Fix — ROG Gladius III (encoder Kailh EN8080)

![ASUS ROG Gladius III](https://m.media-amazon.com/images/I/51MWi-ZraSL.jpg)

Filtro de software en **Go** para el problema conocido de **"wheel kickback"**
del [ASUS ROG Gladius III](https://www.amazon.es/ASUS-ROG-Gladius-III-intercambiables/dp/B096XKJK1V)
(y otros ratones que usan el mismo encoder Kailh EN8080): al girar la rueda, el
encoder a veces genera un tick fantasma en dirección contraria, causando que la
página suba cuando en realidad scrolleaste hacia abajo (o viceversa).

Es un defecto de hardware reportado por múltiples usuarios en los foros
oficiales de ASUS ROG, no un caso aislado. Este proyecto es una **mitigación
por software**.

## Estado del proyecto

**Alpha 0.2 — terminada.** La lógica de detección y compensación (v2) está
implementada y probada. Todo el código es Go; la reescritura desde Rust se hizo
como ejercicio de aprendizaje.

- **No es un ejecutable distribuible.** No hay instalador, interfaz gráfica,
  ícono de bandeja ni autostart — eso es el roadmap.
- Se compila desde el código fuente y se corre en una terminal:
  `go run .` — **sin permisos de administrador**.
- Al arrancar levanta una API HTTP local en `127.0.0.1:47800` para ajustar
  `SILENCE_TICKS` / `TRUST_TICKS` en tiempo real (ver Configuración). La GUI de
  escritorio en Java que la consumirá es el próximo paso del roadmap; por ahora
  se maneja con `curl`.

## Cómo funciona (v2)

Se instala un hook de bajo nivel de mouse (`WH_MOUSE_LL`) que intercepta cada
evento de rueda antes de que llegue a cualquier aplicación.

El filtro cuenta la **racha**: ticks consecutivos en la misma dirección
(comparando con el tick anterior — ninguna dirección tiene pase libre). Un tick
en otra dirección reinicia la racha a 1. Según la racha:

| Racha                                  | Acción                                                              |
| -------------------------------------- | ------------------------------------------------------------------- |
| `1 .. SILENCE_TICKS` (3)               | bloquea el tick, en silencio                                        |
| `SILENCE_TICKS+1 .. TRUST_TICKS` (4-7) | bloquea el tick físico **e inyecta uno sintético** en esa dirección |
| `> TRUST_TICKS` (8+)                   | deja pasar — la dirección se da por confirmada                      |

El bloqueo inicial evita que una racha corta de kickback llegue a pantalla. En
cuanto la racha supera el silencio, se **compensa en tiempo real**: por cada
tick físico que se retiene, se inyecta uno sintético, así el usuario ve
movimiento mientras la racha termina de confirmarse. Si la racha se corta antes
de `TRUST_TICKS`, se descarta y empieza de nuevo.

La inyección (`SendInput`) corre en una goroutine aparte, comunicada por un
canal — nunca dentro del callback del hook, porque eso bloquea el raw input
thread del sistema.

## Validación

El kickback del encoder del autor **se resolvió a nivel de hardware**
(actualizaciones de firmware vía Armoury Crate, limpieza, y swap físico de los
switches principales) antes de poder validar v2 contra kickback real. El filtro
compila, pasa los tests y corre exactamente como lo describe el diseño, pero ya
no hay una señal de kickback contra la cual medir su efecto en uso real.

**Limitación conceptual:** si el encoder generara una racha fantasma más larga
que `TRUST_TICKS`, esa racha se confirmaría como un cambio de dirección real. No
existe un valor que cubra toda racha posible sin volver el filtro lento en
cambios de dirección legítimos — es un trade-off entre precisión y
responsividad. Ver `projectInformation/` para el detalle.

## Requisitos

- Windows (usa la API Win32 vía `syscall` + `golang.org/x/sys/windows`)
- [Go](https://go.dev) 1.24+

Sin CGo, sin compilador de C, sin privilegios de administrador.

## Compilar y ejecutar

```powershell
go run .
```

o, para un binario:

```powershell
go build -ldflags="-s -w" -trimpath -o Kickback_Fix.exe .
```

Ctrl+C para salir (desengancha el hook limpiamente). Para ver el trace de fases:

```powershell
$env:KICKBACK_DEBUG=1; go run .
```

## Configuración

Dos valores, ajustables en caliente por la API HTTP local (`127.0.0.1:47800`)
mientras el filtro corre — sin recompilar ni reiniciar:

- `SILENCE_TICKS` — ticks bloqueados en silencio antes de arrancar la
  compensación. Por defecto `3`, rango **2-5**. Bájalo si sientes el filtro
  lento al cambiar de dirección a propósito.
- `TRUST_TICKS` — racha total a la que la dirección se da por confirmada; el
  tick siguiente ya pasa. Por defecto `7`, rango **6-10**. Súbelo si el
  kickback se sigue colando.

```powershell
# ver la config actual (curl.exe, no el alias `curl` de PowerShell)
curl.exe 127.0.0.1:47800/config

# cambiarla (valida rango; 400 si no cuadra)
curl.exe -X PUT 127.0.0.1:47800/config -d '{\"silence\":4,\"trust\":9}'
```

Los valores por defecto viven en `src/helpers/constants.go`; los límites de los
sliders, en `src/server/config_api.go`.

## Roadmap

- [ ] `SILENCE_TICKS` / `TRUST_TICKS` ajustables en tiempo real — server
      `net/http` local en el núcleo Go (`127.0.0.1`), sin recompilar ni reiniciar
- [ ] GUI de escritorio en **Java** (`gui/`, proceso aparte) que habla con el
      núcleo por HTTP local — sliders de configuración, estado, toggle
- [ ] Hotkey global + botón en la GUI para activar/desactivar el filtro
- [ ] Toggle de autostart con Windows (registro `HKCU\...\Run`)
- [ ] Ícono de bandeja con indicador direccional y color configurable por bloqueo
- [ ] Port a Linux (`evdev`)

## Créditos y contexto

Encoder identificado como Kailh EN8080 según el desmontaje técnico de
[TechPowerUp](https://www.techpowerup.com/review/asus-rog-gladius-iii/4.html).

El defecto de kickback está reportado en múltiples hilos del
[foro oficial de ASUS ROG](https://rog-forum.asus.com/).
