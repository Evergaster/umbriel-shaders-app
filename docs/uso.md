# Uso

Guía de las cuatro vistas de Umbriel Shaders y de lo que hace cada botón. La app es
bilingüe (ES/EN); aquí se usan los textos en español.

## Arranque

Abre **Umbriel Shaders** desde el menú o ejecuta `umbriel-shaders`. La ventana no usa
decoraciones del sistema: se mueve arrastrando la barra superior, y los tres botones de
la derecha minimizan, maximizan y cierran.

Arriba a la izquierda está el logotipo y la versión. A la derecha, el indicador de
estado y el contador de shaders:

- **Umbriel conectado** — `umbriel effects --json` responde; se puede aplicar.
- **Umbriel no disponible** — no se encontró el binario o no responde. Puedes seguir
  explorando y previsualizando shaders, pero aplicar fallará.

Debajo, la cinta **Orden de ejecución** resume el archivo de animaciones que se edita y
tiene los dos botones globales: **Deshacer** y **Restaurar original**.

## Biblioteca

Es la vista principal. A la izquierda, el buscador y las secciones; en el centro, la
tabla de shaders; a la derecha, el detalle del shader seleccionado.

### Secciones y búsqueda

Las secciones filtran por tipo de efecto: **Todo**, **Animaciones**, **Bordes**,
**Mouse**, **Pantalla**, **Ventanas**. La búsqueda filtra por nombre mientras escribes.
El contador de la cabecera muestra cuántos shaders hay en total.

### Catálogo oficial

La tarjeta **Catálogo oficial** descarga la colección de la comunidad
(`noctalia-dev/community-umbriel-shaders`) y la instala en tu biblioteca:

- **Descargar catálogo** la primera vez; **Actualizar catálogo** después.
- Muestra el progreso («Descargando N de M») y verifica el SHA-256 de cada shader
  contra el catálogo publicado.
- Al terminar informa de cuántos se instalaron y de los que fallaron; los fallos no
  interrumpen el resto.

La descarga solo funciona dentro de la app de escritorio (necesita red desde el
backend); en la versión de navegador con backend simulado aparece un aviso.

### Instalar ejemplos e importar

- **Instalar ejemplos** copia los ocho shaders incluidos en la app a tu biblioteca, solo
  si aún no existen. No sobrescribe archivos que hayas modificado.
- **Importar shader** pide una ruta: un archivo `.glsl` o una carpeta. Si es una carpeta,
  se copian todos los `.glsl` que contenga. Tras importar, la biblioteca se reexplora
  automáticamente.

### Tabla

Cada fila es un shader: número, nombre, tipo, slot al que está asignado (si lo está) y
duración. Las filas se recorren con las flechas arriba/abajo.

### Panel de detalle

Al seleccionar un shader:

- **Previsualización** — el shader real ejecutado en WebGL sobre una ventana sintética,
  con puntero en movimiento y progreso en bucle. Si no compila, se muestra el error del
  compilador GLSL en lugar de una imagen.
- **Origen** — de dónde viene: **Biblioteca** (tu carpeta), **Umbriel** (un preset de tu
  configuración) u **Oficial** (el catálogo de la comunidad). Se muestran también la
  ruta y la función de entrada detectada.
- **Slot de destino** — a qué destino se aplicará.
- **Duración** y **Curva** — solo para slots de animación. La duración está en
  milisegundos; la curva es una de las que acepta Umbriel (por ejemplo `easeoutcubic`,
  `overshoot`).
- **Aplicar al slot** — escribe el cambio. `Ctrl+Enter` (o `Cmd+Enter`) hace lo mismo.
- **Probar** — reinicia la previsualización para ver la entrada desde el principio.
- **Quitar del slot** — aparece si el slot elegido ya tiene un shader; lo deja vacío.
- **Eliminar** — borra el shader de la biblioteca (pide confirmación). No afecta a la
  configuración de Umbriel, solo al archivo.

## Slots

La rejilla de **Slots** muestra todos los destinos de Umbriel y qué tiene cada uno:

- **Animaciones**: Vista general, Scratchpad, Animación de borde, Atenuar sin foco,
  Animación de capas, Ventanas al abrir/cerrar/mover, Escritorios, Diálogos.
- **Efectos**: Borde global, Superficie de ventana, Cursor, Pantalla.

Cada tarjeta indica el shader **Actual** (o **Sin asignar**), si el slot está
**Activo** o **Inactivo**, y ofrece:

- **Cambiar** — salta a la Biblioteca con ese slot ya seleccionado como destino.
- **Quitar animación** / **Quitar efecto** — deja el slot sin shader. En animaciones se
  desactiva el evento; en efectos se vuelve a «sin efecto». **Deshacer** lo recupera.

La cinta de orden de ejecución resume cuántos slots están activos y el total de
milisegundos de animación, útil para detectar una interfaz que se siente lenta.

## Familias

Agrupaciones de tus shaders locales por tipo de efecto. Cada familia lista sus miembros
y un botón **Aplicar a N slots** que asigna el shader a todos los destinos compatibles
en un solo cambio (una sola entrada de deshacer).

Sirve para uniformar la interfaz: por ejemplo, aplicar el mismo shader de ventana a
«Ventanas al abrir», «al cerrar» y «al mover». Los efectos que no encajan en el tipo no
se tocan.

## Ajustes

Cuatro bloques:

### Biblioteca de animaciones

- **Ruta de la carpeta** — dónde viven tus `.glsl`. Puede ser cualquier carpeta tuya.
- **Abrir carpeta** — la abre en el gestor de archivos.
- **Reexplorar** — vuelve a leer la carpeta. Úsalo tras editar o copiar shaders a mano.
- **Instalar ejemplos**, **Importar shader** — igual que en la Biblioteca.
- Contadores de **Shaders** y **Tamaño**, y la ruta del **Archivo que se edita** (el
  archivo de configuración de Umbriel que la app modificaría).

### Umbriel

- **Configuración de Umbriel** — ruta a tu `config.toml` original. Por defecto
  `~/.config/umbriel/config.toml`.
- **Validar configuración** — ejecuta `umbriel config validate` y muestra el resultado.
  Útil antes y después de tocar archivos a mano.

Recuerda que la app no edita `config.toml` directamente: detecta el archivo incluido que
define cada animación (`config.toml` o, por ejemplo, `src/animations.toml`) y reescribe
solo las claves de la animación aplicada.

### Interfaz

- **Idioma** — español o inglés; el cambio es inmediato.
- **Reducir movimiento** — detiene las previsualizaciones animadas y las transiciones de
  la app. Recomendado si te marea el movimiento o si la GPU va justa.

### Acerca de

Versión, rutas de datos y licencia.

Los cambios de Ajustes no se aplican hasta pulsar **Guardar cambios**; mientras haya
cambios pendientes verás el aviso **Cambios sin guardar**.

## Deshacer y Restaurar original

- **Deshacer** revierte el último cambio que la app hizo en tu configuración: escribe de
  vuelta los bytes exactos que había antes. Afecta al archivo y a las claves que tocó,
  nada más.
- **Restaurar original** revierte *todos* los cambios registrados por la app y elimina la
  copia prístina `config.toml.umbriel-shaders.bak`. Pide confirmación y no toca los
  shaders de tu biblioteca.

Las animaciones se aplican a la configuración viva de Umbriel, que vigila los archivos
incluidos: el efecto se ve al momento, sin reiniciar la sesión. Si algo sale mal, el
validador de Umbriel lo detecta antes de confirmar y la app deshace el intento.

## Atajos

| Tecla | Acción |
|---|---|
| `Ctrl+Enter` / `Cmd+Enter` | Aplicar el shader seleccionado al slot elegido (en Biblioteca) |
| `↑` / `↓` | Recorrer la tabla de shaders |
| `Esc` | Cerrar el diálogo o panel abierto |

Toda la interfaz es navegable con teclado: los botones muestran foco visible y los
estados no dependen solo del color.

## Flujo típico

1. **Biblioteca** → elige un shader → previsualízalo.
2. Ajusta **slot**, **duración** y **curva** → **Aplicar al slot**.
3. Mira el resultado en tu escritorio (Umbriel lo aplica en vivo).
4. ¿No te convence? Cambia de shader y vuelve a aplicar, o **Deshacer**.
5. Cuando termines de experimentar, deja la configuración que quieras y cierra la app:
   los cambios ya están escritos en disco.
