# Instalación

Umbriel Shaders es una app de escritorio Tauri 2: un binario nativo que abre una
ventana GTK3 con la interfaz embebida. No necesita Node, Python ni ningún servicio en
segundo plano; en ejecución solo requiere GTK 3 y WebKitGTK 4.1.

Hay tres formas de instalarlo. La primera es la recomendada.

---

## 1. Instalador incluido (recomendado)

### Desde el repositorio

```sh
git clone https://github.com/Evergaster/umbriel-shaders-app.git
cd umbriel-shaders-app
./install.sh
```

El instalador:

1. compila en modo release si no hay binario (`src-tauri/target/release/umbriel-shaders`);
2. instala el binario en `~/.local/bin/umbriel-shaders`;
3. instala la entrada de menú en `~/.local/share/applications/umbriel-shaders.desktop`;
4. instala el icono en los tamaños 32, 128, 256 y 512 px bajo `~/.local/share/icons/hicolor/`;
5. copia esta documentación a `~/.local/share/doc/umbriel-shaders/`;
6. refresca la caché de iconos y la base de datos de escritorio si están disponibles.

No pide `sudo`. La primera compilación tarda varios minutos (Tauri y sus
dependencias); las siguientes son incrementales.

### Desde el tarball de una release

Si no quieres compilar, descarga `umbriel-shaders-<versión>-x86_64-linux.tar.gz` de la
[página de releases](https://github.com/Evergaster/umbriel-shaders-app/releases),
verifica el `.sha256` y ejecuta:

```sh
curl -fLO https://github.com/Evergaster/umbriel-shaders-app/releases/download/v0.1.0/umbriel-shaders-0.1.0-x86_64-linux.tar.gz
curl -fLO https://github.com/Evergaster/umbriel-shaders-app/releases/download/v0.1.0/umbriel-shaders-0.1.0-x86_64-linux.tar.gz.sha256
sha256sum -c umbriel-shaders-0.1.0-x86_64-linux.tar.gz.sha256
tar -xzf umbriel-shaders-0.1.0-x86_64-linux.tar.gz
cd umbriel-shaders-0.1.0-x86_64-linux
./install.sh
```

El tarball ya trae el binario: el instalador no compila nada.

### Opciones

```
./install.sh --prefix DIR    instala bajo DIR (por defecto $HOME/.local)
./install.sh --no-build      no compila aunque falte el binario
./install.sh --uninstall     quita binario, lanzador, iconos y documentación
```

También acepta la variable `PREFIX`. Para instalar para todo el sistema:

```sh
make build
sudo ./install.sh --prefix /usr/local
```

En ese caso el binario queda en `/usr/local/bin`, ya en el `PATH` de cualquier sesión.

### PATH

Si instalas en `~/.local` y `~/.local/bin` no está en tu `PATH`, el lanzador del menú
funciona igual (el campo `Exec` se escribe con la ruta absoluta), pero el comando
`umbriel-shaders` desde la terminal no. Añade a `~/.profile`:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

### Comprobar la instalación

```sh
command -v umbriel-shaders     # debe imprimir la ruta instalada
```

El lanzador del menú se llama **Umbriel Shaders**, categoría Utility/Graphics.

---

## 2. Compilar e instalar a mano

```sh
# 1. Dependencias de compilación (ver tabla por distribución)
# 2. Compilar
make build          # equivale a: cd src-tauri && cargo build --release
# 3. Copiar el binario donde quieras
install -Dm755 src-tauri/target/release/umbriel-shaders ~/.local/bin/umbriel-shaders
```

Si además quieres integración con el escritorio, copia `umbriel-shaders.desktop` a
`~/.local/share/applications/` (ajustando `Exec` a la ruta real) y los iconos de
`src-tauri/icons/` a `~/.local/share/icons/hicolor/<tamaño>/apps/umbriel-shaders.png`.

---

## 3. Ejecutar sin instalar

```sh
make run            # compila y ejecuta
# o directamente
./src-tauri/target/release/umbriel-shaders
```

Útil para probar una versión concreta; los datos siguen yendo a las rutas XDG de
siempre (ver [`README.md`](../README.md#dónde-están-tus-datos)).

---

## Dependencias por distribución

### Void Linux (plataforma de desarrollo de esta app)

```sh
# ejecución
sudo xbps-install -S gtk+3 libwebkit2gtk41 libsoup3 librsvg

# compilación
sudo xbps-install -S gtk+3-devel libwebkit2gtk41-devel libsoup3-devel \
    librsvg-devel pkg-config rust cargo
```

Es la combinación con la que se construyó y probó la versión 0.1.0. No hace falta
Node ni el CLI de Tauri para compilar con `cargo` o `make build`.

### Arch Linux y derivados

```sh
sudo pacman -S --needed gtk3 webkit2gtk-4.1 librsvg base-devel rust
```

### Debian / Ubuntu

```sh
sudo apt install libwebkit2gtk-4.1-dev build-essential pkg-config \
    libgtk-3-dev librsvg2-dev
```

### Fedora

```sh
sudo dnf install webkit2gtk4.1-devel gtk3-devel librsvg2-devel \
    pkg-config gcc gcc-c++ make rust cargo
```

Las secciones de Arch, Debian y Fedora son la lista estándar de dependencias de Tauri
2 en Linux; solo Void está verificada en este repositorio. Si tu distribución no
aparece, busca el paquete de desarrollo de `webkit2gtk-4.1` y `gtk+-3.0`: si
`pkg-config --modversion webkit2gtk-4.1 gtk+-3.0` responde, puedes compilar.

---

## Actualizar

```sh
cd umbriel-shaders-app
git pull
make install        # recompila e instala encima; los datos no se tocan
```

O descarga el tarball nuevo y ejecuta su `./install.sh`. Los ajustes, la biblioteca y
el historial de deshacer viven en `~/.config/umbriel-shaders` y
`~/.local/share/umbriel-shaders`; instalar otra versión no los modifica.

Si actualizas mientras la app está abierta, ciérrala antes: el binario en ejecución se
queda con el archivo anterior hasta que la reinicies.

---

## Desinstalar

```sh
./install.sh --uninstall            # desde ~/.local
make uninstall                      # equivalente
sudo ./install.sh --prefix /usr/local --uninstall
```

Esto borra el binario, el lanzador, los iconos y la documentación instalada. **No**
borra tus datos. Si quieres eliminarlos también:

```sh
rm -rf ~/.config/umbriel-shaders ~/.local/share/umbriel-shaders
```

Si activaste cambios permanentes y quieres dejar la configuración de Umbriel como
estaba antes de usar la app, pulsa **Restaurar original** dentro de la app *antes* de
desinstalar (o restaura a mano el `.bak`; ver más abajo).

---

## Problemas comunes

**`error while loading shared libraries: libwebkit2gtk-4.1.so.0`**
Falta el runtime de WebKitGTK. Instala `libwebkit2gtk41` (Void), `webkit2gtk-4.1`
(Arch), `libwebkit2gtk-4.1-0` (Debian/Ubuntu).

**Al compilar: `Package webkit2gtk-4.1 was not found in the pkg-config search path`**
Faltan las cabeceras de desarrollo: `libwebkit2gtk41-devel`, `webkit2gtk-4.1`,
`libwebkit2gtk-4.1-dev` o `webkit2gtk4.1-devel` según la distribución.

**La app abre pero dice «Umbriel no disponible»**
El binario `umbriel` no está en el `PATH` de la sesión gráfica. Comprueba en una
terminal de tu escritorio:

```sh
umbriel effects --json
```

Si falla ahí, la app tampoco puede hablar con Umbriel. Ajusta el `PATH` en el archivo
que lea tu sesión (`.profile`, `.config/environment.d/*.conf`, etc.) y vuelve a entrar
en la sesión.

**Aplicar un shader falla con «la configuración no valida»**
La app valida con `umbriel config validate` antes de confirmar. El mensaje incluye la
salida del validador. Revisa también Ajustes → **Validar configuración**, y que la ruta
de la configuración original apunte al archivo correcto.

**La previsualización está negra o muestra un error GLSL**
El shader no compila en el contexto WebGL de la app: el panel muestra el error del
compilador. Si *ningún* shader se previsualiza, el problema es WebGL en WebKitGTK
(drivers, `WEBKIT_DISABLE_COMPOSITING_MODE=1` como último recurso). Aplicar y
deshacer no necesitan WebGL.

**No aparece en el menú de aplicaciones**
Comprueba que `~/.local/share/applications` está en `$XDG_DATA_DIRS` y refresca:

```sh
update-desktop-database ~/.local/share/applications
```

Cierra y abre la sesión si tu shell de escritorio cachea el menú.

**El icono sale genérico en la barra de tareas**
La app declara `app_id = umbriel-shaders` en Wayland (`StartupWMClass` coincide en el
`.desktop`). Si usas X11, verifica con `xprop WM_CLASS` que coincide; en ese caso el
lanzador debe tener `StartupWMClass=umbriel-shaders`.

**Quiero volver atrás con la configuración de Umbriel**
La app mantiene una copia prístina en
`~/.config/umbriel/config.toml.umbriel-shaders.bak` mientras hay cambios permanentes
activos. **Restaurar original** en la app reescribe los archivos editados con los bytes
exactos anteriores y elimina el `.bak`. También puedes restaurarlo a mano copiándolo
sobre `config.toml` (haz copia antes).

---

## Qué instala exactamente

| Archivo | Contenido |
|---|---|
| `$PREFIX/bin/umbriel-shaders` | Binario release (≈4 MB, sin símbolos) |
| `$PREFIX/share/applications/umbriel-shaders.desktop` | Entrada de menú con `Exec` absoluto |
| `$PREFIX/share/icons/hicolor/{32x32,128x128,256x256,512x512}/apps/umbriel-shaders.png` | Iconos |
| `$PREFIX/share/doc/umbriel-shaders/` | README, licencia y estos documentos |

Ninguno de esos archivos lleva configuración personal: los datos se crean al primer
arranque en las rutas XDG descritas en el README.
