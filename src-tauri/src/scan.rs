use crate::model::{Entry, Library, Settings, SlotSel, SlotState, SLOTS};
use crate::store::{catalogue_file, library_dir};
use crate::util::{detect_entry, slug};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const KINDS: &[&str] = &["animation", "border", "window", "screen", "cursor"];

#[derive(Clone, Debug)]
struct Official {
    name: String,
    description: String,
    author: String,
}

fn parse_official(text: &str) -> HashMap<String, Official> {
    let mut out = HashMap::new();
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return out;
    };
    let Some(effects) = value.get("effects").and_then(|value| value.as_array()) else {
        return out;
    };
    for effect in effects {
        let (Some(id), Some(name)) = (
            effect.get("id").and_then(|value| value.as_str()),
            effect.get("name").and_then(|value| value.as_str()),
        ) else {
            continue;
        };
        out.insert(
            id.to_string(),
            Official {
                name: name.to_string(),
                description: effect
                    .get("description")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string(),
                author: effect
                    .get("author")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string(),
            },
        );
    }
    out
}

fn official_meta() -> HashMap<String, Official> {
    fs::read_to_string(catalogue_file())
        .map(|text| parse_official(&text))
        .unwrap_or_default()
}

fn title_from_slug(slug: &str) -> String {
    slug.split('-')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn config_files(settings: &Settings) -> Vec<PathBuf> {
    let Ok(located) = crate::locate::detect(&settings.original_config) else {
        return vec![PathBuf::from(&settings.original_config)];
    };
    let mut files = located.files;
    // Later files override earlier ones, and values in the including file
    // override every include, so the root is parsed last.
    if files.len() > 1 {
        let root = files.remove(0);
        files.push(root);
    }
    files
}

fn unquote(value: &str) -> String {
    value
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .to_string()
}

fn parse_presets(files: &[PathBuf]) -> Vec<Entry> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for file in files {
        let text = fs::read_to_string(file).unwrap_or_default();
        let mut name = String::new();
        let mut kind = String::new();
        let mut shader = String::new();
        let mut flush = |name: &str, kind: &str, shader: &str, out: &mut Vec<Entry>| {
            if name.is_empty() || shader.is_empty() || !seen.insert(name.to_string()) {
                return;
            }
            let path = file.parent().unwrap_or(Path::new(".")).join(shader.trim());
            if !path.is_file() {
                return;
            }
            let source = fs::read_to_string(&path).unwrap_or_default();
            let meta = fs::metadata(&path).ok();
            out.push(Entry {
                id: format!("umbriel:{name}"),
                name: name.to_string(),
                kind: if kind.is_empty() {
                    "animation".into()
                } else {
                    kind.to_string()
                },
                source: "umbriel".into(),
                path: path.to_string_lossy().to_string(),
                size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                modified: meta
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                entry_fn: detect_entry(&source),
                used_by: vec![],
                official: false,
                description: String::new(),
                author: String::new(),
            });
        };
        for line in text.lines() {
            let l = line.trim();
            if l.starts_with('[') {
                flush(&name, &kind, &shader, &mut out);
                let table = l.trim_matches(|c| c == '[' || c == ']');
                name.clear();
                kind.clear();
                shader.clear();
                if let Some(rest) = table.strip_prefix("effects.preset.") {
                    name = unquote(rest);
                }
                continue;
            }
            if name.is_empty() {
                continue;
            }
            if let Some((key, value)) = l.split_once('=') {
                match key.trim() {
                    "kind" => kind = unquote(value),
                    "shader" => shader = unquote(value),
                    _ => {}
                }
            }
        }
        flush(&name, &kind, &shader, &mut out);
    }
    out
}

fn infer_kind(name: &str, source: &str) -> String {
    let lower = name.to_ascii_lowercase();
    if lower.contains("cursor") {
        "cursor".into()
    } else if lower.contains("border") || source.contains("vec4 border(vec2") {
        "border".into()
    } else if lower.contains("screen") || source.contains("vec4 screen(vec2") {
        "screen".into()
    } else if lower.contains("window") || source.contains("vec4 window(vec2") {
        "window".into()
    } else {
        "animation".into()
    }
}

fn scan_library(settings: &Settings) -> Vec<Entry> {
    let root = library_dir(settings);
    let mut files = Vec::new();
    fn walk(dir: &Path, depth: usize, files: &mut Vec<PathBuf>) {
        if depth > 4 || files.len() > 500 {
            return;
        }
        let Ok(read) = fs::read_dir(dir) else { return };
        for item in read.flatten() {
            let path = item.path();
            if path.is_dir() {
                walk(&path, depth + 1, files);
            } else if path.extension().and_then(|e| e.to_str()) == Some("glsl") {
                files.push(path);
            }
        }
    }
    walk(&root, 0, &mut files);
    let official = official_meta();
    let mut out: Vec<Entry> = files
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path).unwrap_or_default();
            let meta = fs::metadata(&path).ok();
            let rel = path.strip_prefix(&root).unwrap_or(&path);
            let parts: Vec<&str> = rel
                .components()
                .filter_map(|part| part.as_os_str().to_str())
                .collect();
            let community =
                parts.len() >= 3 && parts[0] == "community" && KINDS.contains(&parts[1]);
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("shader");
            let parent = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .unwrap_or(stem);
            let (name, kind, info) = if community {
                let id = format!("{}/{}", parts[1], parts[2]);
                let info = official.get(&id);
                (
                    info.map(|item| item.name.clone())
                        .unwrap_or_else(|| title_from_slug(parts[2])),
                    parts[1].to_string(),
                    info,
                )
            } else {
                let name = if stem == "shader" || stem == "open" || stem == "close" {
                    format!("{parent}-{stem}")
                } else {
                    stem.to_string()
                };
                let kind = infer_kind(&name, &source);
                (name, kind, None)
            };
            Entry {
                id: format!("lib:{}", rel.to_string_lossy()),
                name,
                kind,
                source: "library".into(),
                path: path.to_string_lossy().to_string(),
                size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                modified: meta
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                entry_fn: detect_entry(&source),
                used_by: vec![],
                official: community,
                description: info
                    .map(|item| item.description.clone())
                    .unwrap_or_default(),
                author: info.map(|item| item.author.clone()).unwrap_or_default(),
            }
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn parse_slots(files: &[PathBuf]) -> BTreeMap<String, SlotState> {
    let mut slots: BTreeMap<String, SlotState> = SLOTS
        .iter()
        .map(|key| {
            (
                key.to_string(),
                SlotState {
                    key: key.to_string(),
                    kind: crate::model::slot_kind(key).unwrap_or("animation").into(),
                    effect: String::new(),
                    duration_ms: None,
                    curve: None,
                    enabled: true,
                },
            )
        })
        .collect();
    let mut effects_open = false;
    let mut table: Option<String> = None;
    for file in files {
        let text = fs::read_to_string(file).unwrap_or_default();
        for line in text.lines() {
            let l = line.trim();
            if l.starts_with('[') {
                table = Some(l.trim_matches(|c| c == '[' || c == ']').to_string());
                effects_open = table.as_deref() == Some("effects");
                continue;
            }
            let Some((key, value)) = l.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            if effects_open && matches!(key, "border" | "window" | "cursor" | "screen") {
                if let Some(slot) = slots.get_mut(&format!("effects.{key}")) {
                    slot.effect = unquote(value);
                }
            }
            if let Some(event) = table.as_deref().and_then(|t| t.strip_prefix("animation.")) {
                if let Some(slot) = slots.get_mut(&format!("animation.{event}")) {
                    match key {
                        "effect" => slot.effect = unquote(value),
                        "enabled" => slot.enabled = value == "true",
                        "duration_ms" => slot.duration_ms = value.parse().ok(),
                        "curve" => slot.curve = Some(unquote(value)),
                        _ => {}
                    }
                }
            }
        }
    }
    slots
}

pub fn library(
    settings: &Settings,
    selection: &BTreeMap<String, SlotSel>,
    can_undo: bool,
) -> Library {
    let files = config_files(settings);
    let mut entries = scan_library(settings);
    entries.extend(parse_presets(&files));
    let mut slots = parse_slots(&files);
    for (key, sel) in selection {
        if let Some(slot) = slots.get_mut(key) {
            slot.effect = sel.effect.clone();
            if let Some(d) = sel.duration_ms {
                slot.duration_ms = Some(d);
            }
            if let Some(c) = &sel.curve {
                slot.curve = Some(c.clone());
            }
            slot.enabled = sel.enabled;
        }
    }
    for entry in &mut entries {
        let preset = format!("us-{}", slug(&entry.name));
        entry.used_by = slots
            .values()
            .filter(|slot| slot.effect == entry.name || slot.effect == preset)
            .map(|slot| slot.key.clone())
            .collect();
    }

    let library_bytes = entries
        .iter()
        .filter(|e| e.source == "library")
        .map(|e| e.size)
        .sum();

    let generated = crate::locate::detect(&settings.original_config)
        .map(|located| located.animation_label())
        .unwrap_or_else(|_| settings.original_config.clone());

    Library {
        entries,
        slots: slots.into_values().collect(),
        library_path: settings.library_path.clone(),
        original_config: settings.original_config.clone(),
        library_bytes,
        generated,
        can_undo,
    }
}

pub fn find_entry(settings: &Settings, id: &str) -> Option<Entry> {
    library(settings, &BTreeMap::new(), false)
        .entries
        .into_iter()
        .find(|e| e.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_included_presets_and_slot_state() {
        let root = std::env::temp_dir().join(format!("umbriel-scan-{}", std::process::id()));
        let library_dir = root.join("library");
        let shaders = root.join("shaders");
        fs::create_dir_all(&library_dir).unwrap();
        fs::create_dir_all(&shaders).unwrap();
        fs::write(
            shaders.join("a.glsl"),
            "vec4 animation(vec2 uv) { return vec4(1.0); }",
        )
        .unwrap();
        fs::write(
            root.join("anim.toml"),
            "[effects.preset.test-a]\nkind = \"animation\"\nshader = \"shaders/a.glsl\"\n\n[animation.overview]\neffect = \"us-test-a\"\nduration_ms = 320\ncurve = \"easeoutcubic\"\n",
        )
        .unwrap();
        fs::write(
            root.join("config.toml"),
            "[include]\nfiles = [\"anim.toml\"]\n",
        )
        .unwrap();

        let settings = Settings {
            library_path: library_dir.to_string_lossy().to_string(),
            original_config: root.join("config.toml").to_string_lossy().to_string(),
            language: "es".into(),
            reduce_motion: false,
        };
        let result = library(&settings, &BTreeMap::new(), false);
        assert!(result
            .entries
            .iter()
            .any(|entry| entry.name == "test-a" && entry.kind == "animation"));
        let overview = result
            .slots
            .iter()
            .find(|slot| slot.key == "animation.overview")
            .unwrap();
        assert_eq!(overview.effect, "us-test-a");
        assert_eq!(overview.duration_ms, Some(320));
        assert_eq!(overview.curve.as_deref(), Some("easeoutcubic"));
        let entry = result
            .entries
            .iter()
            .find(|entry| entry.name == "test-a")
            .unwrap();
        assert_eq!(entry.used_by, vec!["animation.overview".to_string()]);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn official_meta_indexes_catalogue_records() {
        let map = parse_official(
            r#"{"effects":[{"id":"border/glow","name":"Glow","description":"Halo","author":"Barrulus"}]}"#,
        );
        let item = map.get("border/glow").unwrap();
        assert_eq!(item.name, "Glow");
        assert_eq!(item.description, "Halo");
        assert_eq!(item.author, "Barrulus");
        assert!(parse_official("not json").is_empty());
        assert_eq!(title_from_slug("midnight-fog"), "Midnight Fog");
    }
}
