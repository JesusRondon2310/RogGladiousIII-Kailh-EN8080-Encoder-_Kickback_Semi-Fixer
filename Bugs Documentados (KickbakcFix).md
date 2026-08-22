================================================================================
wheel-fix (Kickback_Fix) - REGISTRO DE BUGS CORREGIDOS
================================================================================
Última actualización: 2026-08-23
Total de bugs: 4

================================================================================
ÍNDICE POR SECCIONES
================================================================================
1.  Compilación / bindings de Win32 (crate windows) ...... BUG 1,2
2.  Lógica de detección de kickback ....................... BUG 3,4
3.  Limitaciones conocidas (no resueltas al 100%) ......... LIMITACIÓN 1
================================================================================

================================================================================
BUG 1 — CallNextHookEx esperaba un puntero, no un número
================================================================================
SÍNTOMA:
  Al compilar, error indicando que se esperaba `void` y se recibió `usize`,
  en la línea CallNextHookEx(HHOOK(0), code, wparam, lparam).
CAUSA:
  En la versión del crate `windows` usada, HHOOK envuelve internamente un
  puntero (*mut c_void), no un entero. HHOOK(0) intentaba construirlo con
  un número, tipo incompatible.
SOLUCIÓN:
  Cambiar HHOOK(0) por HHOOK(std::ptr::null_mut()) — un puntero nulo
  explícito, que además es lo que Windows espera ahí realmente (ese
  parámetro es ignorado por el sistema en versiones modernas).
ARCHIVO: src/main.rs (dentro de hook_proc)
ESTADO: RESUELTO

================================================================================
BUG 2 — .into() ambiguo al convertir HMODULE a HINSTANCE
================================================================================
SÍNTOMA:
  Al compilar, error "type annotations needed", indicando múltiples impls
  que satisfacen Param<HINSTANCE, CopyType>, en la línea
  SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), hmodule.into(), 0)?.
CAUSA:
  El compilador no podía inferir a qué tipo debía convertir hmodule con
  .into(), porque existía más de una implementación de conversión
  aplicable en esa versión del crate `windows`. Usado inline dentro de la
  llamada a la función, no había suficiente contexto de tipo.
SOLUCIÓN:
  Separar la conversión en una variable con tipo explícito antes de la
  llamada:
    let hinstance: HINSTANCE = hmodule.into();
    let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), hinstance, 0)?;
ARCHIVO: src/main.rs (dentro de main)
ESTADO: RESUELTO

================================================================================
BUG 3 — Debounce de ventana fija no cubría el kickback en cadena
================================================================================
SÍNTOMA:
  El primer enfoque (debounce simple: bloquear un tick si su dirección es
  contraria al último tick aceptado y llegó dentro de una ventana corta de
  ~45ms) dejaba pasar kickback en muchos casos.
CAUSA:
  El kickback real del ROG Gladius 3 no es un rebote inmediato aislado: el
  encoder puede generar cadenas de varios ticks fantasma consecutivos en
  dirección contraria mientras el usuario sigue girando físicamente en la
  dirección original, con retrasos observados de hasta ~600ms entre el
  toque real y el fantasma. Un debounce de ventana corta comparando solo
  contra el último tick no detecta ese patrón.
SOLUCIÓN:
  Se reemplazó el debounce simple por un esquema de "dirección confirmada +
  candidato pendiente": se mantiene una dirección confirmada activa; un tick
  en dirección contraria no se acepta de inmediato, se guarda como
  candidato pendiente; solo se confirma (y pasa a ser la nueva dirección
  confirmada) si llega(n) suficiente(s) tick(s) adicional(es) en esa misma
  dirección nueva (ver BUG 4 y LIMITACIÓN 1 para la evolución de "cuántos").
ARCHIVO: src/main.rs (hook_proc, variables CONFIRMED_DIR / PENDING_DIR /
  PENDING_COUNT)
ESTADO: RESUELTO (mitigado; ver LIMITACIÓN 1 para el caso residual)

================================================================================
BUG 4 — Confirmación con ventana de tiempo fija fallaba en scroll lento/pausado
================================================================================
SÍNTOMA:
  Con scroll lento y deliberado (giro, pausa, giro, pausa) o con pausas
  erráticas dentro de una misma racha de kickback, el filtro nunca
  confirmaba el cambio de dirección: cada tick candidato "llegaba tarde"
  respecto al anterior y el usuario se quedaba sin poder cambiar de
  dirección, o el conteo de confirmaciones se reiniciaba constantemente
  alargando el problema.
CAUSA:
  El contador de confirmaciones (PENDING_COUNT) solo sumaba si el tick
  llegaba dentro de una ventana de tiempo fija (CONFIRM_WINDOW_MS, 400ms)
  desde el candidato/confirmación anterior. Pausas reales del usuario, o
  incluso silencios erráticos de cientos de ms dentro de una misma racha de
  kickback, superaban esa ventana y reiniciaban el candidato a cero en vez
  de sumar al conteo.
SOLUCIÓN:
  Se eliminó por completo el límite de tiempo para acumular el contador de
  confirmaciones. El candidato/contador ahora solo se reinicia cuando llega
  un tick real que coincide con la dirección ya confirmada (CONFIRMED_DIR),
  nunca por el simple paso del tiempo. CONFIRM_WINDOW_MS quedó sin uso y se
  eliminó de las constantes.
ARCHIVO: src/main.rs (hook_proc, eliminación de CONFIRM_WINDOW_MS y
  pending_valid)
ESTADO: RESUELTO

================================================================================
LIMITACIÓN 1 — Rachas de kickback más largas que REQUIRED_CONFIRMATIONS
  se cuelan igual
================================================================================
CONTEXTO:
  El esquema de confirmación por N ticks consecutivos (REQUIRED_CONFIRMATIONS)
  reduce la probabilidad de que el kickback se cuele, pero no la elimina:
  si el encoder genera una racha de fantasmas consecutivos en la misma
  dirección contraria más larga que N, esa racha completa se confirma como
  si fuera un cambio de dirección real.
EVIDENCIA:
  Se observaron rachas reales de tamaño variable en pruebas: 2, 3, 4 y al
  menos 6 ticks fantasma consecutivos. Con REQUIRED_CONFIRMATIONS = 3 (4
  ticks totales necesarios para colarse), se confirmaron falsos positivos
  en varias ocasiones. También se confirmó que el patrón no es exclusivo de
  reversiones rápidas y deliberadas: se observó al menos un caso durante
  scroll normal (Nexus Mods, YouTube, Zed), aunque con baja frecuencia
  (1 falso positivo en más de 15 cambios de dirección reales en una sesión
  de prueba prolongada).
POR QUÉ NO SE RESUELVE AL 100%:
  No existe un valor fijo de REQUIRED_CONFIRMATIONS que cubra toda racha
  posible sin volver el filtro perceptiblemente lento en cambios de
  dirección legítimos — es un trade-off entre precisión y responsividad,
  no un bug con una solución única.
MITIGACIÓN PLANEADA:
  Hacer REQUIRED_CONFIRMATIONS ajustable en tiempo real desde la GUI
  (rango 1-10), para que el usuario pueda subirlo si el patrón de kickback
  de su unidad empeora, sin necesidad de recompilar.
ARCHIVO: src/main.rs (const REQUIRED_CONFIRMATIONS)
ESTADO: NO RESUELTO (mitigación parcial vía número de confirmaciones;
  pendiente de exponer como control ajustable en la GUI)

================================================================================
FIN DEL REGISTRO
================================================================================
