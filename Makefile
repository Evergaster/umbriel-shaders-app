PREFIX  ?= $(HOME)/.local
CARGO   ?= cargo
APP     := umbriel-shaders
BIN     := src-tauri/target/release/$(APP)
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' src-tauri/Cargo.toml | head -1)
DIST    := dist/$(APP)-$(VERSION)-x86_64-linux

.PHONY: help build dev run serve test install uninstall package clean

help:
	@echo "make build      compila en modo release"
	@echo "make dev        Tauri dev con recarga"
	@echo "make run        compila y ejecuta el binario release"
	@echo "make serve      UI en el navegador con backend simulado (puerto 4173)"
	@echo "make test       cargo test"
	@echo "make install    instala en \$$PREFIX (por defecto ~/.local)"
	@echo "make uninstall  desinstala de \$$PREFIX"
	@echo "make package    tarball + sha256 en dist/"
	@echo "make clean      cargo clean y borra dist/"

build:
	cd src-tauri && $(CARGO) build --release

dev:
	npm run dev

run: build
	./$(BIN)

serve:
	npm run serve

test:
	cd src-tauri && $(CARGO) test

install: build
	./install.sh --no-build --prefix "$(PREFIX)"

uninstall:
	./install.sh --uninstall --prefix "$(PREFIX)"

package: build
	rm -rf "$(DIST)"
	mkdir -p "$(DIST)/docs"
	install -m 0755 "$(BIN)" "$(DIST)/$(APP)"
	cp -r src-tauri/icons "$(DIST)/icons"
	cp $(APP).desktop install.sh README.md README.en.md LICENSE "$(DIST)/"
	cp docs/*.md "$(DIST)/docs/"
	tar -C dist -czf "$(DIST).tar.gz" "$(notdir $(DIST))"
	cd dist && sha256sum "$(notdir $(DIST)).tar.gz" > "$(notdir $(DIST)).tar.gz.sha256"
	@echo "Paquete: $(DIST).tar.gz"

clean:
	cd src-tauri && $(CARGO) clean
	rm -rf dist
