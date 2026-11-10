# Shaders

Un shader de Umbriel es un fragmento GLSL con una única función de entrada. La app no
inventa un formato: lee tus archivos `.glsl` tal cual y los ejecuta en la GPU, tanto en
la previsualización WebGL como, a través de Umbriel, en el compositor.

## Dónde vive un shader

Cada shader es **un archivo `.glsl`** dentro de la carpeta de la biblioteca
(por defecto `~/.local/share/umbriel-shaders/library/`, configurable en Ajustes). Se
pueden organizar en subcarpetas hasta cuatro niveles; la app recorre la carpeta y
recoge todos los `.glsl` (hasta 500 archivos).

El **nombre visible** es el nombre del archivo sin extensión (`aurora-veil.glsl` →
`aurora-veil`).

Añadir shaders:

- copiar archivos a la carpeta y pulsar **Reexplorar** en Ajustes;
- **Importar shader** en la Biblioteca o en Ajustes, indicando un archivo o una carpeta;
- **Descargar catálogo oficial**, que instala la colección de la comunidad;
- **Instalar ejemplos**, que copia los ocho shaders incluidos si faltan.

Eliminar un shader desde la app borra el archivo de la biblioteca (con confirmación).
Nunca toca la configuración de Umbriel.

## Cómo se detecta el tipo

El tipo de efecto se deduce del nombre del archivo y del contenido:

| Tipo | Señal |
|---|---|
| `cursor` | «cursor» en el nombre |
| `border` | «border» en el nombre, o el archivo define `vec4 border(vec2` |
| `screen` | «screen» en el nombre, o define `vec4 screen(vec2` |
| `window` | «window» en el nombre, o define `vec4 window(vec2` |
| `animation` | todo lo demás (valor por defecto) |

Escribir la función de entrada correcta es lo que garantiza que la app y Umbriel
coincidan; el nombre ayuda a clasificarlo en la biblioteca.

## Estructura de un shader

Solo escribes la función de entrada. El preámbulo —uniformes, funciones auxiliares y
`main()`— lo pone la app (y Umbriel), igual en ambos lados.

```glsl
vec4 animation(vec2 uv) {
    vec4 base = umbriel_sample(uv);
    float p = umbriel_clamped_progress;
    return mix(base, vec4(1.0), p);
}
```

Reglas:

- Nada de `#version`, `precision` ni `main()`: ya vienen incluidos.
- No declares `varying` propios: el UV llega como argumento (`uv`, en 0..1).
- Devuelve un `vec4` en espacio de color de la ventana (incluida alfa).
- GLSL ES 1.00: bucles con límites constantes, sin `texture()` (usa las ayudas), sin
  arrays dinámicos.
- `sin` y `cos` están redefinidos para aceptar cualquier ángulo (les aplican `mod 2π`).

### Funciones de entrada

| Función | Tipo | Qué se hace con el resultado |
|---|---|---|
| `vec4 animation(vec2 uv)` | Animación | Reemplaza el contenido durante la transición; `umbriel_progress` va de 0 a 1 |
| `vec4 border(vec2 uv)` | Borde | Color del borde; se recorta con `umbriel_border_distance` |
| `vec4 window(vec2 uv)` | Ventana | Se mezcla sobre la textura de la ventana con `umbriel_mask` |
| `vec4 cursor(vec2 uv)` | Cursor | Se mezcla sobre la ventana con `umbriel_mask`; hay historial del puntero |
| `vec4 screen(vec2 uv)` | Pantalla | Sustituye la pantalla completa; se dibuja tal cual |

## Uniformes

### Comunes a todos los tipos

| Uniforme | Tipo | Significado |
|---|---|---|
| `umbriel_size` | `vec2` | Tamaño de la superficie en píxeles |
| `umbriel_scale` | `float` | Escala del monitor (1.0 = sin escalado) |
| `umbriel_time` | `float` | Segundos desde que arrancó la superficie |
| `umbriel_pointer` | `vec2` | Posición del puntero en UV |
| `umbriel_expand` | `vec2` | Expansión de la superficie (sombras, bordes) |
| `umbriel_palette` | `vec4[4]` | Paleta del tema |
| `umbriel_palette_count` | `int` | Entradas válidas de la paleta (0..4) |

### Solo animaciones

| Uniforme | Tipo | Significado |
|---|---|---|
| `umbriel_progress` | `float` | Progreso de la animación (puede salirse de 0..1 en curvas con rebote) |
| `umbriel_clamped_progress` | `float` | `clamp(umbriel_progress, 0.0, 1.0)` |
| `umbriel_linear_progress` | `float` | Progreso sin curva aplicada |
| `umbriel_direction` | `float` | Dirección de la transición (1 entrada, -1 salida) |
| `umbriel_random_seed` | `vec4` | Semilla estable por transición |

### Solo bordes

| Uniforme | Tipo | Significado |
|---|---|---|
| `umbriel_border_hole` | `vec4` | Rectángulo interior: `xy` origen UV, `zw` tamaño UV |
| `umbriel_border_radius` | `vec4` | Radios de esquina (sup. izq., sup. der., inf. der., inf. izq.) |

### Solo ventanas y cursor

| Uniforme | Tipo | Significado |
|---|---|---|
| `umbriel_corner_radius` | `vec4` | Radios de esquina de la ventana |

### Solo cursor

| Uniforme | Tipo | Significado |
|---|---|---|
| `umbriel_pointer_count` | `int` | Puntos válidos del historial |
| `umbriel_pointer_history` | `vec4[8]` | Historial reciente del puntero |
| `umbriel_pointer_path` | `vec4[64]` | Trayectoria del puntero |

## Ayudas

| Función | Devuelve | Uso |
|---|---|---|
| `umbriel_sample(uv)` | `vec4` | Textura de la ventana (negro fuera de 0..1) |
| `umbriel_sample_incoming(uv)` | `vec4` | Textura de la ventana entrante |
| `umbriel_sample_previous(uv)` | `vec4` | Alias de la anterior (compatibilidad) |
| `umbriel_palette_at(t)` | `vec4` | Color interpolado de la paleta en `t` (0..1, cíclico) |
| `umbriel_border_distance(uv)` | `float` | Distancia con signo al borde (negativo dentro del hueco) |
| `umbriel_mask(uv)` | `float` | Máscara de esquinas redondeadas de la ventana |
| `umbriel_audio_available()` | `float` | Hoy devuelve `0.0` |
| `umbriel_audio_spectrum(p)` | `float` | Hoy devuelve `0.0` |

## Ejemplos

Fade suave con una pizca de desplazamiento:

```glsl
vec4 animation(vec2 uv) {
    float p = umbriel_clamped_progress;
    vec2 shift = vec2(0.0, 0.02 * (1.0 - p));
    vec4 base = umbriel_sample(uv + shift);
    return vec4(base.rgb, base.a * p);
}
```

Borde que respira con la paleta del tema:

```glsl
vec4 border(vec2 uv) {
    float d = abs(umbriel_border_distance(uv));
    float glow = smoothstep(6.0, 0.0, d);
    return mix(umbriel_palette_at(0.0), vec4(1.0), glow * 0.4);
}
```

Barrido de pantalla:

```glsl
vec4 screen(vec2 uv) {
    vec4 base = umbriel_sample(uv);
    float band = smoothstep(0.45, 0.5, uv.y) * (1.0 - smoothstep(0.5, 0.55, uv.y));
    return mix(base, vec4(1.0), band * 0.08);
}
```

## Probar un shader

1. Copia el `.glsl` a la biblioteca y pulsa **Reexplorar** (o usa **Importar shader**).
2. Selecciónalo en la Biblioteca: el panel derecho lo compila y lo ejecuta en WebGL
   sobre una ventana sintética.
3. **Probar** reinicia la animación desde el principio.
4. Si no compila, el panel muestra el error del compilador GLSL con su línea. Ese mismo
   error aparecería en Umbriel, así que corrígelo antes de aplicar.
5. Elige slot, duración y curva, y aplica.

La previsualización reproduce el preámbulo de Umbriel (mismos uniformes y ayudas), así
que un shader que se ve bien aquí se comporta igual en el compositor. Ten en cuenta que
la ventana sintética no es tu escritorio real: colores, tamaños y DPI cambian.

## Cómo se aplica por dentro

Al aplicar un shader de tu biblioteca, la app:

1. copia el archivo a `~/.local/share/umbriel-shaders/generated/shaders/us-<slug>.glsl`;
2. crea o actualiza el preset `[effects.preset.us-<slug>]` en el archivo de
   configuración de Umbriel que corresponda (con `kind` y `shader` apuntando a esa
   copia, en ruta absoluta);
3. apunta el slot a ese preset con `effect`, `duration_ms` y `curve`;
4. valida con `umbriel config validate` y solo entonces da el cambio por bueno.

Por eso, si editas el `.glsl` original después de aplicar, el efecto en Umbriel no
cambia hasta que vuelvas a aplicar. **Deshacer** y **Restaurar original** devuelven los
bytes exactos que había antes.

Los presets que ya existen en tu configuración (`source.umbriel`) se listan y se pueden
aplicar, pero la app no los edita.

## Consejos

- Evita bucles largos y `for` con límites variables: el fragmento se ejecuta por píxel
  en el compositor.
- Trabaja con `umbriel_clamped_progress` para transiciones y con `umbriel_time` para
  efectos continuos.
- Usa la paleta del tema (`umbriel_palette_at`) en vez de colores fijos si quieres que
  el efecto combine con el escritorio.
- Las animaciones deben terminar en un estado legible: en `progress = 1` el resultado
  tiene que ser la ventana ya asentada.
- Un shader = un archivo; mantén el nombre descriptivo, es el que verás en la app.
