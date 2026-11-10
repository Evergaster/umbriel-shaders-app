# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

Delivered as a Tauri 2 desktop program for Linux/Wayland: a web-rendered UI inside a native window. Not a browser extension, not a web page.

## Stack

Tauri 2 (Rust core + HTML/CSS/JS UI). Chosen by the user over Electron, GTK4/libadwaita and PySide6. UI ships bilingual: Spanish and English, switchable in Settings.

## Users

Umbriel users on Linux Wayland who want animated window effects (shaders) on their compositor without hand-editing `config.toml`. They are comfortable with a terminal but expect a visual tool for browsing, comparing and applying effects. Noctalia users are the existing audience; the program must not require Noctalia.

## Product Purpose

A standalone program that browses a local shader library, previews each shader live, applies it to Umbriel effect slots with timing, and keeps the animation files in a folder the user owns. Success: a user finds a shader, sees it, applies it to the right slot, and can undo or restore the original configuration without touching a config file.

## Positioning

The app owns a local, inspectable shader library (plain .glsl files on disk) and generates a separate Umbriel configuration that never rewrites the user's original. Applying, undoing and restoring are first-class, and the shader file itself drives a live WebGL preview before anything is applied.

## Operating Context

- Linux with a running Umbriel session and the `umbriel` command on PATH (`effects --json`, `config validate`, `config-replace`).
- XDG conventions: `$XDG_CONFIG_HOME/umbriel/config.toml` is the default original configuration.
- The local library is the source of truth; the app also reads the presets already defined in the original configuration.
- Shaders animate real surfaces: overview, scratchpad, borders, unfocused dimming, layer/panel transitions. Timing (duration, easing curve) is part of a selection.
- Users may have an existing configuration with presets that must keep precedence.

## Capabilities and Constraints

- Browse the local library with search and type filters; preview the actual shader file in WebGL.
- Apply one shader to a slot, or one shader to every compatible slot of a type.
- Clear a slot's shader in one action (animation events are disabled, effect
  slots return to "no effect"); Undo restores it.
- Override duration and curve on animation slots; apply timing with the selection.
- Undo the last change; restore the exact previous contents of every file the
  app edited, without touching anything else.
- Locate the animation file automatically: `config.toml` or whichever included
  file defines the slot, then rewrite only the keys of the applied animation.
  Never regenerate or replace whole configuration files; report the edited path.
- Local library: downloaded or imported shaders live in a user-configurable folder; the app lists, rescans and removes them.
- Download the official community catalogue from `noctalia-dev/community-umbriel-shaders` at its published revision, verifying each shader's SHA-256, and browse it in sections by effect kind.
- Settings: library path, Umbriel configuration path, language, reduced motion.
- Selections persist independently of the app; the user's own configuration and library files are the source of truth.
- Must work without Noctalia and without any helper executable beyond `umbriel`.
- Open decision: whether the program keeps the name "Umbriel Shaders".

## Brand Commitments

Existing name and identity: "Umbriel Shaders", MIT, author Barrulus. No logo or type system exists yet. The program is a redesign of the existing plugin's surface, so its workflows and terminology (slots, families, timing, Restore original) are binding.

## Evidence on Hand

- The built program: `/home/evergaster/proyectos/umbriel-shaders-app/` (Tauri 2, static UI, Rust backend).
- The existing plugin as behavioural spec: `/home/evergaster/umbriel-shaders/umbriel-shaders/` (panel, service, lib).
- Plugin guide with full workflow, settings and recovery text: `/home/evergaster/umbriel-shaders/umbriel-shaders/README.md`.
- Catalogue schema: `/home/evergaster/umbriel-shaders/catalog.toml`.
- Eight original starter shaders in `src-tauri/starters/`, also served to the browser preview in `ui/shaders/`.
- The official community catalogue is fetched at runtime from its published JSON; no community shader assets are committed.
- Screenshots of the four shipped views in `.impeccable/review/` and `docs/library.png`.

## Product Principles

1. The user's original configuration is never the app's to rewrite, except when
   the user explicitly opts into permanent changes (the original is kept as a
   backup and Restore original undoes the swap).
2. Every change is reversible in one action.
3. Show the effect, not a description of it: previews carry the decision.
4. The library is plain files the user can open, move and back up.
5. Terminal-grade truth, visual-grade surface: the app may look good, but it reports real state from `umbriel`.

## Accessibility & Inclusion

Keyboard-first operation for a desktop tool: every action reachable without a pointer, visible focus, and no colour-only state. Bilingual UI must not rely on untranslated strings.
