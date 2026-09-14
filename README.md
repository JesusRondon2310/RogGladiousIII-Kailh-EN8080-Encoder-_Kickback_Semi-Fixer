# Kickback Semi Fixer — ROG Gladius III (encoder Kailh EN8080)

![ASUS ROG Gladius III](https://m.media-amazon.com/images/I/51MWi-ZraSL.jpg)

Filtro de software en **Rust** para el problema conocido de **"wheel kickback"**
del [ASUS ROG Gladius III](https://www.amazon.es/ASUS-ROG-Gladius-III-intercambiables/dp/B096XKJK1V)
(y otros ratones que usan el mismo encoder Kailh EN8080): al girar la rueda, el
encoder a veces genera un tick fantasma en dirección contraria, causando que la
página suba cuando en realidad scrolleaste hacia abajo (o viceversa).

Es un defecto de hardware reportado por múltiples usuarios en los foros
oficiales de ASUS ROG, no un caso aislado. Este proyecto es una **mitigación
por software**.

## Estado del proyecto

**Terminado.** La lógica de detección y compensación está implementada,
probada, y el proyecto se considera completo tal como está.

- Se compila a un único ejecutable independiente — no necesita el toolchain
  de Rust instalado en la máquina donde se use, ni ninguna otra dependencia
  externa.
- Corre con una ventana de consola visible (se puede minimizar) — a propósito,
  como recordatorio permanente de que sigue activo. Ver la sección de
  seguridad más abajo para el porqué.
- Se configura y se ajusta en tiempo real editando `config.toml` — sin
  recompilar, sin reiniciar el programa.
- No requiere permisos de administrador en ningún momento.

## Cómo funciona

Se instala un hook de bajo nivel de mouse (`WH_MOUSE_LL`) que intercepta cada
evento de rueda antes de que llegue a cualquier aplicación.

El filtro cuenta la **racha**: ticks consecutivos en la misma dirección
(comparando con el tick anterior — ninguna dirección tiene pase libre). Un
tick en otra dirección reinicia la racha a 1. Según la racha:

| Racha                                            | Acción                                                              |
| ------------------------------------------------- | -------------------------------------------------------------------- |
| `1 .. SilencioInicial`                            | bloquea el tick, en silencio                                        |
| `SilencioInicial+1 .. TechoKickback`              | bloquea el tick físico **e inyecta uno sintético** en esa dirección |
| `> TechoKickback`                                 | deja pasar — la dirección se da por confirmada                      |

El bloqueo inicial evita que una racha corta de kickback llegue a pantalla.
En cuanto la racha supera el silencio, se **compensa en tiempo real**: por
cada tick físico que se retiene, se inyecta uno sintético, así el usuario ve
movimiento mientras la racha termina de confirmarse. Si la racha se corta
antes de `TechoKickback`, se descarta y empieza de nuevo.

La inyección (`SendInput`) corre en un hilo aparte, comunicado por un canal
(`mpsc`) — nunca dentro del callback del hook, porque eso bloquea el raw
input thread del sistema.

## Configuración

Al ejecutarlo por primera vez, el programa crea un archivo `config.toml`
junto al ejecutable, con tres valores comentados:

```toml
Filtro = false                 # true = filtro activado, false = lo apaga
SilencioInicial = 3            # ticks bloqueados en silencio antes de empezar a compensar
TechoKickback = 7               # racha total a la que la dirección se da por confirmada
```

- **`Filtro`**: por defecto viene en `false`. Al descomprimir o instalar el
  programa, no empieza a interceptar el mouse por su cuenta — hay que
  activarlo a propósito poniendo `true` y guardando el archivo (o volviendo
  a ejecutar el programa con el archivo ya en `true`).
- **`SilencioInicial`** y **`TechoKickback`**: ajustan qué tan agresivo es el
  filtro. Súbelos si el kickback se sigue colando; bájalos si sientes el
  filtro lento al cambiar de dirección a propósito. El mínimo válido para
  ambos es `0` — no hay techo superior. Con `TechoKickback = 0`, el filtro
  deja de inyectar ticks sintéticos por completo (solo mantiene el silencio
  inicial).

Mientras el programa está corriendo, el archivo se revisa cada segundo: si
lo editas y guardas, el cambio se aplica de inmediato, sin reiniciar nada.
Si cambias `Filtro` a `false` con el programa activo, se apaga solo y de
forma limpia (desinstala el hook antes de terminar).

## Uso

1. Ejecuta `Kickback_Fix.exe`. La primera vez, esto abre la consola, crea
   `config.toml` con el filtro desactivado, y se cierra solo — no queda
   nada corriendo.
2. Edita `config.toml`, pon `Filtro = true`, guarda.
3. Vuelve a ejecutar `Kickback_Fix.exe`. Esta vez queda corriendo, con su
   ventana de consola abierta — puedes minimizarla, pero déjala ahí: es tu
   recordatorio de que el filtro sigue activo.
4. Para apagarlo: cierra esa ventana de consola (se apaga limpio, solo),
   o cambia `Filtro` a `false` en `config.toml` y guarda (se cierra solo en
   el siguiente segundo, aunque hayas minimizado la ventana).

La consola normalmente solo muestra un par de mensajes fijos al arrancar y
al cerrar — no imprime nada por cada tick de la rueda. Si quieres ver el
detalle de cada decisión del filtro en vivo (útil solo para depurar), corre
el ejecutable con la variable de entorno `KICKBACK_DEBUG` activada:

```powershell
$env:KICKBACK_DEBUG=1; .\Kickback_Fix.exe
```

## Seguridad: por qué la consola queda visible a propósito

El riesgo real de esta clase de herramienta (cualquier programa sin firmar
que instala un hook de bajo nivel de input) es dejarlo corriendo sin
acordarse y entrar a un juego con anticheat activo. A diferencia de software
de fabricantes reconocidos (Razer Synapse, Logitech G HUB), que está firmado
y en listas blancas negociadas directamente con los proveedores de
anticheat, este programa no tiene ni puede tener ese mismo nivel de
confianza institucional — así que el riesgo de que algún anticheat lo marque
no se puede descartar del todo, sea cual sea el juego.

Por eso la consola se deja **visible a propósito**, en vez de esconderla:
es la forma más simple de tener un recordatorio constante de "esto sigue
activo" sin construir una interfaz completa. No elimina el riesgo, pero lo
reduce — la recomendación real sigue siendo apagarlo (`Filtro = false`, o
cerrar la ventana) antes de abrir cualquier juego con anticheat, cada vez,
sin excepción.

## Por qué no tiene GUI, hotkey ni arranque automático con Windows

Estas tres cosas se consideraron y se descartaron a propósito, no por falta
de tiempo:

- **Interfaz gráfica**: el archivo `config.toml` cubre exactamente lo mismo
  que una GUI habría cubierto (dos números y un interruptor), sin necesitar
  un servidor local, un frontend, ni mantener ese código a futuro. Construir
  una interfaz para tres valores no se justificaba.
- **Hotkey para activar/desactivar**: para que un atajo de teclado apague el
  filtro, alguien tiene que estar corriendo para escucharlo — con lo cual el
  atajo solo puede servir mientras el programa ya está activo, nunca para
  reactivarlo estando apagado. Y editar una línea de `config.toml` a mano
  toma prácticamente el mismo tiempo que presionar un atajo, así que el
  ahorro real no compensaba el código y la configuración extra que hubiera
  hecho falta.
- **Arranque automático con Windows**: se descartó por una razón concreta,
  no de comodidad. Este programa instala un hook de bajo nivel sobre el
  input del mouse; esa combinación (persistencia en el arranque de Windows +
  interceptar input a bajo nivel) es exactamente el patrón que muchos
  antivirus y sistemas anticheat (Vanguard de Valorant es un ejemplo
  particularmente estricto) marcan como sospechoso, incluso siendo
  inofensivo. El riesgo real es que alguien lo deje corriendo sin acordarse,
  entre a un juego con anticheat activo, y termine sancionado por algo que
  ni siquiera recordaba tener encendido. Por eso el programa se activa
  siempre a propósito, cada vez, y nunca por su cuenta. Si aun así alguien
  quiere que arranque solo, puede configurarlo por su cuenta con el
  Programador de Tareas de Windows — eso queda fuera del programa mismo.

## Validación

El kickback del encoder del autor **se resolvió a nivel de hardware**
(actualizaciones de firmware vía Armoury Crate, limpieza, y swap físico de
los switches principales) antes de poder validar el filtro contra kickback
real en producción. El filtro compila, corre exactamente como lo describe
el diseño, y se probó en vivo con inyección de ticks sintéticos
indistinguibles de ticks físicos reales — pero no hay señal de kickback real
contra la cual medir su efecto final.

**Limitación conceptual:** si el encoder generara una racha fantasma más
larga que `TechoKickback`, esa racha se confirmaría como un cambio de
dirección real. No existe un valor que cubra toda racha posible sin volver
el filtro lento en cambios de dirección legítimos — es un trade-off entre
precisión y responsividad. Ver `projectInformation/` para el detalle.

## Requisitos

- Windows (usa la API Win32 vía la crate `windows-sys`, bindings crudos sin
  wrappers ni runtime propio)
- [Rust](https://www.rust-lang.org) vía `rustup` — solo para
  compilar desde el código fuente. El ejecutable ya compilado no necesita
  nada de esto instalado.

Sin dependencias de C, sin privilegios de administrador.

## Compilar

```powershell
cargo build --release
```

El binario queda en `target\release\Kickback_Fix.exe` — se puede copiar y
correr en cualquier máquina Windows sin instalar nada más.

## Créditos y contexto

Encoder identificado como Kailh EN8080 según el desmontaje técnico de
[TechPowerUp](https://www.techpowerup.com/review/asus-rog-gladius-iii/4.html).

El defecto de kickback está reportado en múltiples hilos del
[foro oficial de ASUS ROG](https://rog-forum.asus.com/).
