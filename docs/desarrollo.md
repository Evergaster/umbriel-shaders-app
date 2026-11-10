# Desarrollo

Cómo está construida Umbriel Shaders, cómo ejecutarla sin instalar y qué tener en
cuenta antes de tocar el código.

## Requisitos

- Rust 1.77+ y Cargo.
- Cabeceras de GTK 3 y WebKitGTK 4.1 (`pkg-config --modversion webkit2gtk-4.1 gtk+-3.0`
  debe responder). Ver [`instalacion.md`](instalacion.md#dependencias-por-distribución).
- Opcional: Node 18+ si quieres usar el CLI de Tauri (`npm run dev`) y su recarga
  automática. Para `make build`, `make test` y `make install` **no** hace falta Node.
- `umbriel` en el `PATH` para probar el flujo de aplicación de verdad.

## Comandos

```sh
make build      # cargo build --release → src-tauri/target/release/umbriel-shaders
make run        # compila y ejecuta
make dev        # tauri dev (recarga de UI)
make test       # cargo test (12 pruebas)
make serve      # UI en el navegador contra el backend simulado, puerto 4173
make install    # instala en $PREFIX (por defecto ~/.local)
make package    # tarball + sha256 en dist/
make clean      # cargo clean y borra dist/
```

Prueba de instalación sin ensuciar tu `~/.local`:

```sh
PREFIX=$(mktemp -d) ./install.sh --prefix "$PREFIX"
"$PREFIX/bin/umbriel-shaders"
./install.sh --uninstall --prefix "$PREFIX"
```

## Estructura

```
.
├── DESIGN.md                    sistema visual y decisiones de diseño
├── PRODUCT.md                   alcance, principios y contexto
├── README.md / README.en.md     presentación (ES / EN)
├── Makefile                     build, test, install, package
├── install.sh                   instalador/desinstalador portable
├── umbriel-shaders.desktop      entrada de menú
├── docs/                        esta documentación
├── ui/                          interfaz (se incrusta en el binario)
│   ├── index.html               esqueleto: barra, navegación, contenedor de vista
│   ├── app.js                   estado, render de las cuatro vistas, eventos
│   ├── app.css                  estilos (tema oscuro, tipografía, layout)
│   ├── i18n.js                  diccionario ES/EN
│   ├── preview.js               previsualización WebGL del shader real
│   ├── mock.js                  backend simulado para trabajar en el navegador
│   ├── shaders/                 copia de los starters para `make serve`
│   └── fonts/                   Archivo Narrow y demás fuentes empaquetadas
└── src-tauri/
    ├── Cargo.toml
    ├── build.rs                 hook de reconstrucción de la UI
    ├── tauri.conf.json          ventana, identificador, bundle
    ├── capabilities/            permisos de ventana
    ├── icons/                   iconos de app
    ├── starters/*.glsl          ocho shaders de ejemplo (include_str!)
    └── src/
        ├── main.rs              arranque
        ├── lib.rs               comandos Tauri y registro de handlers
        ├── model.rs             tipos compartidos con JS (camelCase)
        ├── store.rs             rutas XDG, settings.json, state.json
        ├── scan.rs              biblioteca local, presets, catálogo oficial
        ├── locate.rs            detección del archivo de animaciones e includes
        ├── edit.rs              reescritura quirúrgica de TOML (toml_edit)
        ├── apply.rs             aplicar/quitar/deshacer/restaurar + validación
        └── util.rs              rutas, ejecución de procesos, escritura atómica
```

## Arquitectura

```
ui/ (WebView)                src-tauri/ (Rust)                 sistema
─────────────                ──────────────────                ──────
app.js ── invoke(cmd) ─────► lib.rs  #[tauri::command]
preview.js (WebGL)                │
                                  ├─ scan.rs ──► biblioteca ~/.local/share/…
                                  ├─ locate.rs ► config.toml + includes
                                  ├─ edit.rs  ─► toml_edit (solo claves del slot)
                                  ├─ apply.rs ─► umbriel config validate
                                  │              escritura atómica del archivo
                                  └─ store.rs ─► settings.json / state.json
```

- **Un solo sentido.** La UI no guarda estado propio de la configuración: llama a
  comandos y renderiza la instantánea (`Library`) que devuelve el backend. No añadas
  una segunda fuente de verdad en JS.
- **Escritura quirúrgica.** `edit.rs` parsea con `toml_edit`, que conserva comentarios,
  orden y formato. Nunca se regenera el archivo entero.
- **Validación antes de confirmar.** `apply.rs` escribe a un temporal, lanza
  `umbriel config validate -c <tmp>` y solo entonces renombra atómicamente. Si falla,
  borra el temporal y devuelve el error.
- **Reversible.** `state.json` guarda `Snapshot`s con los bytes `before`/`after` de cada
  archivo tocado; **Deshacer** los reescribe y **Restaurar original** lo hace en orden
  inverso.

### Flujo al aplicar un shader de la biblioteca

1. La UI llama `apply_shader({ slot, entryId, durationMs, curve })`.
2. Se copia el `.glsl` a `generated/shaders/us-<slug>.glsl`.
3. `locate.rs` decide qué archivo define el slot (raíz o incluido) y `edit.rs` inserta
   o actualiza `[effects.preset.us-<slug>]` (`kind`, `shader`, `radius`) y las claves del
   slot (`effect`, `duration_ms`, `curve`, `enabled`).
4. Se valida, se escribe y se registra el `Snapshot` en `state.json`.
5. El resultado vuelve a la UI con la ruta editada, que se muestra en la cinta y en
   Ajustes.

Umbriel vigila los archivos incluidos, así que no hay que notificar ni reiniciar: el
cambio se aplica en vivo en el compositor.

## Trampa: la UI va embebida

`frontendDist` (`../ui`) se incrusta en el binario en tiempo de compilación.
`tauri-build` solo emite `rerun-if-changed` para `tauri.conf.json`, capabilities e
iconos; por eso `src-tauri/build.rs` añade:

```rust
println!("cargo:rerun-if-changed=../ui");
```

Sin esa línea, editar solo la UI deja la interfaz vieja dentro del binario aunque cargo
diga «Finished». Si alguna vez un cambio de CSS o JS «no aparece», comprueba primero ese
hook.

Otra consecuencia: el binario se comprime con brotli dentro de
`target/<perfil>/build/<crate>-*/out/tauri-codegen-assets/*.js`, así que `strings` no
encuentra el texto. Para verificar que el embed se refrescó:

```sh
for f in src-tauri/target/release/build/umbriel-shaders-*/out/tauri-codegen-assets/*.js; do
    brotli -dc "$f" | grep -q "mi-marcador-nuevo" && echo "actualizado: $f"
done
```

## Frontend

- **`app.js`** — estado `S` (vista, selección, filtros, idioma, ajustes), `render()`
  por vista, delegación de eventos. Los manejadores se vuelven a enlazar en cada
  render; si añades un botón a una plantilla, enlázalo en el mismo sitio donde se
  enlazan los demás.
- **`preview.js`** — compila el shader del archivo sobre un preámbulo que replica el de
  Umbriel (mismos uniformes y ayudas) y lo dibuja sobre una ventana sintética. Si el
  shader no compila, informa por `onError`; no hay imagen falsa.
- **`i18n.js`** — todo texto visible pasa por `t("clave")`. Añadir una cadena implica
  añadirla en **es** y en **en**, o la interfaz mostrará la clave.
- **`mock.js`** — implementa cada comando con datos en memoria. Sirve para diseño y QA
  en el navegador (`make serve`); nunca se usa dentro de Tauri. Si añades un comando,
  impleméntalo también aquí o la UI de navegador se rompe.

## Backend

Convenciones:

- Los tipos serializados usan `#[serde(rename_all = "camelCase")]`: JS habla camelCase.
- Nuevos campos persistidos necesitan `#[serde(default)]` (o `default_*` si el valor por
  defecto es `true`), o `state.json`/`settings.json` antiguos dejan de cargar.
- Los mensajes de error que ve el usuario van en español, con la salida del comando
  externo incluida cuando la hay.
- Los archivos se escriben con `util::atomic_write` (temporal + rename), nunca a medias.
- Para limpiar un slot se escribe un valor explícito (desactivado / sin efecto) en lugar
  de borrar la clave: borrarla haría que Umbriel heredara la configuración original.

### Añadir un comando

1. Añade el tipo de petición/respuesta en `model.rs`.
2. Impleméntalo como función normal (lógica en `apply.rs`, `scan.rs`, etc.).
3. Envuelve con `#[tauri::command]` en `lib.rs` **y** regístralo en
   `generate_handler![...]`. Olvidar la lista compila y luego falla en tiempo de
   ejecución.
4. Llámalo desde `app.js` con `invoke("nombre", { arg })` y añádelo a `mock.js`.
5. Añade prueba unitaria si toca TOML o rutas.

## Pruebas

```sh
make test
```

Cubren: reescritura de TOML preservando comentarios y claves ajenas, inserción de
presets, detección del archivo de animaciones entre includes, escaneo de la biblioteca
y clasificación por tipo, y el comportamiento de los starters. Son pruebas de unidad;
no arrancan la ventana ni necesitan Umbriel.

Para probar el flujo completo hace falta `umbriel` instalado y una sesión en marcha:
aplica un shader de prueba y comprueba con `umbriel effects --json` que el slot cambió.

## Empaquetado y release

```sh
make package
# dist/umbriel-shaders-0.1.0-x86_64-linux.tar.gz
# dist/umbriel-shaders-0.1.0-x86_64-linux.tar.gz.sha256
```

El tarball contiene el binario, los iconos, el `.desktop`, `install.sh`, README y la
documentación. Para publicar:

```sh
gh release create v0.1.0 dist/umbriel-shaders-0.1.0-x86_64-linux.tar.gz* \
    --title "Umbriel Shaders 0.1.0" --notes-file CHANGELOG.md
```

Antes de publicar, comprueba en una máquina limpia que `./install.sh` funciona desde el
tarball (sin toolchain de Rust) y que la app abre desde el menú.

## Ideas pendientes

Ver la sección «Estado de esta versión» del [README](../README.md#estado-de-esta-versión):
reglas por aplicación, familias coordinadas del catálogo y edición de presets.
