# Kickback Fix — ROG Gladius III (encoder Kailh EN8080)

Filtro de software en Rust para el problema conocido de **"wheel kickback"**
del [ASUS ROG Gladius III](https://www.amazon.es/ASUS-ROG-Gladius-III-intercambiables/dp/B096XKJK1V)
(y otros ratones que usan el mismo encoder Kailh EN8080): al girar la
rueda, el encoder a veces genera un tick fantasma en dirección contraria,
causando que la página suba cuando en realidad scrolleaste hacia abajo
(o viceversa).

Este defecto es un problema de hardware conocido y reportado por múltiples
usuarios en los foros oficiales de ASUS ROG, no un caso aislado. Este
proyecto es una **mitigación por software**, no una solución de hardware —
no elimina el kickback al 100%, lo reduce significativamente.

## Estado del proyecto

**Alpha 0.1** — Funcional para uso diario. La lógica central de detección
está probada extensivamente contra el hardware real del autor. Todavía no
tiene interfaz gráfica: se configura editando constantes en el código y
recompilando.

## Cómo funciona

En vez de un debounce simple (bloquear cualquier reversión de dirección
que llegue "muy rápido"), este proyecto usa un esquema de **dirección
confirmada + candidato pendiente**:

1. Se mantiene una dirección "confirmada" activa (la que se considera real
   en este momento).
2. Un tick en dirección contraria no se dejar pasar de inmediato — se
   guarda como candidato pendiente.
3. El candidato solo se confirma (y pasa a ser la nueva dirección
   confirmada) si llegan **N ticks consecutivos** en esa misma dirección
   nueva (`REQUIRED_CONFIRMATIONS`, ajustable en el código).
4. El candidato se reinicia únicamente cuando llega un tick real que
   coincide con la dirección ya confirmada — nunca por el simple paso del
   tiempo.

Esto se instala como un hook de bajo nivel de mouse (`WH_MOUSE_LL`) a
nivel de sistema operativo Windows, interceptando cada evento de rueda
antes de que llegue a cualquier aplicación.

## Limitación conocida

Si el encoder genera una racha de ticks fantasma consecutivos **más larga**
que `REQUIRED_CONFIRMATIONS`, esa racha completa se cuela como si fuera un
cambio de dirección real. No existe un valor fijo que cubra toda racha
posible sin volver el filtro perceptiblemente lento en cambios de
dirección legítimos — es un trade-off entre precisión y responsividad.
Ver `Bugs_Documentados.md` para el detalle completo y la evidencia
reunida durante las pruebas.

## Requisitos

- Windows (usa la API Win32 directamente vía el crate `windows`)
- Rust (instalación vía [rustup](https://rustup.rs))
- Linker: MSVC (Visual Studio Build Tools) o GNU (MinGW-w64)

## Compilar

```powershell
cargo build --release
```

El ejecutable queda en `target\release\wheel-fix.exe` (o el nombre que le
hayas dado al binario).

## Ejecutar

Ejecuta el `.exe` como administrador. Los hooks de bajo nivel de mouse a
veces se comportan mal o son ignorados si el proceso no tiene privilegios
elevados, sobre todo si hay otro software (Armoury Crate, utilidades RGB)
enganchado antes en la cadena de hooks.

## Configuración

Por ahora, ajustable directamente en `src/main.rs`, recompilando después
de cada cambio:

- `REQUIRED_CONFIRMATIONS` — número de ticks consecutivos necesarios para
  confirmar un cambio de dirección. Por defecto `3`. Súbelo si notas que
  el kickback se sigue colando; bájalo si sientes el filtro demasiado
  lento al cambiar de dirección intencionalmente.

## Roadmap

- [ ] Interfaz gráfica con ícono en la bandeja del sistema
- [ ] `REQUIRED_CONFIRMATIONS` ajustable en tiempo real, sin recompilar
- [ ] Hotkey global para activar/desactivar el filtro
- [ ] Toggle de inicio automático con Windows
- [ ] Ícono de bandeja con indicador direccional y color configurable en cada bloqueo

## Créditos y contexto

Encoder identificado como Kailh EN8080 según el desmontaje técnico
publicado por [TechPowerUp](https://www.techpowerup.com/review/asus-rog-gladius-iii/4.html).
El defecto de kickback está reportado en múltiples hilos del
[foro oficial de ASUS ROG](https://rog-forum.asus.com/).
