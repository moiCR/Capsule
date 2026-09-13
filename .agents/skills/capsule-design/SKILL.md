---
name: capsule-design
description: Diseña y refina el frontend GPUI de Capsule y Orbit con el estilo minimalista del proyecto. Úsala para cambios visuales, módulos, widgets, orbs, satélites, controles, espaciado, tipografía, estados y animaciones. Basa las decisiones en las implementaciones existentes, el tema y la configuración; evita añadir interfaz permanente o decoración innecesaria.
---

# Diseño de Capsule y Orbit

## Dirección

Capsule es una shell contextual de Wayland, no un dashboard web. Su interfaz debe ocupar y reclamar solo lo necesario para la tarea presente.

**Menos elementos simultáneos, una jerarquía clara y detalle bajo demanda.** Minimalismo no significa hacer todo diminuto, esconder acciones indispensables ni sustituir etiquetas comprensibles por iconos ambiguos.

- Parte del módulo o widget existente más parecido. Mejora su coherencia antes de inventar otro lenguaje visual.
- Cada elemento visible debe comunicar un estado útil, permitir una acción necesaria o facilitar la comprensión. Si quitarlo no perjudica ninguna de esas funciones, omítelo.
- No interpretes una petición de diseño como permiso para añadir funciones, opciones de configuración, paneles o dependencias.
- No conviertas la inspiración de Capsule Corp. en decoración temática: evita motivos de anime, neón o estética de ciencia ficción salvo petición explícita.
- Las reglas siguientes dirigen cambios nuevos; no autorizan rediseñar módulos ajenos al encargo.

## Antes de diseñar

1. Delimita la tarea: qué necesita ver o hacer el usuario, cuándo aparece la interfaz y cuándo deja de ser necesaria.
2. Consulta el grafo del proyecto y verifica su cobertura/frescura según las reglas del repositorio. Lee el código actual de la superficie afectada y de un referente cercano. Si el grafo no basta, usa lectura directa y declara la limitación.
3. Revisa el contenedor, el widget, sus estados y el origen de tema/configuración; no diseñes a partir del README solamente.
4. Elige la superficie más pequeña que resuelva la tarea usando las distinciones siguientes.
5. Conserva la estructura y los gestos existentes salvo que el cambio solicitado los afecte. No solicites decisiones que el código ya resuelve.

Las rutas de esta guía son relativas a la raíz del repositorio, no al directorio de la skill. Los valores documentados son referencias observadas, no tokens universales ni sustitutos de la configuración. Verifica su vigencia antes de reutilizarlos.

## Responsabilidades de cada superficie

### Capsule: interacción principal

- Usa el contenedor existente para presentar el módulo activo. No dibujes una segunda cápsula con su propio marco dentro de él.
- Ajusta el tamaño a la información y los controles necesarios. Una acción breve puede vivir en una fila; una búsqueda necesita entrada y resultados, no una página de bienvenida.
- Conserva el morphing de dimensiones y radio. No fuerces todos los módulos a un rectángulo grande idéntico.
- Usa el launcher como referencia para listas compactas y la barra de grabación para estados y controles en una sola fila.
- Un módulo complejo como el dashboard puede agrupar controles. Eso no justifica copiar su densidad en una interacción simple.

### Orbit y orbs: presencia contextual

Orbit gestiona disposición, visibilidad y movimiento; los widgets de orb dibujan el indicador y conectan su interacción.

- Un orb representa un estado activo que merece acceso rápido, no una función instalada. No añadas un orb permanente por cada módulo.
- El patrón actual muestra Shelf cuando contiene elementos y Recording mientras no está detenido. Capsule permite mostrarlos en modo `Default`, sin destino de arrastre activo; abrirlos lleva al módulo correspondiente.
- Conserva esa política salvo que el encargo requiera cambiarla. No mantengas orbs alrededor de todos los módulos por decoración.
- Mantén un círculo con un único símbolo o indicador. Un badge solo se justifica si aporta información operativa, como el número de elementos del Shelf.
- No añadas al orb títulos, descripciones, temporizadores, menús de acciones o leyendas permanentes. El detalle pertenece al módulo que abre.
- Reutiliza `ORB_SIZE` y la geometría de Orbit. No calcules posiciones independientes en cada widget ni crees otro gestor.
- Preserva la estabilidad lateral de los orbs activos, el cierre de huecos y la continuidad al invertir una transición. No los reordenes con cada actualización de estado.
- Respeta `interactive` y las regiones de entrada al cambiar visibilidad. No dejes blancos de clic invisibles ni bloquees el escritorio con el área transparente de la ventana.
- Conserva interacciones especializadas, como arrastrar archivos al Shelf, sin generalizarlas a todos los orbs.

### Satélites: detalle auxiliar

- No confundas los orbs circulares de estado con los paneles de `satellites/`.
- Reutiliza los satélites y su gestor para detalles auxiliares que ya siguen ese patrón; no crees una ventana o sistema de popups paralelo.
- Abrir un detalle no debe desplegar controles adicionales sin relación. Respeta las reglas existentes de colocación, espacio disponible y cierre.

## Lenguaje visual

### Color y superficies

- Obtén los colores del `Theme` existente: `background()`, `background_alt()`, `surface()`, `foreground()`, `foreground_muted()`, `accent()`, `red()` y `green()` según su significado.
- Usa `background` para la base; `surface` y sus opacidades para agrupación y feedback. No asumas que el tema siempre es oscuro.
- Reserva `accent` para selección, acción principal, estado activado o feedback puntual. No conviertas todos los bordes, iconos y fondos en acentos simultáneamente.
- El texto principal usa `foreground`; el secundario, `foreground_muted`. No rebajes tanto la opacidad que el contenido útil deje de leerse.
- Un control activado puede usar fondo de acento, como las pills de conectividad. No confundas esa señal funcional con decoración.
- Reutiliza el borde y la sombra del contenedor. No apiles sombras, marcos ni tarjetas anidadas para fabricar jerarquía; primero usa espacio y alineación.
- No introduzcas gradientes, glow, glassmorphism, blur adicional o colores arbitrarios para hacer el diseño «más moderno».
- Hay colores fijos en los widgets actuales de grabación y blanco fijo en el badge del Shelf. Son excepciones observadas, no una paleta que debas copiar. Prioriza los colores semánticos disponibles sin inventar métodos de tema ni ampliar su esquema fuera del alcance.

### Forma, escala y espaciado

- Mantén la familia de formas existente: contenedor adaptable, pills para controles compactos, círculos para acciones por icono y filas de esquinas suaves para listas.
- Respeta los parámetros que recibe el contenedor y la configuración disponible. No fijes otro radio global, fuente, separación o duración en un widget.
- Usa alineación, proximidad y espacio libre para agrupar. Añade un fondo interno solo cuando distinga un control o grupo funcional real.
- Conserva una densidad compacta pero legible. No llenes el espacio libre con contenido adicional ni reduzcas las zonas interactivas para parecer más minimalista.

Referencias de escala verificables en el código:

| Elemento | Referencia actual | Uso |
| --- | --- | --- |
| Orb | `ORB_SIZE = 26.0`; icono de 13 px | Reutilizar la constante, no duplicarla |
| Indicador de grabación del orb | Punto de 8 px | Estado sin texto adicional |
| `IconButton` | Control de 24 px; icono de 13 px por defecto | Acción compacta reutilizable |
| Controles de la barra de grabación | Control de 26 px; icono de 12 px | Referencia para una fila de actividad |
| Fila del launcher | Radio de 12 px; padding vertical de 6 px | Lista compacta, no una tarjeta de dashboard |
| Texto de aplicación del launcher | Principal de 13 px semibold; secundario de 11 px | Jerarquía breve dentro de una fila |
| Lista del launcher | Separación de 4 px | Ritmo entre resultados |

No conviertas los tamaños pequeños de un badge en tamaños de texto general. No impongas estas medidas a superficies con otras necesidades.

### Tipografía, iconos y contenido

- Hereda la familia configurada mediante el tema/contenedor; Geist es el valor por defecto, no una fuente que debas fijar en cada widget.
- Prefiere un nivel principal y uno secundario. Usa semibold o bold para énfasis localizado, no en cada etiqueta.
- Usa los SVG existentes y conserva su escala óptica. Evita emoji decorativos, mezclar familias de iconos o introducir otro paquete.
- Evita títulos que repiten lo que ya explica el control o el contenido. Una barra de grabación no necesita además una cabecera «Grabación».
- Escribe etiquetas cortas, concretas y localizadas mediante el servicio de idioma existente. No añadas párrafos de ayuda por defecto.
- Conserva elipsis, límites de altura y scroll donde corresponda. Prueba títulos, rutas, traducciones y contadores largos sin agrandar arbitrariamente toda la interfaz.
- Los estados vacíos deben ser discretos: una frase útil y, si aporta contexto, un icono tenue. Sin ilustraciones grandes, onboarding ni llamadas a funciones ajenas.

## Interacción y movimiento

- Haz distinguibles selección, hover, presión y deshabilitado con cambios contenidos de superficie, borde o contraste. No añadas desplazamientos o escalados a cada hover.
- Mantén selección de teclado visible y coherente con el ratón. Conserva Enter, Escape, foco y navegación del módulo existente.
- No hagas depender una acción indispensable únicamente de hover o de un gesto nuevo oculto. Los iconos deben tener significado claro y nombre accesible donde la API lo permita.
- No elimines feedback o confirmaciones necesarias en nombre del minimalismo. Los estados importantes deben entenderse sin depender solo del color.
- Cerrar una vista no equivale a detener su actividad: conserva la distinción entre cerrar la barra de grabación y detener la grabación.
- Reutiliza las transiciones de Capsule y las curvas de Orbit/satélites. Lee `animation_duration_ms` donde se usa; no supongas que todas las animaciones comparten configuración.
- El movimiento explica aparición, retracción o cambio de tamaño. No añadas rebotes independientes, pulsos permanentes o animación ornamental.
- Al interrumpir una animación, continúa desde el estado visual actual. Al terminarla, no mantengas repintados continuos sin una actividad que los necesite.

## Composición GPUI

- Mantén el render declarativo. Los widgets presentan estado y conectan acciones; no instancian servicios ni realizan trabajo bloqueante.
- Extiende el módulo correspondiente en `crates/app/src/capsule/modules/` y sus widgets en `crates/app/src/capsule/widgets/`. No acumules todo el frontend nuevo en `capsule.rs`.
- Reutiliza controles de `crates/ui/src/components/` cuando encajen. Extrae un componente compartido solo si hay reutilización real, no para envolver cada `div()`.
- Usa las entidades y los servicios globales de `AppState`; no dupliques gestores de tema, configuración u Orbit.
- Sigue las reglas del proyecto para trabajo de fondo con Tokio. Las implementaciones antiguas no autorizan copiar patrones que contradigan esas reglas.
- Conserva IDs estables, propagación de eventos, foco y regiones de entrada. Una modificación visual no debe disparar accidentalmente la acción del contenedor padre.

## Referencias de implementación

Lee solo las referencias pertinentes a la tarea:

| Ruta | Qué consultar |
| --- | --- |
| `crates/app/src/capsule/capsule.rs` | `sync_orbit_visibility`, `open_orb`, dimensiones, configuración, foco y regiones de entrada |
| `crates/app/src/capsule/container/normal.rs` | Contenedor con parámetros de tamaño, radio, fuente, fondo y borde |
| `crates/app/src/capsule/orbit.rs` | `ORB_SIZE`, `Layout`, `Motion`, geometría y pruebas de continuidad/visibilidad |
| `crates/app/src/capsule/widgets/record/orb.rs` | Indicador mínimo de grabación o pausa y acceso al módulo |
| `crates/app/src/capsule/widgets/shelf/orb.rs` | Icono, contador y feedback de arrastre |
| `crates/app/src/capsule/widgets/record/bar.rs` | Estado, duración y acciones compactas sin cabecera extra |
| `crates/app/src/capsule/modules/record.rs` | Ancho según estado/contenido y distinción entre cerrar y detener |
| `crates/app/src/capsule/modules/launcher.rs` | Composición de búsqueda/resultados, vacío y navegación de teclado |
| `crates/app/src/capsule/widgets/launcher/app_item.rs` | Jerarquía de fila, selección, elipsis y feedback |
| `crates/app/src/capsule/widgets/dashboard/quick_settings.rs` | Agrupación funcional y pills con estado activado |
| `crates/app/src/capsule/satellites/mod.rs` | Paneles auxiliares, lanes, límites y animación |
| `crates/ui/src/components/button.rs` | `IconButton` y `TextButton` existentes |
| `crates/ui/src/theme/mod.rs` | API real de colores y familia tipográfica |

Estas referencias documentan patrones concretos, no una auditoría visual de todas las superficies. Si el código y esta guía divergen, verifica el estado actual y no inventes APIs ni fuerces medidas antiguas.

## Filtro final

Antes de dar por terminado un diseño, comprueba:

- ¿Se resuelve la tarea sin añadir una nueva superficie permanente?
- ¿Se entiende qué es principal sin otro título, tarjeta o color?
- ¿El orb sigue siendo un indicador/acceso y no un módulo en miniatura?
- ¿Los detalles aparecen cuando hacen falta y desaparecen sin cancelar actividades por accidente?
- ¿Se respetan tema, configuración, densidad y componentes del entorno?
- ¿Se conserva legibilidad con temas claros/oscuros, contenido largo y la escala disponible?
- ¿Funcionan teclado, ratón y, si aplica, arrastre, apertura/cierre rápido y varios orbs?
- ¿La zona transparente deja pasar entrada y los elementos ocultos no capturan clics?
- ¿Se evitó ampliar el alcance con funciones, configuración o refactors no solicitados?

Si cambias Rust, ejecuta `cargo fmt`, las pruebas pertinentes y Clippy según el alcance y las reglas del proyecto. Verifica visualmente los estados afectados cuando puedas ejecutar la shell; si no, di que la revisión fue de código y especifica lo que falta comprobar. No presentes una compilación correcta como validación del diseño visual.
