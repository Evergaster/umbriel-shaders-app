#!/bin/sh
# Instalador de Umbriel Shaders.
#
#   ./install.sh                 compila si hace falta e instala en ~/.local
#   ./install.sh --prefix /usr   instala en /usr (requiere permisos)
#   ./install.sh --no-build      usa el binario ya compilado / incluido
#   ./install.sh --uninstall     quita lo instalado
#
# Funciona tanto en un clon del repositorio (compila con cargo) como en el
# tarball de una release (binario ya incluido).

set -eu

APP=umbriel-shaders
PREFIX=${PREFIX:-$HOME/.local}
BUILD=1
UNINSTALL=0

usage() {
    cat <<EOF
Uso: ./install.sh [opciones]

Opciones:
  --prefix DIR    prefijo de instalación (por defecto: \$HOME/.local)
  --no-build      no compilar; usa el binario disponible
  --uninstall     desinstalar
  -h, --help      mostrar esta ayuda

Variables de entorno:
  PREFIX          mismo efecto que --prefix
  CARGO           binario de cargo a usar (por defecto: cargo)
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --prefix)   [ $# -ge 2 ] || { echo "Falta el valor de --prefix" >&2; exit 1; }
                    PREFIX=$2; shift 2 ;;
        --prefix=*) PREFIX=${1#*=}; shift ;;
        --no-build) BUILD=0; shift ;;
        --uninstall) UNINSTALL=1; shift ;;
        -h|--help)  usage; exit 0 ;;
        *)          echo "Opción desconocida: $1" >&2; usage >&2; exit 1 ;;
    esac
done

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
BIN_DIR="$PREFIX/bin"
APPS_DIR="$PREFIX/share/applications"
ICONS_DIR="$PREFIX/share/icons/hicolor"
DOCS_DIR="$PREFIX/share/doc/$APP"

refresh_caches() {
    [ -d "$PREFIX/share/icons/hicolor" ] && command -v gtk-update-icon-cache >/dev/null 2>&1 \
        && gtk-update-icon-cache -q -t "$PREFIX/share/icons/hicolor" 2>/dev/null || true
    [ -d "$APPS_DIR" ] && command -v update-desktop-database >/dev/null 2>&1 \
        && update-desktop-database -q "$APPS_DIR" 2>/dev/null || true
}

if [ "$UNINSTALL" -eq 1 ]; then
    rm -f "$BIN_DIR/$APP" "$APPS_DIR/$APP.desktop"
    for size in 32x32 128x128 256x256 512x512; do
        rm -f "$ICONS_DIR/${size}/apps/$APP.png"
    done
    rm -rf "$DOCS_DIR"
    refresh_caches
    echo "Umbriel Shaders desinstalado de $PREFIX"
    exit 0
fi

# --- Localizar o compilar el binario -----------------------------------------

BIN=""
if [ -x "$ROOT/$APP" ]; then
    BIN="$ROOT/$APP"                      # tarball de release
elif [ -x "$ROOT/src-tauri/target/release/$APP" ]; then
    BIN="$ROOT/src-tauri/target/release/$APP"
fi

if [ -z "$BIN" ] && [ "$BUILD" -eq 1 ]; then
    echo "Compilando en modo release (la primera vez tarda varios minutos)..."
    ( cd "$ROOT/src-tauri" && "${CARGO:-cargo}" build --release )
    BIN="$ROOT/src-tauri/target/release/$APP"
fi

if [ ! -x "$BIN" ]; then
    echo "No hay binario que instalar. Compílalo con: cd src-tauri && cargo build --release" >&2
    exit 1
fi

# --- Fuentes de los archivos auxiliares --------------------------------------

ICON_SRC=""
for candidate in "$ROOT/icons" "$ROOT/src-tauri/icons"; do
    [ -d "$candidate" ] && ICON_SRC="$candidate" && break
done

DESKTOP_SRC="$ROOT/$APP.desktop"

# --- Instalación --------------------------------------------------------------

echo "Instalando en $PREFIX"
install -d "$BIN_DIR" "$APPS_DIR" "$DOCS_DIR"
install -m 0755 "$BIN" "$BIN_DIR/$APP"

if [ -n "$ICON_SRC" ]; then
    for pair in "32x32.png:32x32" "128x128.png:128x128" "128x128@2x.png:256x256" "icon.png:512x512"; do
        src=$ICON_SRC/${pair%%:*}
        size=${pair##*:}
        [ -f "$src" ] || continue
        install -d "$ICONS_DIR/$size/apps"
        install -m 0644 "$src" "$ICONS_DIR/$size/apps/$APP.png"
    done
fi

if [ -f "$DESKTOP_SRC" ]; then
    # Exec absoluto: el lanzador funciona aunque ~/.local/bin no esté en PATH.
    sed -e "s|^Exec=.*|Exec=$BIN_DIR/$APP|" \
        -e "s|^TryExec=.*|TryExec=$BIN_DIR/$APP|" \
        "$DESKTOP_SRC" > "$DESKTOP_SRC.$$.tmp"
    install -m 0644 "$DESKTOP_SRC.$$.tmp" "$APPS_DIR/$APP.desktop"
    rm -f "$DESKTOP_SRC.$$.tmp"
else
    echo "Aviso: no se encontró $APP.desktop; se instala solo el binario." >&2
fi

for doc in README.md LICENSE; do
    [ -f "$ROOT/$doc" ] && install -m 0644 "$ROOT/$doc" "$DOCS_DIR/$doc"
done
if [ -d "$ROOT/docs" ]; then
    install -d "$DOCS_DIR/docs"
    for doc in "$ROOT"/docs/*.md; do
        [ -f "$doc" ] && install -m 0644 "$doc" "$DOCS_DIR/docs/"
    done
fi

refresh_caches

echo
echo "Listo. Binario: $BIN_DIR/$APP"
echo "Lanzador:      $APPS_DIR/$APP.desktop"
case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) echo
       echo "Aviso: $BIN_DIR no está en tu PATH."
       echo "Añádelo a tu shell (por ejemplo en ~/.profile) para lanzar '$APP' desde la terminal:" >&2
       echo "  export PATH=\"\$HOME/.local/bin:\$PATH\"" >&2 ;;
esac
