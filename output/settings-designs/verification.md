# Verificación de la implementación

Panel nativo de 840 × 560: buscador centrado sin título, seis pestañas, selectores de estilo y tarjetas de dimensiones, radios y movimiento. La miniatura cóncava reutiliza los SVG de los extremos reales, unidos al borde superior.

La vista previa utiliza el Animator de new_capsule. Reproducir ejecuta una expansión y una contracción; soltar el deslizador de duración también la activa. No queda un bucle al terminar. Las transiciones de pestañas y de ventana se programan fuera de render mediante on_next_frame.

## Comprobaciones

- Compilación de desarrollo correcta.
- 44 pruebas de new_capsule aprobadas, incluyendo animación interrumpida, duración cero y superficies opacas en ambos temas.
- Formato de capsule y services correcto; git diff --check correcto.
- Clippy --no-deps completa sin diagnósticos en los archivos modificados. Clippy estricto con dependencias falla por advertencias preexistentes en assets, ui y services.
- Inspección real en Wayland: tema oscuro, tema claro, estilo normal, estilo cóncavo, pestaña Aplicaciones, retorno a Cápsula y cierre.
- Capturas de la aplicación, no imágenes generadas: implemented-dark.png, implemented-concave.png, implemented-apps.png e implemented-light.png.

## Medición y límites

Prueba aislada, compilación debug y WAYLAND_DEBUG activo, monitor a 144 Hz. La vista previa registró 40 callbacks de frame, intervalo mediano de 16 ms y máximo de 35 ms. Esto no acredita 144 FPS. Tras finalizar, el panel registró 0 commits durante 3 segundos y el proceso consumió aproximadamente 0,67 % de un núcleo. Las mediciones completas están en measurements.json.

No se verificaron visualmente todas las categorías, entradas largas ni toda la navegación de teclado. La instancia aislada registró fallos previos de servicios D-Bus; las capturas no certifican esos servicios. Se conservan sus rutas existentes.
