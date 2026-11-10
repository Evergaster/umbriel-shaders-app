---
version: 1
slug: "ui-index-html"
primary_target: "ui/index.html"
related_targets: []
---

# Surface brief — Biblioteca

Mode: Operate. Target: `ui/index.html`, the main window of the desktop program.

## Scope

The whole program shell: Biblioteca (shader library), Slots, Familias and Ajustes. The library is the first surface and sets the world; the other three inherit it.

## Audience and job

Umbriel users on Linux Wayland. They open the app to find an effect, see it running, and put it on the right slot without editing `config.toml`. Frequent, short sessions. Keyboard and pointer both matter.

## Direction contract

THESIS: A working shader library that behaves like a printed running order — every effect has an index, a duration and a slot, and the timeline at the bottom is the page's spine. It refuses the category's card-grid gallery and the dark-glass dashboard.

OWN-WORLD: Flat black paper (#0A0A0A), bone ink (#F1EFE8), one acid ink (#D8FF00) reserved for selection, primary action and live state; hairline rules (#262622); condensed grotesk for names and controls, tabular mono for numbers; zero radius, no shadow, no gradient except inside the shader preview itself.

STORY: Scan the order, pick a shader, watch it run on a window, apply it to a slot, undo or restore. The ruler teaches that shaders are timed, not just toggled.

FIRST VIEWPORT: Top bar with wordmark and four sections; a metadata tape; a dense five-column table (No., shader, type, slot, duration) over two thirds; a 480 px preview column with a live shader on a window and an inspector; a 132 px running-order ruler spanning the full width, the previewed slot inverted in acid.

FORM: Demoscene party print / running order. Position 7 of 7 grounded directions; seed key a6f96e1d6fd847e5ad851036d3a4ffdf.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance.

## Constraints

- The original Umbriel configuration is never rewritten.
- Every apply validates the generated config before `config-replace`.
- Undo and restore are one action away, always.
- Bilingual ES/EN, keyboard-first, no colour-only state.
- Preview must show the real shader file, not a stand-in.

## Unresolved

- Per-application window rules are not in this version.
- Families are local groups by effect kind, not the plugin's curated coordinated sets.
