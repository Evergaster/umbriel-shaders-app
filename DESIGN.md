---
name: Umbriel Shaders
description: A local shader library and running order for Umbriel effect slots.
colors:
  ground: "#0A0A0A"
  panel: "#111110"
  panel-raised: "#0D0D0C"
  ink: "#F1EFE8"
  ink-muted: "#7C7C74"
  ink-faint: "#8A8A80"
  rule: "#262622"
  acid: "#D8FF00"
  acid-hover: "#E4FF4D"
  on-acid: "#151500"
  danger: "#FF5A45"
  danger-deep: "#3A201C"
  on-danger: "#160300"
typography:
  display:
    fontFamily: "Archivo Narrow, Liberation Sans Narrow, sans-serif"
    fontSize: "46px"
    fontWeight: 700
    lineHeight: 0.85
    letterSpacing: "0.005em"
  headline:
    fontFamily: "Archivo Narrow, Liberation Sans Narrow, sans-serif"
    fontSize: "40px"
    fontWeight: 700
    lineHeight: 0.95
    letterSpacing: "0.005em"
  title:
    fontFamily: "Archivo Narrow, Liberation Sans Narrow, sans-serif"
    fontSize: "30px"
    fontWeight: 700
    lineHeight: 0.95
    letterSpacing: "0.005em"
  brand:
    fontFamily: "Archivo Narrow, Liberation Sans Narrow, sans-serif"
    fontSize: "26px"
    fontWeight: 700
    lineHeight: 0.9
    letterSpacing: "0.01em"
  section:
    fontFamily: "Archivo Narrow, Liberation Sans Narrow, sans-serif"
    fontSize: "21px"
    fontWeight: 400
    lineHeight: 1.05
    letterSpacing: "0.01em"
  row:
    fontFamily: "Archivo Narrow, Liberation Sans Narrow, sans-serif"
    fontSize: "19px"
    fontWeight: 400
    lineHeight: 1.1
    letterSpacing: "0.01em"
  body:
    fontFamily: "Archivo Narrow, Liberation Sans Narrow, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.35
    letterSpacing: "0em"
  small:
    fontFamily: "Archivo Narrow, Liberation Sans Narrow, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "0em"
  label:
    fontFamily: "JetBrains Mono, Iosevka, monospace"
    fontSize: "11px"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "0.14em"
rounded:
  none: "0px"
spacing:
  xs: "6px"
  sm: "8px"
  md: "12px"
  lg: "18px"
  xl: "26px"
components:
  button-primary:
    backgroundColor: "{colors.acid}"
    textColor: "{colors.ground}"
    rounded: "{rounded.none}"
    padding: "10px 16px"
  button-ghost:
    backgroundColor: "{colors.panel-raised}"
    textColor: "{colors.ink}"
    rounded: "{rounded.none}"
    padding: "10px 16px"
  nav-active:
    backgroundColor: "{colors.acid}"
    textColor: "{colors.ground}"
    rounded: "{rounded.none}"
    padding: "9px 16px"
  filter-chip:
    backgroundColor: "{colors.panel-raised}"
    textColor: "{colors.ink-muted}"
    rounded: "{rounded.none}"
    padding: "5px 9px"
  filter-chip-active:
    backgroundColor: "{colors.ink}"
    textColor: "{colors.ground}"
    rounded: "{rounded.none}"
    padding: "5px 9px"
  field:
    backgroundColor: "{colors.panel-raised}"
    textColor: "{colors.ink}"
    rounded: "{rounded.none}"
    height: "32px"
    padding: "0 10px"
---

# Design System: Umbriel Shaders

## Overview

**Creative North Star: "The Running Order"**

Umbriel Shaders looks like the printed running order of a demoscene party, repurposed as an instrument. Every shader is an entry with a number, a type, a destination and a duration; the timeline along the bottom is the page's spine. The interface is dense but quiet: one flat black field, hairline rules, and a single acid ink that only ever marks what is selected, live or actionable.

The system refuses two category defaults: the card-grid shader gallery and the dark-glass dashboard. There are no floating panels, no rounded tiles, no second accent, and no glow outside the preview. The shader preview is the one place where the product's subject — light moving on a surface — is allowed to be lush; everything around it stays printed, factual and legible.

**Key Characteristics:**
- Flat black ground with 1px rules doing all the structural work.
- One acid ink, reserved for selection, primary action and live state.
- Condensed grotesk for names and controls; tabular mono for every number.
- Zero radius and zero elevation; the only shadow belongs to the preview window.
- A proportional running-order ruler that makes timing visible.

## Colors

The palette is a printed programme under stage light: bone ink on black paper, one fluorescent spot.

### Primary
- **Acid Ink** (#D8FF00): the only accent. It fills the selected table row, the active nav item, the primary button, the live-status dot and the active timeline segment. It never appears as body text, a border, a heading or a decorative wash.
- **Acid Hover** (#E4FF4D): the primary button's hover state only.
- **On Acid** (#151500): text and marks that sit on an acid field. Never pure black; it keeps the printed-ink warmth.

### Neutral
- **Paper Black** (#0A0A0A): the application ground and the space between table rows.
- **Panel Black** (#111110): the preview/inspector column and cards, one step above the ground.
- **Raised Black** (#0D0D0C): the titlebar, metadata tape, inputs and chips — recessed rather than raised.
- **Bone Ink** (#F1EFE8): primary text, headings and the active chip field.
- **Muted Ink** (#7C7C74): secondary labels, column headers and inactive nav.
- **Faint Ink** (#8A8A80): placeholders, ticks and disabled text; still above 4.5:1 on the ground.
- **Rule** (#262622): every hairline, divider and table border.

### Tertiary
- **Warning Red** (#FF5A45): destructive actions only — restore original, remove shader, errors. Never used for status or decoration.
- **Danger Deep** (#3A201C): the 1px border on a danger button.
- **On Danger** (#160300): text on a danger field.

### Named Rules
**The One Ink Rule.** Acid is a field, not a tint. If it is not selected, primary or live, it does not get the acid.

**The Hairline Rule.** Structure comes from 1px rules and flat tonal steps. No card shadows, no glass, no gradient panels.

## Typography

**Display Font:** Archivo Narrow (with Liberation Sans Narrow, sans-serif)
**Body Font:** Archivo Narrow (same stack)
**Label/Mono Font:** JetBrains Mono (with Iosevka, monospace)

**Character:** A condensed grotesk that behaves like letterpress programme type, paired with a mono that carries every measurement. The display voice is loud and tight; the mono is quiet and exact.

### Hierarchy
- **Display** (700, 46px, 0.85 line-height, 0.005em tracking): page titles — Slots, Familias, Ajustes.
- **Headline** (700, 40px, 0.95): the empty-library state.
- **Title** (700, 30px, 0.95): the selected shader name in the inspector and dialog titles.
- **Brand** (700, 26px, 0.9): the two-line wordmark only.
- **Section** (400, 21px, 1.05): slot card names, settings section headings, stat numbers.
- **Row** (400, 19px, 1.1): shader names in the table and settings section titles; 700 when the row is selected.
- **Body** (400, 14px, 1.35): descriptions, slot names, table cells, buttons.
- **Small** (400, 13px, 1.4): toast copy, settings descriptions, small buttons, path inputs.
- **Label** (400, 11px mono, 0.14em tracking, uppercase): column headers, durations, paths, counts, filter chips, the metadata tape. Never below 11px for functional text.

### Named Rules
**The Tabular Rule.** Every number is mono, and the number that ends a row is right-aligned. Durations, counts, hashes and paths never use the display face.

**The No Kicker Rule.** A page title stands alone. No eyebrow, no section number, no label above a heading.

## Layout

A fixed desktop shell: 34px titlebar, 74px top bar, 30px metadata tape, then the view. The library is a two-column grid — a flexible table and a 480px inspector — over a 118px running-order ruler that spans both columns. The table has five columns (number, shader, type, slot, duration); the inspector holds a 280px preview and a form.

Other views inherit the shell. Slots and families use a two-column grid of 1px-separated cards inside a 26px page gutter. Settings is a single 980px column with a 220px label column per section.

Breakpoints: the inspector narrows to 420px at 1240px and 380px at 1080px; filter chips beyond the fourth hide at 1080px; short windows (840px and under) shrink the preview to 200px and tighten the inspector so the primary action stays visible. The window's minimum is 1080×680.

Spacing follows a 6/8/12/18/26 rhythm. Sections are separated by a hairline and 22px of air; more space sits above a heading than below it.

## Elevation & Depth

The system is flat by default. Depth is tonal, not shadowed: panel black sits one step above paper black, raised black sits one step below. The only shadow in the product belongs to the preview window (`0 24px 70px rgba(0,0,0,.55)`), because it depicts a real window on a desktop. Its acid corner brackets are the one ornament in the system.

**The Flat-By-Default Rule.** A surface earns a shadow only by depicting something physical. Panels, cards, menus and toasts stay flat.

## Shapes

Square. Every corner is 0px: buttons, chips, fields, cards, rows, the preview window, the dialog. Borders are always 1px in Rule. The only curves in the interface are inside the shader canvas and the circular live-status dot. The preview window carries 15px acid L-brackets at its top-left and bottom-right corners as a framing device.

**The Zero Radius Rule.** If a corner is rounded, it is not this system. Curves belong to the shader, not the chrome.

## Components

### Buttons
- **Shape:** square, 1px border, 10px 16px padding, uppercase 14px display face.
- **Primary:** Acid Ink field, black text, 700 weight. One per view; it is the action the view exists for.
- **Hover / Focus:** primary lightens to #E4FF4D; ghost borders move from Rule to Faint Ink; focus is a 2px acid outline with 2px offset.
- **Ghost:** transparent field, 1px Rule border, bone text. Used for secondary actions and settings commands.
- **Danger:** transparent field, Warning Red text and a dark red border. Used only for remove and restore.

### Chips
- **Style:** 11px mono uppercase, 1px Rule border, Raised Black field, muted text.
- **State:** the active filter inverts to bone ink with black text; the inactive chip keeps the muted label.

### Table Rows
- **Style:** 52px tall, five-column grid, 1px Rule divider, no background at rest.
- **Hover:** #131312, one step above ground.
- **Selected:** the entire row becomes an Acid Ink field with black type and a small black triangle marker; assigned shaders carry a 5px acid dot (black when selected).

### Fields
- **Style:** 32px tall, Raised Black field, 1px Rule border, 11px mono text, no radius.
- **Focus:** the border turns Acid Ink; the caret is acid.
- **Labels:** 11px mono uppercase, muted ink, 5px above the field.

### Slot Cards
- **Style:** flat panel black, 1px separators from the grid, 18px internal padding, 21px slot name, 11px kind label at the far right.
- **Current assignment:** acid mono text; unassigned reads "Sin asignar" in Faint Ink.
- **Footer:** duration and curve in mono on the left, a text-link "Cambiar" on the right.

### Running-Order Ruler
- **Style:** a 52px bordered track whose segments are positioned by real duration, so their widths are proportional. The previewed slot's segment is an Acid Ink field; the rest are paper black with 1px dividers. A 12px tick row below carries the millisecond scale.
- **Behaviour:** it is the signature component and the product's argument — shaders are timed, not toggled.

### Preview Window
- **Style:** a 16:10 window frame with a 24px title bar, the shader canvas filling the screen area, a 1px #3A3A33 border, a deep soft shadow and acid corner brackets. The shader itself supplies all colour inside the frame.

## Do's and Don'ts

### Do:
- **Do** invert a selected row, nav item or timeline segment completely (acid field, black type) instead of tinting it.
- **Do** keep the ruler proportional to the real durations, and show the total in milliseconds.
- **Do** keep every functional label at 11px or larger, even in the metadata tape and table headers.
- **Do** put paths, durations, counts and curve names in the mono face.
- **Do** let the shader preview be the only vivid surface in the window.

### Don't:
- **Don't** add a second accent colour; Warning Red is for destruction only.
- **Don't** round corners, add drop shadows to panels, or use glass and blur as decoration.
- **Don't** put an eyebrow or kicker above a page title.
- **Don't** use gradients, glows or coloured halos outside the shader canvas.
- **Don't** use acid for body copy, borders or headings; it is a field colour.
