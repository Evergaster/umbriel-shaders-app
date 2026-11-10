// Browser-only development backend. The Tauri build never loads this file.
const names = [
  ["aurora-veil", "animation"],
  ["paper-fade", "animation"],
  ["focus-ring-pulse", "border"],
  ["desaturate-soft", "window"],
  ["panel-glide", "animation"],
  ["caustic-drift", "window"],
  ["sodium-haze", "cursor"],
  ["scanline-memory", "screen"],
];

let settings = {
  libraryPath: "~/.local/share/umbriel-shaders/library",
  originalConfig: "~/.config/umbriel/config.toml",
  language: "es",
  reduceMotion: false,
};

const ANIMATION_FILE = "~/.config/umbriel/src/animations.toml";

const entries = names.map(([name, kind], index) => ({
  id: `lib:${name}`,
  name,
  kind,
  source: "library",
  path: `~/.local/share/umbriel-shaders/library/${name}/${name}.glsl`,
  size: 620 + index * 37,
  modified: 1760000000 + index,
  entryFn: kind === "border" ? "border" : kind === "window" ? "window" : kind === "cursor" ? "cursor" : kind === "screen" ? "screen" : "animation",
  usedBy: [],
  official: false,
  description: "",
  author: "",
}));

const slots = {
  "animation.overview": { key: "animation.overview", kind: "animation", effect: "aurora-veil", durationMs: 320, curve: "easeoutcubic", enabled: true },
  "animation.scratchpad": { key: "animation.scratchpad", kind: "animation", effect: "paper-fade", durationMs: 180, curve: "easeoutcubic", enabled: true },
  "animation.border": { key: "animation.border", kind: "animation", effect: "", durationMs: 900, curve: "easeout", enabled: false },
  "animation.dim_unfocused": { key: "animation.dim_unfocused", kind: "animation", effect: "", durationMs: 250, curve: "easeout", enabled: false },
  "animation.layers": { key: "animation.layers", kind: "animation", effect: "panel-glide", durationMs: 260, curve: "easeoutcubic", enabled: true },
  "animation.windows_in": { key: "animation.windows_in", kind: "animation", effect: "", durationMs: 500, curve: "easeoutcubic", enabled: true },
  "animation.windows_out": { key: "animation.windows_out", kind: "animation", effect: "", durationMs: 500, curve: "easeoutcubic", enabled: true },
  "animation.windows_move": { key: "animation.windows_move", kind: "animation", effect: "", durationMs: 300, curve: "easeout", enabled: true },
  "animation.workspaces": { key: "animation.workspaces", kind: "animation", effect: "", durationMs: 300, curve: "easeout", enabled: true },
  "animation.dialogs": { key: "animation.dialogs", kind: "animation", effect: "", durationMs: 250, curve: "easeout", enabled: true },
  "effects.border": { key: "effects.border", kind: "border", effect: "focus-ring-pulse", durationMs: null, curve: null, enabled: true },
  "effects.window": { key: "effects.window", kind: "window", effect: "caustic-drift", durationMs: null, curve: null, enabled: true },
  "effects.cursor": { key: "effects.cursor", kind: "cursor", effect: "sodium-haze", durationMs: null, curve: null, enabled: true },
  "effects.screen": { key: "effects.screen", kind: "screen", effect: "scanline-memory", durationMs: null, curve: null, enabled: true },
};

let history = [];
let connected = true;

function usedBy(name) {
  return Object.values(slots).filter((s) => s.effect === name).map((s) => s.key);
}

function library() {
  const list = entries.map((e) => ({ ...e, usedBy: usedBy(e.name) }));
  return {
    entries: list,
    slots: Object.values(slots),
    libraryPath: settings.libraryPath,
    originalConfig: settings.originalConfig,
    libraryBytes: entries.reduce((sum, e) => sum + e.size, 0),
    generated: ANIMATION_FILE,
    canUndo: history.length > 0,
  };
}

export async function mockInvoke(command, args = {}) {
  switch (command) {
    case "settings_get":
      return { ...settings };
    case "settings_save":
      settings = { ...settings, ...args.settings };
      return { ...settings };
    case "library_scan":
      return library();
    case "library_install_shader":
      return null;
    case "catalogue_info":
      return null;
    case "catalogue_save":
      return {
        revision: "0000000000000000000000000000000000000000",
        total: 169,
        kinds: { animation: 66, border: 25, cursor: 29, screen: 3, window: 46 },
      };
    case "library_install_starters":
      return 0;
    case "library_import":
      return 1;
    case "library_remove":
      return null;
    case "library_reveal":
      return null;
    case "shader_source": {
      const entry = entries.find((e) => e.id === args.id);
      if (!entry) throw new Error("Shader no encontrado.");
      const response = await fetch(`shaders/${entry.name}.glsl`);
      if (!response.ok) throw new Error("No se pudo leer el shader.");
      return {
        path: entry.path,
        source: await response.text(),
        entryFn: entry.entryFn,
        kind: entry.kind,
        name: entry.name,
      };
    }
    case "umbriel_status":
      return {
        connected,
        compiled: 3,
        presets: [
          { name: "voronoi-open", kind: "animation", state: "unreferenced" },
          { name: "nier-open", kind: "animation", state: "compiled" },
          { name: "glow-naranja", kind: "cursor", state: "compiled" },
        ],
        error: connected ? null : "Umbriel no está disponible.",
      };
    case "apply_shader": {
      const entry = entries.find((e) => e.id === args.req.entryId);
      if (!entry) throw new Error("Shader no encontrado.");
      const slot = slots[args.req.slot];
      if (!slot) throw new Error("Slot desconocido.");
      if (slot.kind !== entry.kind) throw new Error("Ese shader no es compatible con el destino.");
      history.push(JSON.parse(JSON.stringify(slots)));
      slot.effect = entry.name;
      slot.enabled = true;
      if (slot.kind === "animation") {
        slot.durationMs = args.req.durationMs || 280;
        slot.curve = args.req.curve || "easeoutcubic";
      }
      return { ok: true, message: `${entry.name} aplicado a ${args.req.slot} en ${ANIMATION_FILE}`, generated: ANIMATION_FILE };
    }
    case "apply_family": {
      const entry = entries.find((e) => e.id === args.entryId);
      if (!entry) throw new Error("Shader no encontrado.");
      const targets = Object.values(slots).filter((slot) => slot.kind === args.kind);
      history.push(JSON.parse(JSON.stringify(slots)));
      for (const slot of targets) {
        slot.effect = entry.name;
        slot.enabled = true;
        if (slot.kind === "animation") {
          slot.durationMs = slot.durationMs || 280;
          slot.curve = slot.curve || "easeoutcubic";
        }
      }
      return { ok: true, message: `${entry.name} aplicado a ${targets.length} destinos`, generated: ANIMATION_FILE };
    }
    case "clear_slot": {
      const slot = slots[args.slot];
      if (!slot) throw new Error("Slot desconocido.");
      history.push(JSON.parse(JSON.stringify(slots)));
      slot.effect = "";
      slot.enabled = false;
      slot.durationMs = null;
      slot.curve = null;
      return { ok: true, message: `${args.slot} sin efecto`, generated: ANIMATION_FILE };
    }
    case "undo_change": {
      const previous = history.pop();
      if (!previous) throw new Error("No hay cambios que deshacer.");
      for (const key of Object.keys(slots)) Object.assign(slots[key], previous[key]);
      return { ok: true, message: "Deshecho", generated: "" };
    }
    case "restore_original": {
      history = [];
      for (const slot of Object.values(slots)) {
        slot.effect = "";
        slot.enabled = true;
        if (slot.kind === "animation") slot.durationMs = 250;
      }
      return { ok: true, message: "Configuración original restaurada.", generated: settings.originalConfig };
    }
    case "validate_config":
      return { ok: true, message: "Configuración válida.", generated: "" };
    default:
      throw new Error(`Comando desconocido: ${command}`);
  }
}
