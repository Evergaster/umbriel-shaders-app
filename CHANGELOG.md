# Changelog

Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/);
versionado [SemVer](https://semver.org/lang/es/).

## [0.1.0] — 2026-11-10

Primera versión publicada.

### Añadido

- Biblioteca local de shaders `.glsl` con búsqueda, secciones por tipo y detalle con
  previsualización WebGL del shader real.
- Descarga del catálogo oficial de la comunidad con verificación SHA-256 por shader.
- Ocho shaders de ejemplo incluidos en la app.
- Aplicación de un shader a un slot con duración y curva, reescribiendo solo las claves
  de esa animación en el archivo de configuración correcto (`config.toml` o un incluido).
- Validación previa con `umbriel config validate` y escritura atómica del archivo.
- **Deshacer** por cambio y **Restaurar original** con los bytes exactos anteriores,
  más copia prístina `config.toml.umbriel-shaders.bak`.
- Vista **Slots** con todos los destinos de Umbriel, quitar animación/efecto y orden de
  ejecución.
- Vista **Familias**: aplicar un shader a todos los destinos compatibles de una vez.
- **Ajustes**: ruta de la biblioteca, configuración de Umbriel, validación, idioma
  (ES/EN) y reducción de movimiento.
- Instalador portable (`install.sh`), `Makefile`, entrada de menú e iconos; paquete
  `.tar.gz` con suma SHA-256.
- Documentación en español: instalación, uso, formato de shaders y desarrollo.

[0.1.0]: https://github.com/Evergaster/umbriel-shaders-app/releases/tag/v0.1.0
