use crate::edit::{self, Preset, SlotEdit};
use crate::locate;
use crate::model::{
    slot_kind, ActionResult, ApplyReq, Entry, FileChange, Settings, SlotSel, Snapshot, State,
    CURVES, SLOTS,
};
use crate::store::{generated_dir, load_state, save_state};
use crate::util::{atomic_write, now_ms, run, slug};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn shader_dir() -> PathBuf {
    generated_dir().join("shaders")
}

fn shader_path(name: &str) -> String {
    shader_dir()
        .join(format!("{name}.glsl"))
        .to_string_lossy()
        .to_string()
}

fn validate(path: &str) -> Result<(), String> {
    let (code, out, err) = run("umbriel", &["config", "validate", "-c", path])?;
    if code == 0 {
        return Ok(());
    }
    let detail = if err.trim().is_empty() { out } else { err };
    Err(detail.trim().to_string())
}

fn path_label(changes: &[FileChange], root: &Path) -> String {
    if changes.is_empty() {
        return root.to_string_lossy().to_string();
    }
    changes
        .iter()
        .map(|change| change.path.clone())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Copies a library shader into the generated shader folder and returns the
/// preset name Umbriel should reference.
fn prepare_preset(entry: &Entry) -> Result<String, String> {
    if entry.source != "library" {
        return Ok(entry.name.clone());
    }
    let preset = format!("us-{}", slug(&entry.name));
    let dir = shader_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::copy(&entry.path, dir.join(format!("{preset}.glsl"))).map_err(|e| e.to_string())?;
    Ok(preset)
}

struct PresetOwned {
    name: String,
    kind: String,
    shader: String,
    radius: Option<i64>,
}

fn preset_defs(slots: &[String], state: &State) -> Vec<PresetOwned> {
    let mut out: Vec<PresetOwned> = Vec::new();
    for slot in slots {
        let Some(sel) = state.selection.get(slot) else {
            continue;
        };
        if !sel.lib || sel.effect.is_empty() || out.iter().any(|p| p.name == sel.effect) {
            continue;
        }
        out.push(PresetOwned {
            name: sel.effect.clone(),
            kind: sel.kind.clone(),
            shader: shader_path(&sel.effect),
            radius: if sel.kind == "cursor" { Some(64) } else { None },
        });
    }
    out
}

fn groups(
    located: &locate::Resolved,
    slots: &[String],
    state: &State,
) -> BTreeMap<PathBuf, Vec<String>> {
    let mut out: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
    for slot in state.selection.keys() {
        if slots.contains(slot) {
            out.entry(located.file_for_slot(slot))
                .or_default()
                .push(slot.clone());
        }
    }
    out
}

fn rollback(changes: &[FileChange]) {
    for change in changes.iter().rev() {
        let path = Path::new(&change.path);
        match &change.before {
            Some(text) => {
                let _ = atomic_write(path, text);
            }
            None => {
                let _ = fs::remove_file(path);
            }
        }
    }
}

/// Rewrites only the requested slots in the file that owns each one, then
/// validates the full configuration. Umbriel watches every file it includes,
/// so saving is enough for the change to take effect. Records exact
/// before/after bytes so undo never has to regenerate a file.
fn write_selection(
    settings: &Settings,
    state: &State,
    slots: &[String],
) -> Result<(Vec<FileChange>, PathBuf), String> {
    let located = locate::detect(&settings.original_config)?;
    let root = located.root.to_string_lossy().to_string();
    // A configuration that was already invalid before this edit should not
    // block the app; only a regression caused by this edit is rolled back.
    let root_valid_before = validate(&root).is_ok();

    let mut changes = Vec::new();
    for (path, group) in groups(&located, slots, state) {
        let before = fs::read_to_string(&path).ok();
        if before.is_none() {
            return Err(format!(
                "No existe el archivo de configuración: {}",
                path.display()
            ));
        }
        let text = before.clone().unwrap_or_default();
        let edits: Vec<SlotEdit> = group
            .iter()
            .filter_map(|slot| {
                state.selection.get(slot).map(|sel| SlotEdit {
                    slot: slot.as_str(),
                    sel,
                })
            })
            .collect();
        let presets = preset_defs(&group, state);
        let preset_refs: Vec<Preset> = presets
            .iter()
            .map(|preset| Preset {
                name: &preset.name,
                kind: &preset.kind,
                shader: &preset.shader,
                radius: preset.radius,
            })
            .collect();
        let after = edit::apply(&text, &edits, &preset_refs)?;
        if before.as_deref() == Some(after.as_str()) {
            continue;
        }
        atomic_write(&path, &after)?;
        changes.push(FileChange {
            path: path.to_string_lossy().to_string(),
            before,
            after: Some(after),
        });
    }

    if root_valid_before {
        if let Err(err) = validate(&root) {
            rollback(&changes);
            return Err(err);
        }
    }
    Ok((changes, located.root))
}

fn apply_entry(
    settings: &Settings,
    entry: &Entry,
    slots: &[String],
    label: &str,
) -> Result<(Vec<FileChange>, PathBuf), String> {
    let preset = prepare_preset(entry)?;
    let mut state = load_state();
    let previous = state.selection.clone();
    let default_duration = 280u32;
    let default_curve = "easeoutcubic".to_string();
    for slot in slots {
        let kind = slot_kind(slot).ok_or_else(|| format!("Destino desconocido: {slot}"))?;
        let duration = state
            .selection
            .get(slot)
            .and_then(|sel| sel.duration_ms)
            .unwrap_or(default_duration);
        let curve = state
            .selection
            .get(slot)
            .and_then(|sel| sel.curve.clone())
            .unwrap_or_else(|| default_curve.clone());
        state.selection.insert(
            slot.clone(),
            SlotSel {
                effect: preset.clone(),
                duration_ms: if kind == "animation" {
                    Some(duration)
                } else {
                    None
                },
                curve: if kind == "animation" {
                    Some(curve)
                } else {
                    None
                },
                lib: entry.source == "library",
                kind: kind.to_string(),
                enabled: true,
            },
        );
    }

    let (changes, root) = write_selection(settings, &state, slots)?;
    if !changes.is_empty() {
        state.history.push(Snapshot {
            at: now_ms(),
            label: label.to_string(),
            selection: previous,
            files: changes.clone(),
        });
        if state.history.len() > 30 {
            state.history.remove(0);
        }
        save_state(&state)?;
    }
    Ok((changes, root))
}

pub fn apply(settings: &Settings, req: &ApplyReq) -> Result<ActionResult, String> {
    let kind = slot_kind(&req.slot).ok_or_else(|| format!("Destino desconocido: {}", req.slot))?;
    let entry = locate_entry(settings, &req.entry_id)?;
    if entry.kind != kind {
        return Err(format!(
            "{} no es compatible con este destino ({} ≠ {}).",
            entry.name, entry.kind, kind
        ));
    }
    let curve = req.curve.clone().unwrap_or_else(|| "easeoutcubic".into());
    if !CURVES.contains(&curve.as_str()) {
        return Err(format!("Curva no válida: {curve}"));
    }
    let duration = req.duration_ms.unwrap_or(280).clamp(1, 10_000);

    // Keep the requested timing: apply_entry inherits existing timing, so write
    // the explicit values into a temporary state first.
    let mut state = load_state();
    let previous = state.selection.clone();
    let preset = prepare_preset(&entry)?;
    state.selection.insert(
        req.slot.clone(),
        SlotSel {
            effect: preset.clone(),
            duration_ms: if kind == "animation" {
                Some(duration)
            } else {
                None
            },
            curve: if kind == "animation" {
                Some(curve)
            } else {
                None
            },
            lib: entry.source == "library",
            kind: kind.to_string(),
            enabled: true,
        },
    );
    let slots = vec![req.slot.clone()];
    let (changes, root) = write_selection(settings, &state, &slots)?;
    if !changes.is_empty() {
        state.history.push(Snapshot {
            at: now_ms(),
            label: format!("{} · {}", req.slot, preset),
            selection: previous,
            files: changes.clone(),
        });
        if state.history.len() > 30 {
            state.history.remove(0);
        }
        save_state(&state)?;
    }

    let target = path_label(&changes, &root);
    Ok(ActionResult {
        ok: true,
        message: format!("{} aplicado a {} en {target}", preset, req.slot),
        generated: target,
    })
}

pub fn apply_family(
    settings: &Settings,
    entry_id: &str,
    kind: &str,
) -> Result<ActionResult, String> {
    let entry = locate_entry(settings, entry_id)?;
    if entry.kind != kind {
        return Err(format!("{} no es un shader de tipo {}.", entry.name, kind));
    }
    let slots: Vec<String> = SLOTS
        .iter()
        .filter(|key| slot_kind(key) == Some(kind))
        .map(|key| key.to_string())
        .collect();
    if slots.is_empty() {
        return Err("No hay destinos para ese tipo de shader.".into());
    }
    let label = format!("{} · {} slots", entry.name, slots.len());
    let (changes, root) = apply_entry(settings, &entry, &slots, &label)?;
    let target = path_label(&changes, &root);
    Ok(ActionResult {
        ok: true,
        message: format!(
            "{} aplicado a {} destinos en {target}",
            entry.name,
            slots.len()
        ),
        generated: target,
    })
}

/// Removes the shader assigned to one slot: animation events are disabled,
/// effect slots are set to Umbriel's empty effect. Undo brings it back.
pub fn clear(settings: &Settings, slot: &str) -> Result<ActionResult, String> {
    let kind = slot_kind(slot).ok_or_else(|| format!("Destino desconocido: {slot}"))?;
    let mut state = load_state();
    let previous = state.selection.clone();
    state.selection.insert(
        slot.to_string(),
        SlotSel {
            effect: String::new(),
            duration_ms: None,
            curve: None,
            lib: false,
            kind: kind.to_string(),
            enabled: false,
        },
    );
    let slots = vec![slot.to_string()];
    let (changes, root) = write_selection(settings, &state, &slots)?;
    if !changes.is_empty() {
        state.history.push(Snapshot {
            at: now_ms(),
            label: format!("{slot} · sin efecto"),
            selection: previous,
            files: changes.clone(),
        });
        if state.history.len() > 30 {
            state.history.remove(0);
        }
        save_state(&state)?;
    }
    let target = path_label(&changes, &root);
    Ok(ActionResult {
        ok: true,
        message: format!("{slot} sin efecto en {target}"),
        generated: target,
    })
}

fn restore_files(changes: &[FileChange]) -> Result<(), String> {
    for change in changes {
        let current = fs::read_to_string(&change.path).ok();
        if current != change.after {
            return Err(format!(
                "{} cambió fuera de esta app; no se deshizo nada.",
                change.path
            ));
        }
    }
    for change in changes.iter().rev() {
        let path = Path::new(&change.path);
        match &change.before {
            Some(text) => atomic_write(path, text)?,
            None => {
                if path.exists() {
                    fs::remove_file(path).map_err(|e| e.to_string())?;
                }
            }
        }
    }
    Ok(())
}

pub fn undo(settings: &Settings) -> Result<ActionResult, String> {
    let mut state = load_state();
    let snapshot = state
        .history
        .pop()
        .ok_or_else(|| "No hay cambios que deshacer.".to_string())?;
    if snapshot.files.is_empty() {
        return Err("Ese cambio no dejó un registro de archivos que restaurar.".into());
    }
    restore_files(&snapshot.files)?;
    state.selection = snapshot.selection.clone();
    save_state(&state)?;
    let located = locate::detect(&settings.original_config)?;
    Ok(ActionResult {
        ok: true,
        message: format!(
            "Deshecho: {} en {}",
            snapshot.label,
            snapshot
                .files
                .iter()
                .map(|file| file.path.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        generated: located.animation_label(),
    })
}

pub fn restore(settings: &Settings) -> Result<ActionResult, String> {
    let mut state = load_state();
    if state.history.is_empty() {
        return Err("No hay cambios de esta app que restaurar.".into());
    }
    let mut before: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut after: BTreeMap<String, Option<String>> = BTreeMap::new();
    for snapshot in &state.history {
        for file in &snapshot.files {
            before
                .entry(file.path.clone())
                .or_insert_with(|| file.before.clone());
            after.insert(file.path.clone(), file.after.clone());
        }
    }
    for (path, expected) in &after {
        let current = fs::read_to_string(path).ok();
        if &current != expected {
            return Err(format!(
                "{path} cambió fuera de esta app; no se restauró nada."
            ));
        }
    }
    for (path, text) in &before {
        match text {
            Some(body) => atomic_write(Path::new(path), body)?,
            None => {
                let _ = fs::remove_file(path);
            }
        }
    }
    state.selection.clear();
    state.history.clear();
    save_state(&state)?;
    let located = locate::detect(&settings.original_config)?;
    Ok(ActionResult {
        ok: true,
        message: format!(
            "Contenido original restaurado en {}",
            located.animation_label()
        ),
        generated: located.animation_label(),
    })
}

pub fn validate_config(settings: &Settings) -> Result<ActionResult, String> {
    let located = locate::detect(&settings.original_config)?;
    let root = located.root.to_string_lossy().to_string();
    validate(&root)?;
    Ok(ActionResult {
        ok: true,
        message: format!("Configuración válida: {root}"),
        generated: located.animation_label(),
    })
}

fn locate_entry(settings: &Settings, id: &str) -> Result<Entry, String> {
    crate::scan::find_entry(settings, id)
        .ok_or_else(|| "El shader ya no está en la biblioteca.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sandbox(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("umbriel-apply-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).unwrap();
        root
    }

    fn settings(root: &Path) -> Settings {
        Settings {
            library_path: root.join("library").to_string_lossy().to_string(),
            original_config: root.join("config.toml").to_string_lossy().to_string(),
            language: "es".into(),
            reduce_motion: false,
        }
    }

    fn animation(text: &str) -> SlotSel {
        SlotSel {
            effect: text.into(),
            duration_ms: Some(420),
            curve: Some("easeoutcubic".into()),
            lib: false,
            kind: "animation".into(),
            enabled: true,
        }
    }

    #[test]
    fn writes_only_the_requested_animation_in_the_included_file() {
        let root = sandbox("included");
        let animations = root.join("src/animations.toml");
        let pristine_other = "[general]\nmod_key = \"Super\"\nautostart = [\"a\", \"b\"]\n";
        fs::write(
            &animations,
            "# keep me\n[animation.windows_in]\nenabled = true\neffect = \"old-open\"\ndim = 0.4\n\n[animation.overview]\nenabled = true\ncurve = \"spring:1,800\"\n",
        )
        .unwrap();
        fs::write(root.join("src/general.toml"), pristine_other).unwrap();
        fs::write(
            root.join("config.toml"),
            "[include]\nfiles = [\"src/animations.toml\", \"src/general.toml\"]\n",
        )
        .unwrap();

        let mut state = State::default();
        state
            .selection
            .insert("animation.windows_in".into(), animation("new-open"));
        let (changes, _root) =
            write_selection(&settings(&root), &state, &["animation.windows_in".into()]).unwrap();

        assert_eq!(changes.len(), 1);
        assert_eq!(Path::new(&changes[0].path), animations);
        let body = fs::read_to_string(&animations).unwrap();
        assert!(body.contains("# keep me"));
        assert!(body.contains("[animation.windows_in]\nenabled = true\neffect = \"new-open\""));
        assert!(body.contains("dim = 0.4"));
        assert!(body.contains("[animation.overview]\nenabled = true\ncurve = \"spring:1,800\""));
        assert_eq!(
            fs::read_to_string(root.join("src/general.toml")).unwrap(),
            pristine_other
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn undo_puts_the_exact_bytes_back() {
        let root = sandbox("undo");
        let animations = root.join("src/animations.toml");
        fs::create_dir_all(root.join("shaders")).unwrap();
        fs::write(
            root.join("shaders/demo.glsl"),
            fs::read_to_string(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("starters/aurora-veil.glsl"),
            )
            .unwrap(),
        )
        .unwrap();
        let original = format!(
            "[effects.preset.demo]\nkind = \"animation\"\nshader = \"shaders/demo.glsl\"\n\n[animation.overview]\nenabled = true\neffect = \"demo\"\ncurve = \"spring:1,800\"\n"
        );
        fs::write(&animations, &original).unwrap();
        fs::write(
            root.join("config.toml"),
            "[include]\nfiles = [\"src/animations.toml\"]\n",
        )
        .unwrap();

        let mut state = State::default();
        let mut slot = animation("demo");
        slot.duration_ms = Some(900);
        state.selection.insert("animation.overview".into(), slot);
        let (changes, _root) =
            write_selection(&settings(&root), &state, &["animation.overview".into()]).unwrap();
        assert_ne!(fs::read_to_string(&animations).unwrap(), original);

        restore_files(&changes).unwrap();
        assert_eq!(fs::read_to_string(&animations).unwrap(), original);
        let _ = fs::remove_dir_all(&root);
    }
}
