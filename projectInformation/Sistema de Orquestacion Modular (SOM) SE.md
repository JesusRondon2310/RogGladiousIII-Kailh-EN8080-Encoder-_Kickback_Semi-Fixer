# Sistema de Orquestación Modular (SOM). *2.ª edición*

*Documentación completa y actualizada — solo Rust* 🦀

## Índice

- [Meta-Regla: Evidencia primero, regla después](#meta-regla-evidencia-primero-regla-después)

1. [¿Qué es SOM?](#1-qué-es-som)
2. [Principios fundamentales](#2-principios-fundamentales)
3. [Niveles de profundidad y nomenclatura de archivos](#3-niveles-de-profundidad-y-nomenclatura-de-archivos)
4. [Roles de los archivos](#4-roles-de-los-archivos)
5. [Reglas de encapsulamiento y acceso](#5-reglas-de-encapsulamiento-y-acceso)
6. [Convenciones de código](#6-convenciones-de-código)
7. [Excepciones controladas (el 20% mutable)](#7-excepciones-controladas-el-20-mutable)
8. [Ejemplo práctico: estructura de una aplicación modular](#8-ejemplo-práctico-estructura-de-una-aplicación-modular)
9. [Comparación y complementariedad con otros patrones](#9-comparación-y-complementariedad-con-otros-patrones)
10. [Guía de aplicación de SOM](#10-guía-de-aplicación-de-som)
11. [Origen y filosofía](#11-origen-y-filosofía)

---

## Meta-Regla: Evidencia primero, regla después

Es la regla que gobierna a todas las demás, incluyendo cómo evoluciona este
mismo documento.

Las reglas de SOM se formalizan solo cuando el proyecto choca con la
necesidad real: síntoma concreto → causa → regla general. No se agrega una
regla, ni estructura (carpetas, archivos, capas), "por si acaso". Un archivo
o un módulo se divide cuando de verdad crece o mezcla responsabilidades, no
antes. Una sección sin evidencia suficiente se deja explícitamente abierta,
en vez de rellenarse con reglas especulativas o copiadas de otra fuente sin
haberlas comprobado en un proyecto propio.

---

## 1. ¿Qué es SOM?

SOM (Sistema de Orquestación Modular) es un patrón de encapsulamiento
especializado en gestión de errores determinista y trazabilidad (backtracing)
sin dependencia de herramientas de depuración (debuggers).

Organiza el sistema en módulos independientes donde cada uno expone un único
punto de entrada público (el orquestador) y oculta estrictamente su
funcionamiento interno al resto de la aplicación. Al forzar flujos de
ejecución unidireccionales y lineales, el patrón confina los fallos y produce
un stack trace predecible que permite diagnosticar la causa raíz de cualquier
anomalía directamente desde los logs de producción.

Nació de la adaptación del sistema de módulos de Rust a C#, y es aplicable a
cualquier lenguaje que permita distinguir entre código público y privado.

---

## 2. Principios fundamentales

### 2.1. Un orquestador por módulo

Cada módulo tiene un único archivo o clase pública que actúa como punto de
entrada. Nadie más en el módulo es visible desde fuera.

### 2.2. Encapsulamiento estricto

Todas las funciones y estructuras internas del módulo son inaccesibles desde
otros módulos. El compilador lo impone: todo es privado por defecto en Rust;
`pub` marca el orquestador, `pub(super)` (o `pub`/`pub(crate)` en la función
puntual del mini-orquestador, ver [4.2](#42-mini-orquestador)) el resto.

### 2.3. Archivos pequeños y responsables

Cada archivo tiene una sola responsabilidad. No hay archivos de 300 líneas
que mezclen lógica de negocio, persistencia y UI.

**Límite de líneas por archivo:**

- **Recomendado:** 120 líneas.
- **Máximo aceptable (aún SOM):** 125 líneas.
- Por encima de 125 líneas, el archivo deja de ser SOM y debe refactorizarse
  extrayendo responsabilidades en nuevos componentes internos o
  mini-orquestadores, o subiendo de nivel si corresponde.
- Si un archivo alcanza 125 líneas por razones justificadas (cohesión
  máxima, funcionalidad que no puede dividirse), se acepta temporalmente,
  pero queda registrada como deuda técnica para futura refactorización.

**Excepción controlada para proyectos grandes:**

En proyectos grandes o masivos —donde SOM se combina con otro patrón de
arquitectura (Hexagonal, MVC, Clean Architecture) y el número de módulos
hace que el tope de 125 líneas sea una restricción molesta— el equipo puede
acordar un límite superior. Esto rompe la [Meta-Regla](#meta-regla-evidencia-primero-regla-después)
(no ampliar estructura por especulación) de forma deliberada y acotada; por
eso la ampliación:

- La decide el equipo de forma unánime y queda documentada en la guía de
  estilo del proyecto.
- Mantiene intacto el principio de "una responsabilidad por archivo" y la
  trazabilidad lineal.
- Se revisa periódicamente.

Límites para ese caso:

- **Rango sugerido:** 120–300 líneas. Sigue siendo el objetivo por defecto.
- **Tope máximo (sin deuda):** 500 líneas.
- **Tope máximo con deuda técnica:** 550 líneas, con refactorización
  planificada. No se mantienen archivos en este rango de forma indefinida.
- La cohesión no justifica por sí sola superar las 500 líneas: suele
  indicar que el archivo mezcla responsabilidades y debe dividirse.

### 2.4. Trazabilidad directa

El flujo de ejecución es lineal y predecible. Una acción en el menú llama al
orquestador, que delega en sus internos. El stack trace señala exactamente
dónde falló algo.

### 2.5. Independencia entre módulos

Los módulos no se acoplan entre sí. Si un módulo falla, otro no se entera.
Si un módulo necesita algo de otro, lo pide a través de su orquestador
público, o directamente a la función puntual `pub` de un mini-orquestador
ajeno cuando ese mini-orquestador la expuso para eso (ver [4.2](#42-mini-orquestador)).

### 2.6. Delegación de entrada (UI / controladores)

El orquestador principal (core, manager o handler) **no debe contener
lógica de detección de entrada** (eventos de teclado, ratón, etc.). Esa
lógica debe delegarse en componentes internos especializados o, cuando la
complejidad lo requiera, en mini-orquestadores (ver [sección 4.2](#42-mini-orquestador)).
Los mini-orquestadores solo pueden existir a partir del nivel 3 de
profundidad; en niveles inferiores (1 y 2), la delegación se realiza
mediante componentes internos simples. El orquestador invoca funciones de
sus internos o mini-orquestadores; nunca realiza comprobaciones de entrada
directamente.

---

## 3. Niveles de profundidad y nomenclatura de archivos

Los módulos se organizan en carpetas con un máximo de 3 niveles de
profundidad desde la raíz del proyecto.

Cada nivel tiene un nombre de archivo específico para su orquestador. El
manifiesto del módulo (`mod.rs`, el que declara `mod ...;` y re-exporta con
`pub use`) es un archivo aparte del orquestador — el orquestador vive en su
propio archivo, con el nombre del nivel:

| Nivel | Archivo       | Rol                                                                  | Ejemplo                                                              |
| ----- | ------------- | --------------------------------------------------------------------- | --------------------------------------------------------------------- |
| 1     | `core.rs`     | Orquestador principal del módulo de primer nivel.                     | `inventory/mod.rs` + `inventory/core.rs`                              |
| 2     | `manager.rs`  | Orquestador del submódulo de segundo nivel; coordina funcionalidades. | `inventory/transaction/mod.rs` + `inventory/transaction/manager.rs`   |
| 3     | `handler.rs`  | Orquestador del submódulo de tercer nivel; lógica de negocio específica y coordinación de internos. | `inventory/transaction/validation/mod.rs` + `.../handler.rs` |

Estos niveles se añaden a medida que el módulo crece, no desde el inicio. Un
módulo pequeño puede tener solo su orquestador (`core`) y componentes
internos, sin `manager` ni `handler`. La cadena core → manager → handler no
es un requisito de arranque: se instancia cuando la complejidad real lo pide
(ver [Meta-Regla](#meta-regla-evidencia-primero-regla-después)).

Los archivos que no son orquestadores se nombran según su responsabilidad
específica, en snake_case (`item_validator.rs`, `price_calculator.rs`, etc.).

---

## 4. Roles de los archivos

### 4.1. Orquestador principal del módulo

- Es el único punto de entrada público del módulo: `pub fn` en su archivo.
- Nombres reservados: `core.rs` (nivel 1), `manager.rs` (nivel 2),
  `handler.rs` (nivel 3).
- No depende de ningún otro archivo dentro del mismo nivel.
- **Por defecto, cada `pub fn` es una sola línea de delegación pura** hacia
  un interno o mini-orquestador — nunca calcula nada del dominio. Puede
  coordinar varios colaboradores distintos (una línea, un destino cada
  vez); no está atado a delegar siempre en el mismo interno.
- Puede ser comando (devuelve `()`, dispara la acción y no le importa el
  resultado) o consulta (devuelve un valor, tal cual lo entregó el interno,
  sin transformarlo).
- Puede mantener estado propio del módulo (ej. una colección construida por
  un interno), pero no lo transforma él mismo — lo recibe ya armado.
- **Los únicos dos tipos de lógica propia permitidos** dentro de una
  `pub fn` del orquestador, ambos de coordinación, nunca de dominio:
  1. **Inicialización/bootstrap** del estado propio del módulo — la
     primera vez que el módulo necesita construirse a sí mismo (ej.
     cargar de una base de datos y crear su instancia interna). Se marca
     como excepción explícita con un comentario, no se disimula como si
     fuera delegación pura.
  2. **Guard clause de precondición** — decide *si* delegar o no
     (`if !inicializado { return Err(...); }`, o su forma compacta con
     `?` sobre un `Option`/`Result`, o `.ok_or(...)?`). Nunca decide
     *qué* calcular, solo si hay base para seguir.
- **Nunca llama primitivas de plataforma directamente** (FFI, `unsafe`,
  syscalls) en su propio cuerpo — eso vive en un componente interno que
  las encapsula y expone una interfaz segura (ver [7.1](#71-el-20-mutable-poo)).
  Un orquestador que hace esto dejó de ser un orquestador puro, aunque el
  resto de sus `pub fn` sean delegación — hay que extraer esa parte a un
  interno dedicado.

### 4.2. Mini-Orquestador

Un mini-orquestador es un tipo especial de componente interno que contiene y
ejecuta su propia lógica de negocio específica — no delega en otros
componentes internos, resuelve la tarea por sí mismo. Su función es similar
a la de un conserje en un hotel: tiene su propio "cubículo" (archivo) donde
guarda sus propias herramientas (funciones auxiliares privadas a él) y sabe
cómo realizar tareas concretas por sí mismo, bajo demanda — ya sea porque el
recepcionista de su propio piso (el orquestador de su módulo) se lo pide, o
porque alguien de otro piso tocó directo su puerta para la tarea puntual que
él mismo anunció (ver *Visibilidad* más abajo).

**Restricción de nivel:**

- Los mini-orquestadores **solo pueden existir a partir del nivel 3** de
  profundidad (ej. `nivel1/nivel2/nivel3/mi_mini_orquestador.rs`).
- En los niveles 1 y 2 no se definen mini-orquestadores; toda la lógica de
  delegación se maneja mediante componentes internos comunes.

**Visibilidad:**

- El struct/módulo del mini-orquestador es siempre **privado por defecto**
  (sin `pub`). Nunca se marca `pub` completo.
- Son accesibles por **todos los archivos que pertenecen al mismo módulo**,
  sin importar el nivel de profundidad en el que se encuentren (nivel 1, 2
  o 3), ya que todos comparten el mismo encapsulador.
- Si un archivo de **otro módulo** necesita una función puntual de un
  mini-orquestador —situación real: SOM limita la profundidad a 3 niveles,
  y el mini-orquestador es donde se mete lógica adicional sin poder subir
  de nivel—, esa función concreta se marca `pub` (o `pub(crate)` si solo
  debe verse dentro del mismo crate, nunca fuera de él). El resto del
  struct/módulo se queda privado. Nunca se abre todo para exponer una sola
  función; de ahí el nombre "mini-orquestador": solo esa función puntual
  actúa como punto de entrada, igual que un orquestador, pero a escala de
  una sola función.
- **A favor de Rust:** a diferencia de otros lenguajes donde esta
  distinción puede no estar impuesta por el compilador en ciertos
  contextos (ej. un ensamblado único en C#, donde todo `internal` ya es
  visible en todo el proyecto), en Rust la privacidad de módulo la impone
  el compilador **siempre**, incluso dentro de un mismo crate. No hay
  escenario donde la frontera dependa solo de la disciplina del equipo.

**Nomenclatura:** `nombre_especifico_handler.rs` (ej:
`price_calculator_handler.rs`, `input_handler.rs`).

### 4.3. Componente interno

- Structs/funciones que no son orquestadores ni mini-orquestadores.
- Pueden ser structs de datos, helpers, factories, etc.
- **Visibilidad:** privado por defecto (sin `pub`).
- **Excepción:** los helpers de uso general ubicados en la carpeta
  `helpers/` deben ser `pub` para permitir su reutilización desde
  cualquier módulo del proyecto (ejemplos: `timers.rs`,
  `string_helper.rs`, `math_utils.rs`, `constants.rs`).
- `helpers/` es SOLO para lo reutilizable entre varios módulos. Un valor,
  constante o utilidad que solo usa un módulo se queda dentro de ese
  módulo, no sube a `helpers/`.
- **Un interno nunca llama a la `pub fn` de otro módulo directamente** —
  esa llamada cruzando módulos es exclusiva del orquestador o el
  mini-orquestador (ver [5.3](#53-comunicación-entre-módulos)). Si un
  interno necesita un dato de otro módulo, se lo pasa por parámetro quien
  lo invoca (orquestador/mini-orquestador), el interno no lo va a buscar
  por su cuenta. Un interno que cruza esa línea dejó de ser un interno —
  hay que corregirlo, o reconocer que en realidad hace trabajo de
  mini-orquestador y subir su rol.

---

## 5. Reglas de encapsulamiento y acceso

### 5.1. Visibilidad

- Orquestador principal: `pub` (solo él es accesible desde fuera).
- Mini-orquestadores: el struct/módulo siempre privado por defecto (sin
  `pub`); funciones puntuales `pub` según la regla [4.2](#42-mini-orquestador).
- Componentes internos: privado por defecto (salvo helpers de `helpers/`
  que son `pub`).

### 5.2. Dependencias

Cada orquestador (`core`, `manager`, `handler`) es un archivo independiente
— no depende de otro orquestador para existir ni para funcionar.
"core/manager/handler" es solo el nombre según el nivel de profundidad de
la carpeta (ver [sección 3](#3-niveles-de-profundidad-y-nomenclatura-de-archivos)),
no una relación de dependencia entre ellos. Lo único que hace cualquier
orquestador es delegar hacia los internos y mini-orquestadores de **su
propio** nivel/carpeta. Si necesita algo de un submódulo anidado, delega
hacia el orquestador público de ese submódulo (`manager` o `handler`)
exactamente igual que delegaría hacia cualquier otro colaborador — es una
llamada más, no una dependencia estructural.

Accesibilidad dentro del módulo: ver [4.2](#42-mini-orquestador). Entre
mini-orquestadores del mismo módulo pueden hablarse como pares (pedirse
información, coordinarse puntualmente), pero ninguno delega en otro ni lo
orquesta — esa sigue siendo función exclusiva del orquestador.

Un archivo de otro módulo nunca accede a un mini-orquestador **como struct
completo**. La vía por defecto sigue siendo pasar por el orquestador
público del módulo destino. La excepción es la función puntual que el
propio mini-orquestador marcó `pub` (ver [4.2](#42-mini-orquestador)): esa
función sí se llama directo desde otro módulo. La diferencia de fondo entre
ambos roles: el orquestador delega y expone la lógica de **otros**
(coordina); el mini-orquestador delega y expone **su propia** lógica
interna, bajo demanda, por sí mismo — no necesita al orquestador para eso.
El resto del struct sigue tan inaccesible como siempre.

En resumen: un orquestador (a cualquier nivel) delega hacia los internos y
mini-orquestadores de su propio módulo; un mini-orquestador nunca delega,
se resuelve a sí mismo.

```
Orquestador → { Interno | Mini-Orquestador (resuelve por sí mismo) }
```

### 5.3. Comunicación entre módulos

- Un módulo accede al orquestador público de otro módulo, o —cuando el
  mini-orquestador expuso una función puntual `pub` (ver [4.2](#42-mini-orquestador))—
  a esa función directamente. Nunca accede al resto de los internos de
  otro módulo.
- Los componentes internos de un MISMO módulo se comunican entre sí sin
  restricción: comparten el mismo encapsulador.
- El cruce ENTRE módulos pasa por el orquestador del módulo destino, que
  delega internamente en sus propios componentes — nunca reexpone la
  lógica de un mini-orquestador, porque el mini-orquestador ya se expone y
  se delega a sí mismo. La excepción es la función puntual marcada `pub`
  de un mini-orquestador ([4.2](#42-mini-orquestador)): el módulo llamador
  invoca esa función específica directamente, sin pasar por el
  orquestador — el resto del struct sigue fuera de alcance.

**Analogía del hotel:**

Imagina un hotel con varios pisos. Cada piso tiene un **recepcionista**
(orquestador) que atiende a los huéspedes y visitantes. En ese piso hay
**conserjes** (mini-orquestadores), cada uno con su propio cuarto de
herramientas (archivo) donde guarda utensilios específicos. Los
**inquilinos** (internos) son los que realizan las tareas más básicas.

- El recepcionista recibe las peticiones de fuera (otros pisos o el
  exterior) y las resuelve con su propio personal (internos) — nunca
  reenvía el trabajo a un conserje, porque el conserje ya atiende directo
  lo suyo.
- El conserje (mini-orquestador) tiene lógica de negocio específica: sabe
  cómo hacer ciertas tareas y para ello utiliza sus propias herramientas
  (componentes internos).
- Los inquilinos (internos) realizan tareas concretas sin orquestar nada.
- Por defecto, si alguien de otro piso necesita algo, no sube directamente
  a tocar la puerta del conserje: habla con el recepcionista del piso
  destino, quien resuelve con su propio personal. La excepción es cuando
  el conserje anunció una tarea puntual que él mismo atiende — ahí sí
  puede tocarle la puerta directo, sin pasar por ningún recepcionista.
- **Los conserjes (mini-orquestadores) solo son conocidos por el personal
  del mismo piso (módulo) — salvo que el propio conserje haya sido llamado
  desde otro piso para una tarea puntual. Todo lo demás de su cuarto sigue
  siendo invisible para fuera.**

**Regla de niveles:** los conserjes (mini-orquestadores) solo existen a
partir del tercer piso (nivel 3). Los pisos 1 y 2 solo tienen
recepcionistas y personal auxiliar (internos).

---

## 6. Convenciones de código

6.1 a 6.3 son recomendaciones de estilo. 6.5 y 6.6 son reglas: su
incumplimiento ha causado bugs reales (ver el origen en [6.5](#65-modelado-de-estados)).

### 6.1. Nombres de archivos

- El archivo del orquestador usa el nombre reservado de su nivel
  (`core.rs`/`manager.rs`/`handler.rs`, ver [sección 3](#3-niveles-de-profundidad-y-nomenclatura-de-archivos))
  — no el nombre de ningún struct que contenga.
- Los archivos que no son orquestadores (mini-orquestadores, internos) se
  nombran según su responsabilidad, en snake_case, no según el nombre de
  un struct público.

### 6.2. Comentarios y legibilidad

- Se usan comentarios solo cuando el código no se explica por sí mismo.
- Los comentarios son breves y no obstructivos.
- Se evitan comentarios largos que repitan lo que el código ya dice.
- Excepción: para un flujo de ejecución complejo que el código no puede
  proyectar por sí solo, se usan comentarios numerados (`1.`, `1.1.`, y
  `2.2b.` para pasos insertados sin renumerar el resto) que marcan la
  secuencia. Ahí el comentario SÍ es necesario, no es ruido. Cada archivo
  numera su propio flujo desde 1; al dividir un archivo, se renumera por
  archivo.
- No se ponen comentarios de cabecera que expliquen el rol del archivo —
  el nombre del archivo y su estructura ya lo dicen.
- "Breve" se refiere a no repetir lo que el código dice, no al ancho de
  línea. El único límite de tamaño es el de líneas por archivo (sección [2.3](#23-archivos-pequeños-y-responsables)).

### 6.3. Estilo de código

- Se evita la duplicación de código (DRY) siempre que no fuerce una
  abstracción innecesaria.
- Un valor literal se reemplaza por una constante con nombre SOLO cuando
  el valor carga un significado de dominio (`BLOCK`, `MAX_INTENTOS`,
  `SIN_SELECCION`). Los `0` y `1` en contexto convencional —índices,
  conteos, inicialización, flags booleanos— NO son "valores mágicos" y no
  necesitan nombre; nombrarlos (`ZERO`, `ONE`) agrega ruido. `clippy` (el
  linter de Rust) no marca números pequeños como mágicos.

### 6.5. Modelado de estados

- Un estado no se modela con cadenas de `if x == N` no exhaustivas. Se usa
  un tipo con variantes nombradas (`enum`), se usa Result Pattern.
- El compilador de Rust verifica la exhaustividad de un `match` sobre un
  `enum`: vuelve imposible compilar si se olvida una transición.
- Origen: en un caso real, un estado modelado como `int fase` (0..3) sin
  ningún código que devolviera `fase` a 0 dejó un componente
  auto-desactivado de forma permanente. Un enum con `match` exhaustivo lo
  habría hecho imposible de compilar sin cubrir el caso.

### 6.6. Manejo de errores: Result Pattern

- El mecanismo obligatorio para errores esperables (validación, I/O,
  parseo, cualquier fallo que forme parte del flujo normal) es el
  **Result Pattern**: la función devuelve explícitamente `Result<T, E>`
  en vez de entrar en pánico.
- `panic!` queda reservado para lo verdaderamente excepcional: bugs
  internos, invariantes rotas, fallos no recuperables. No se usa para
  flujo de control normal.
- El match exhaustivo de [6.5](#65-modelado-de-estados) no es la regla
  aquí, es su consecuencia: el Result Pattern, por diseño, obliga a
  evaluar el caso de éxito y el de error antes de continuar (con `match` o
  con `?`). No hay forma de "olvidar" el error sin que el compilador se
  queje.
- En Rust esto es prácticamente gratis — es el mecanismo nativo del
  lenguaje (`Result<T, E>`, propagación con `?`). La recomendación aquí es
  simplemente no evitarlo: no usar `.unwrap()`/`.expect()` fuera de un
  caso donde el fallo es realmente un bug, no envolver todo en `panic!`
  para ahorrarse modelar el error.
- Origen/razón: un error esperado que se propaga como `panic!` sin
  capturar rompe la trazabilidad lineal de SOM (ver [2.4](#24-trazabilidad-directa))
  — el llamador no sabe que la función puede fallar sin leer su
  implementación (a diferencia de un `Result<T, E>` en la firma, que sí
  lo documenta en el tipo).
- El 20% mutable (ver [7.1](#71-el-20-mutable-poo)) es donde sí se traduce
  una falla externa a `Result`: interop, FFI, plataforma. El componente
  interno que envuelve esa API convierte lo que devuelva (código de
  error, puntero nulo, etc.) en un `Result` antes de devolver el control
  al resto del módulo. El 80% funcional nunca ve ese detalle de
  plataforma cruzar hacia adentro.

---

## 7. Excepciones controladas (el 20% mutable)

### 7.1. El 20% Mutable (POO)

- Los módulos que interactúan con APIs externas mutables (UI, hardware,
  servicios) pueden mantener estado mutable interno.
- Esto incluye referencias a manejadores de ventanas, colas de eventos,
  conexiones.
- Estos módulos representan el 20% del código total.
- El orquestador NO llama a primitivas de plataforma (FFI, `unsafe`,
  syscalls) en su propio cuerpo. Ese código vive en los componentes
  internos, que lo encapsulan y exponen una interfaz segura: funciones
  sin tipos de plataforma (handles, structs FFI) en la firma. El
  orquestador solo coordina esas funciones.
- El bloque `unsafe` se acota a la llamada concreta, no a la función
  entera.

### 7.2. ¿Cuándo se permite?

- Solo en los módulos de interfaz de usuario (UI) o de integración con
  librerías que requieran estado mutable.
- En los observadores que registran callbacks para eventos asíncronos.
- En helpers que encapsulan llamadas a sistemas externos (archivos, red,
  etc.).

### 7.3. ¿Cuándo NO se permite?

- En el motor funcional (80% del código): reglas de negocio, cálculos,
  validaciones.
- En objetos de dominio (entidades, DTOs): deben ser inmutables.

### 7.4. Uso controlado de objetos mutables externos

Aunque el sistema evita depender de objetos mutables externos (parte del
20%), existen **casos legítimos** donde se permite su uso directo:

- **Leer un struct externo vivo para construir un valor inmutable
  propio**: ej. leer los campos de `*const MSLLHOOKSTRUCT` dentro del
  callback del hook (válido solo durante esa llamada) para construir
  `direction`/`streak` — en ese instante el dato es fiable, y el
  resultado que se construye ya es inmutable.
- **Consultar el estado vivo de un recurso externo para decidir una
  acción**: ej. comprobar si una conexión (`TcpStream`) sigue abierta, o
  si un archivo sigue existiendo, antes de actuar — comparando contra el
  registro inmutable ya guardado, sin persistir el objeto vivo en sí.

En cualquier otro contexto, debe consultarse el registro inmutable en
lugar del objeto externo vivo.

### 7.5. Flags mutuamente excluyentes en entidades

Las entidades (inmutables en su mayoría, pero con estado de ciclo de vida)
deben mantener consistencia entre sus flags. Las siguientes parejas nunca
pueden ser `true` simultáneamente:

- `is_deleted` y `is_archived`
- `is_pending` y `is_confirmed`
- `is_locked` y `is_available`

**Regla:** al modificar un flag de estado, las funciones responsables
deben revisar los flags relacionados y forzarlos a valores coherentes.
Esta responsabilidad recae en los mini-orquestadores o componentes
internos que modifican la base de datos.

---

## 8. Ejemplo práctico: estructura de una aplicación modular

```
my_app/
├── src/
│   ├── main.rs
│   ├── core/                      ← Nivel 1 (Encapsulador)
│   │   ├── mod.rs (manifiesto: `mod ...;` + `pub use core::...;`)
│   │   ├── core.rs (orquestador público)
│   │   ├── data/                  ← Nivel 2 (Encapsulador)
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs (orquestador)
│   │   │   ├── readers/           ← Nivel 3 (Encapsulador)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── handler.rs (orquestador nivel 3)
│   │   │   │   └── csv_reader_handler.rs (mini-orquestador, solo permitido a partir del nivel 3)
│   │   │   ├── writers/           ← Nivel 3 (Encapsulador)
│   │   │   └── validators.rs
│   │   ├── calculations/          ← Nivel 2 (Encapsulador)
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs (orquestador)
│   │   │   ├── formulas/          ← Nivel 3 (Encapsulador)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── handler.rs
│   │   │   │   └── tax_handler.rs
│   │   │   └── helpers.rs
│   │   └── ui/                    ← Nivel 2 (Encapsulador)
│   │       ├── mod.rs
│   │       ├── manager.rs (orquestador nivel 2)
│   │       ├── windows/           ← Nivel 3 (Encapsulador)
│   │       ├── buttons/           ← Nivel 3 (Encapsulador)
│   │       ├── build.rs (interno)
│   │       └── fill.rs (interno)
│   └── helpers/                   ← Helpers globales (`pub`)
│       ├── mod.rs
│       ├── timers.rs
│       ├── math_helper.rs
│       └── string_helper.rs
```

---

## 9. Comparación y complementariedad con otros patrones

SOM es un patrón de encapsulamiento, no una arquitectura. Por eso no compite
con MVC, Hexagonal o Clean Architecture; los complementa.

| Patrón             | Tamaño archivos        | Encapsulamiento | Trazabilidad | Curva aprendizaje |
| ------------------ | ---------------------- | ---------------- | ------------- | ------------------ |
| SOM                | Pequeños (120-125 max) | Alto             | Alta          | Baja               |
| MVC                | Medianos (100-200)     | Medio            | Baja          | Media              |
| Monolítico         | Grandes (300-500)      | Bajo             | Media         | Baja               |
| Event-driven       | Pequeños               | Muy alto         | Muy baja      | Alta               |
| Clean Architecture | Pequeños/medianos      | Muy alto         | Baja          | Muy alta           |

**Cuándo usar cada combinación:**

- **Proyecto pequeño-mediano** (ej. utilidad, script, mod): SOM solo es
  suficiente. La estructura SOM (core → manager → handler) proporciona
  organización clara y trazabilidad sin complejidad adicional.
- **Proyecto grande** (ej. aplicación empresarial, sistema con múltiples
  frontends): SOM + Hexagonal/Clean Architecture. SOM gestiona la
  trazabilidad dentro de los casos de uso; Hexagonal gestiona el
  desacoplamiento de infraestructura.
- **Proyecto con UI compleja:** SOM + MVC. SOM gestiona la lógica detrás
  del Controlador y dentro del Modelo; MVC gestiona la interacción con el
  usuario.

---

## 10. Guía de aplicación de SOM

Esta guía asume que quieres aplicar SOM a un proyecto nuevo. Si el proyecto
ya existe, se puede aplicar de forma gradual, módulo por módulo.

### Paso 1: Identificar los módulos de alto nivel

Pregunta: ¿qué partes del sistema son independientes entre sí?

- Ejemplos: `UI`, `Lógica`, `Persistencia`, `Red`, `Hardware`.

### Paso 2: Definir el orquestador de cada módulo

El orquestador es el único punto de entrada público. Expone `pub fn` que
representan las operaciones que el módulo ofrece al resto del sistema — en
el archivo raíz del módulo (`core.rs`/`manager.rs`/`handler.rs` según el
nivel), exportadas con `pub use` desde el manifiesto (`mod.rs`).

**Recomendación:** `mod` es el mecanismo de control de visibilidad —
declara qué archivos pertenecen al módulo y, por sí solo, los mantiene
privados hacia afuera; `pub use` es lo único que abre una puerta
específica. Ejemplo real (`filter/mod.rs`):

```rust
mod core;
mod detection;
mod injector;
pub use core::run;
```

Aquí `detection` e `injector` quedan declarados (y por eso visibles entre
sí y desde `core`, dentro del mismo módulo `filter`), pero al no tener su
propio `pub use`, son invisibles para cualquier módulo fuera de `filter`.
Solo `run` — la única función que el orquestador `core` expone — cruza esa
frontera.

### Paso 3: Dividir la lógica interna en mini-orquestadores

- Nivel 1: `core` (orquestador principal). Solo orquestador y componentes
  internos básicos. No hay mini-orquestadores en este nivel.
- Nivel 2: `manager` (coordinador de funcionalidades). Solo orquestador y
  componentes internos. Sigue sin haber mini-orquestadores.
- Nivel 3: `handler` (lógica específica). Aquí **ya se permiten**
  mini-orquestadores (ej. `price_calculator_handler.rs`,
  `input_handler.rs`). Estos contienen lógica de negocio propia, resuelta
  por sí mismos (ver [4.2](#42-mini-orquestador)).

Los mini-orquestadores son **internos al módulo** como struct: cualquier
archivo dentro del mismo módulo puede usarlos completos. Un archivo de otro
módulo solo puede llegar a la función puntual que el mini-orquestador
marcó `pub` para eso (ver [4.2](#42-mini-orquestador)) — nunca al resto
del struct.

### Paso 4: Asegurar que cada archivo tenga una sola responsabilidad

- Si un archivo supera el límite acordado (120 líneas por defecto),
  extrae responsabilidades a nuevos mini-orquestadores o componentes
  internos.
- En proyectos grandes, se puede ampliar el límite (ver [sección 2.3](#23-archivos-pequeños-y-responsables)).

### Paso 5: Establecer las dependencias entre módulos

- Aplica la independencia entre módulos (ver [2.5](#25-independencia-entre-módulos)).

### Paso 6: Configurar el registro de contexto

- Cada orquestador y mini-orquestador registra su entrada y salida con
  contexto.
- En caso de error, el log debe decir: "Módulo X, paso Y, con datos Z,
  falló".

### Paso 7: (Opcional) Añadir simulación y pruebas

- Si el módulo interactúa con hardware o servicios externos, extrae esa
  lógica a componentes internos que puedan ser reemplazados por versiones
  de simulación.
- Escribe pruebas unitarias que verifiquen el comportamiento del
  algoritmo sin las dependencias externas.

### Paso 8: (Opcional) Integrar con MVC o Hexagonal

- Si el proyecto es grande, usa MVC para la UI y Hexagonal para la
  infraestructura.
- SOM se encarga de la trazabilidad dentro de los casos de uso.

---

## 11. Origen y filosofía

SOM nace de la adaptación del sistema de módulos de Rust a C#. En Rust,
cada archivo es un módulo, todo es privado por defecto, y solo lo marcado
con `pub` es accesible desde fuera. Esa filosofía de "privado por defecto,
público bajo demanda" se trasladó a C# usando `internal` y
`public static class` como orquestadores.

> "Si Rust puede tener módulos limpios y encapsulados sin herencia ni
> interfaces innecesarias, C# también puede."

**Nota:** SOM está concebido para trabajar en armonía con la programación
funcional. Aunque permite un 20% de código mutable para interactuar con
APIs externas, el 80% del motor de negocio (cálculos, validaciones,
transformaciones de datos) debe ser puro e inmutable, favoreciendo
funciones sin efectos secundarios, transparencia referencial. El manejo de
errores recomendado es vía Result Pattern (ver [6.6](#66-manejo-de-errores-result-pattern));
`Option<T>` u otros tipos se usan cuando aplique para modelar ausencia de
valor. La arquitectura no impone un estilo funcional en lo demás, pero su
diseño de encapsulamiento y trazabilidad lineal lo hace especialmente
compatible con él.
