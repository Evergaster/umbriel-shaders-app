mod apply;
mod edit;
mod locate;
mod model;
mod scan;
mod store;
mod util;

use model::{ActionResult, ApplyReq, Library, Settings};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const CATALOGUE_REPO: &str = "https://github.com/noctalia-dev/community-umbriel-shaders";
const CATALOGUE_KINDS: &[&str] = &["animation", "border", "window", "screen", "cursor"];

const STARTERS: &[(&str, &str)] = &[
    ("aurora-veil", include_str!("../starters/aurora-veil.glsl")),
    ("paper-fade", include_str!("../starters/paper-fade.glsl")),
    (
        "focus-ring-pulse",
        include_str!("../starters/focus-ring-pulse.glsl"),
    ),
    (
        "desaturate-soft",
        include_str!("../starters/desaturate-soft.glsl"),
    ),
    ("panel-glide", include_str!("../starters/panel-glide.glsl")),
    (
        "caustic-drift",
        include_str!("../starters/caustic-drift.glsl"),
    ),
    ("sodium-haze", include_str!("../starters/sodium-haze.glsl")),
    (
        "scanline-memory",
        include_str!("../starters/scanline-memory.glsl"),
    ),
];

#[tauri::command]
fn settings_get() -> Settings {
    store::load_settings()
}

#[tauri::command]
fn settings_save(mut settings: Settings) -> Result<Settings, String> {
    settings.library_path = util::expand(&settings.library_path);
    settings.original_config = util::expand(&settings.original_config);
    let library = Path::new(&settings.library_path);
    if !library.is_absolute() {
        return Err("La ruta de la biblioteca debe ser absoluta.".into());
    }
    let original = Path::new(&settings.original_config);
    if !original.is_absolute() {
        return Err("La ruta de la configuración original debe ser absoluta.".into());
    }
    // The app edits the configuration file in place, animation by animation, so
    // changing the path never rewrites or backs up the previous file.
    store::save_settings(&settings)?;
    Ok(store::load_settings())
}

#[tauri::command]
fn library_scan() -> Library {
    let settings = store::load_settings();
    let state = store::load_state();
    scan::library(&settings, &state.selection, !state.history.is_empty())
}

#[tauri::command]
fn library_install_starters() -> Result<usize, String> {
    let settings = store::load_settings();
    let root = PathBuf::from(&settings.library_path);
    let mut installed = 0;
    for (name, source) in STARTERS {
        let dir = root.join(name);
        let path = dir.join(format!("{name}.glsl"));
        if path.exists() {
            continue;
        }
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        fs::write(&path, source).map_err(|e| e.to_string())?;
        installed += 1;
    }
    Ok(installed)
}

#[tauri::command]
fn library_import(path: String) -> Result<usize, String> {
    let settings = store::load_settings();
    let source =
        fs::canonicalize(util::expand(&path)).map_err(|_| "Esa ruta no existe.".to_string())?;
    let root = PathBuf::from(&settings.library_path);
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let root = fs::canonicalize(&root).map_err(|e| e.to_string())?;
    if source.starts_with(&root) {
        return Err("Ese shader ya está dentro de la biblioteca.".into());
    }

    let mut copied = 0;
    if source.is_dir() {
        let slug = util::slug(
            source
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("shader"),
        );
        let target = root.join(&slug);
        fs::create_dir_all(&target).map_err(|e| e.to_string())?;
        for item in fs::read_dir(&source).map_err(|e| e.to_string())?.flatten() {
            let file = item.path();
            if file.extension().and_then(|e| e.to_str()) == Some("glsl") {
                let name = file.file_name().unwrap_or_default();
                fs::copy(&file, target.join(name)).map_err(|e| e.to_string())?;
                copied += 1;
            }
        }
        if copied == 0 {
            return Err("La carpeta no contiene archivos .glsl.".into());
        }
    } else {
        if source.extension().and_then(|e| e.to_str()) != Some("glsl") {
            return Err("Solo se pueden importar archivos .glsl.".into());
        }
        let stem = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("shader");
        let slug = util::slug(stem);
        let target = root.join(&slug);
        fs::create_dir_all(&target).map_err(|e| e.to_string())?;
        fs::copy(&source, target.join(format!("{slug}.glsl"))).map_err(|e| e.to_string())?;
        copied = 1;
    }
    Ok(copied)
}

#[tauri::command]
fn library_remove(id: String) -> Result<(), String> {
    let settings = store::load_settings();
    let root = PathBuf::from(&settings.library_path);
    let rel = id
        .strip_prefix("lib:")
        .ok_or_else(|| "Solo se pueden borrar shaders de la biblioteca.".to_string())?;
    if rel.contains("..") || Path::new(rel).is_absolute() {
        return Err("Ruta no válida.".into());
    }
    let target = root.join(rel);
    let canonical = fs::canonicalize(&target).map_err(|_| "El shader ya no existe.".to_string())?;
    let root = fs::canonicalize(&root).map_err(|e| e.to_string())?;
    if !canonical.starts_with(&root) {
        return Err("Ruta fuera de la biblioteca.".into());
    }
    fs::remove_file(&canonical).map_err(|e| e.to_string())?;
    if let Some(parent) = canonical.parent() {
        let _ = fs::remove_dir(parent);
    }
    Ok(())
}

#[tauri::command]
fn library_reveal() -> Result<(), String> {
    let settings = store::load_settings();
    util::run("xdg-open", &[&settings.library_path])?;
    Ok(())
}

#[tauri::command]
fn shader_source(id: String) -> Result<serde_json::Value, String> {
    let settings = store::load_settings();
    let entry =
        scan::find_entry(&settings, &id).ok_or_else(|| "Shader no encontrado.".to_string())?;
    let source = fs::read_to_string(&entry.path).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "path": entry.path,
        "source": source,
        "entryFn": entry.entry_fn,
        "kind": entry.kind,
        "name": entry.name,
    }))
}

#[tauri::command]
fn catalogue_info() -> serde_json::Value {
    fs::read_to_string(store::catalogue_file())
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .map(|value| catalogue_summary(&value))
        .unwrap_or(serde_json::Value::Null)
}

fn catalogue_summary(value: &serde_json::Value) -> serde_json::Value {
    let effects = value
        .get("effects")
        .and_then(|items| items.as_array())
        .cloned()
        .unwrap_or_default();
    let mut kinds = serde_json::Map::new();
    for effect in &effects {
        if let Some(kind) = effect.get("kind").and_then(|item| item.as_str()) {
            let count = kinds.get(kind).and_then(|item| item.as_u64()).unwrap_or(0) + 1;
            kinds.insert(kind.to_string(), count.into());
        }
    }
    serde_json::json!({
        "revision": value.pointer("/source/revision").and_then(|item| item.as_str()),
        "total": effects.len(),
        "kinds": kinds,
    })
}

#[tauri::command]
fn catalogue_save(json: String) -> Result<serde_json::Value, String> {
    if json.len() > 4 * 1024 * 1024 {
        return Err("El catálogo es demasiado grande.".into());
    }
    let value: serde_json::Value =
        serde_json::from_str(&json).map_err(|err| format!("Catálogo no válido: {err}"))?;
    if value.get("draft").and_then(|item| item.as_bool()) != Some(false) {
        return Err("El catálogo descargado no está publicado.".into());
    }
    if value
        .pointer("/source/repository")
        .and_then(|item| item.as_str())
        != Some(CATALOGUE_REPO)
    {
        return Err("El catálogo no viene del repositorio oficial.".into());
    }
    let revision = value
        .pointer("/source/revision")
        .and_then(|item| item.as_str())
        .unwrap_or_default();
    if revision.len() != 40 || !revision.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("La revisión del catálogo no es válida.".into());
    }
    let effects = value
        .get("effects")
        .and_then(|items| items.as_array())
        .map(|items| items.len())
        .unwrap_or(0);
    if effects == 0 || effects > 1000 {
        return Err("El catálogo no contiene una lista de shaders válida.".into());
    }
    util::atomic_write(&store::catalogue_file(), &json)?;
    Ok(catalogue_summary(&value))
}

#[tauri::command]
fn library_install_shader(
    kind: String,
    slug: String,
    source: String,
    sha256: String,
) -> Result<(), String> {
    if !CATALOGUE_KINDS.contains(&kind.as_str()) {
        return Err("Tipo de shader desconocido.".into());
    }
    let valid_slug = !slug.is_empty()
        && slug.len() <= 128
        && !slug.contains("--")
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !valid_slug {
        return Err("Nombre de shader no válido.".into());
    }
    if source.len() > 1024 * 1024 {
        return Err("El shader es demasiado grande.".into());
    }
    let digest = format!("{:x}", Sha256::digest(source.as_bytes()));
    if digest != sha256.to_ascii_lowercase() {
        return Err(format!(
            "El shader {kind}/{slug} no coincide con el catálogo."
        ));
    }
    let root = PathBuf::from(&store::load_settings().library_path);
    let target = root
        .join("community")
        .join(&kind)
        .join(&slug)
        .join("shader.glsl");
    util::atomic_write(&target, &source)
}

#[tauri::command]
fn umbriel_status() -> serde_json::Value {
    match util::run("umbriel", &["effects", "--json"]) {
        Ok((0, out, _)) => match serde_json::from_str::<serde_json::Value>(&out) {
            Ok(value) => {
                let presets = value
                    .get("presets")
                    .and_then(|p| p.as_array())
                    .cloned()
                    .unwrap_or_default();
                let compiled = presets
                    .iter()
                    .filter(|p| p.get("state").and_then(|s| s.as_str()) == Some("compiled"))
                    .count();
                serde_json::json!({
                    "connected": true,
                    "presets": presets,
                    "compiled": compiled,
                    "error": null
                })
            }
            Err(err) => serde_json::json!({
                "connected": false,
                "presets": [],
                "compiled": 0,
                "error": format!("Respuesta inesperada de Umbriel: {err}")
            }),
        },
        Ok((_, _, err)) => serde_json::json!({
            "connected": false,
            "presets": [],
            "compiled": 0,
            "error": if err.trim().is_empty() { "Umbriel no está disponible.".to_string() } else { err.trim().to_string() }
        }),
        Err(err) => serde_json::json!({
            "connected": false,
            "presets": [],
            "compiled": 0,
            "error": err
        }),
    }
}

#[tauri::command]
fn apply_shader(req: ApplyReq) -> Result<ActionResult, String> {
    let settings = store::load_settings();
    apply::apply(&settings, &req)
}

#[tauri::command]
fn apply_family(entry_id: String, kind: String) -> Result<ActionResult, String> {
    let settings = store::load_settings();
    apply::apply_family(&settings, &entry_id, &kind)
}

#[tauri::command]
fn clear_slot(slot: String) -> Result<ActionResult, String> {
    let settings = store::load_settings();
    apply::clear(&settings, &slot)
}

#[tauri::command]
fn undo_change() -> Result<ActionResult, String> {
    let settings = store::load_settings();
    apply::undo(&settings)
}

#[tauri::command]
fn restore_original() -> Result<ActionResult, String> {
    let settings = store::load_settings();
    apply::restore(&settings)
}

#[tauri::command]
fn validate_config() -> Result<ActionResult, String> {
    let settings = store::load_settings();
    apply::validate_config(&settings)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            settings_get,
            settings_save,
            library_scan,
            library_install_starters,
            library_import,
            library_remove,
            library_reveal,
            library_install_shader,
            catalogue_info,
            catalogue_save,
            shader_source,
            umbriel_status,
            apply_shader,
            apply_family,
            clear_slot,
            undo_change,
            restore_original,
            validate_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running Umbriel Shaders");
}
