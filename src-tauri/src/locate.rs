use crate::util::expand;
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item};

/// Files that make up one Umbriel configuration, plus the one that actually
/// defines the animation slots.
#[derive(Clone, Debug)]
pub struct Resolved {
    pub root: PathBuf,
    pub files: Vec<PathBuf>,
    pub animation_files: Vec<PathBuf>,
    /// File that holds most `[animation.*]` tables; new slots are created there
    /// so the user's layout of files stays the same.
    pub preferred_animation: PathBuf,
}

pub fn default_root() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("umbriel")
        .join("config.toml")
}

fn parse(path: &Path) -> Option<DocumentMut> {
    let text = fs::read_to_string(path).ok()?;
    text.parse::<DocumentMut>().ok()
}

fn include_names(item: &Item) -> Vec<String> {
    item.get("files")
        .and_then(|files| files.as_array())
        .map(|files| {
            files
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn include_paths(path: &Path) -> Vec<PathBuf> {
    let Some(doc) = parse(path) else {
        return Vec::new();
    };
    let Some(include) = doc.get("include") else {
        return Vec::new();
    };
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let mut names = include_names(include);
    if let Some(optional) = include.get("optional") {
        names.extend(include_names(optional));
    }
    names
        .into_iter()
        .map(|name| {
            let candidate = PathBuf::from(name);
            if candidate.is_absolute() {
                candidate
            } else {
                base.join(candidate)
            }
        })
        .collect()
}

fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn has_include(path: &Path) -> bool {
    parse(path).map(|doc| doc.get("include").is_some()).unwrap_or(false)
}

/// Root plus every file reachable through `[include]`, in declaration order.
fn closure(root: &Path) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    let mut queue = VecDeque::from([(root.to_path_buf(), 0usize)]);
    while let Some((path, depth)) = queue.pop_front() {
        if depth > 6 || out.len() > 64 {
            break;
        }
        let key = canonical(&path);
        if !seen.insert(key) {
            continue;
        }
        if !path.is_file() {
            continue;
        }
        out.push(path.clone());
        for child in include_paths(&path) {
            queue.push_back((child, depth + 1));
        }
    }
    out
}

fn animation_keys(path: &Path) -> Option<Vec<String>> {
    let doc = parse(path)?;
    let animation = doc.get("animation")?;
    let table = animation.as_table_like()?;
    if table.is_empty() {
        return None;
    }
    Some(table.iter().map(|(key, _)| key.to_string()).collect())
}

fn contains_section(path: &Path, section: &str) -> bool {
    parse(path)
        .and_then(|doc| doc.get(section).map(|_| ()))
        .is_some()
}

fn contains_key(path: &Path, section: &str, key: &str) -> bool {
    parse(path)
        .and_then(|doc| {
            doc.get(section)
                .and_then(|item| item.as_table_like().map(|table| table.contains_key(key)))
        })
        .unwrap_or(false)
}

/// Finds the configuration Umbriel loads and which of its files carries the
/// animation slots. Accepts a root config, an included file, or a path that no
/// longer exists inside the standard Umbriel config directory.
pub fn detect(configured: &str) -> Result<Resolved, String> {
    let configured = PathBuf::from(expand(configured));
    let default = default_root();
    let configured = if configured.is_file() {
        configured
    } else if default.is_file() {
        default.clone()
    } else {
        return Err(format!(
            "No encuentro la configuración de Umbriel en {}. Indica la ruta correcta en Ajustes.",
            configured.display()
        ));
    };
    let configured = canonical(&configured);

    let base = default.parent().map(|dir| dir.to_path_buf());
    let inside_base = base
        .as_deref()
        .map(|dir| configured.starts_with(dir))
        .unwrap_or(false);

    let root = if has_include(&configured) {
        configured.clone()
    } else {
        // The configured file may be an include of a nearby root (the standard
        // `config.toml` or one in a parent directory), which is what Umbriel
        // actually loads.
        let mut candidates: Vec<PathBuf> = vec![default.clone()];
        let mut dir = configured.parent();
        for _ in 0..3 {
            let Some(parent) = dir else { break };
            candidates.push(parent.join("config.toml"));
            dir = parent.parent();
        }
        candidates
            .into_iter()
            .filter(|candidate| candidate.is_file())
            .map(|candidate| canonical(&candidate))
            .find(|candidate| {
                closure(candidate)
                    .iter()
                    .any(|file| canonical(file) == configured)
            })
            .unwrap_or_else(|| {
                if inside_base && default.is_file() {
                    canonical(&default)
                } else {
                    configured.clone()
                }
            })
    };

    let files = closure(&root);
    let mut animation_files: Vec<PathBuf> = files
        .iter()
        .filter(|file| animation_keys(file).is_some())
        .cloned()
        .collect();
    if animation_files.is_empty() {
        animation_files.push(root.clone());
    }
    animation_files.dedup();
    let preferred_animation = animation_files
        .iter()
        .max_by_key(|file| animation_keys(file).map(|keys| keys.len()).unwrap_or(0))
        .cloned()
        .unwrap_or_else(|| root.clone());

    Ok(Resolved {
        root,
        files,
        animation_files,
        preferred_animation,
    })
}

fn split_slot(slot: &str) -> (&str, &str) {
    match slot.split_once('.') {
        Some((section, key)) => (section, key),
        None => (slot, ""),
    }
}

impl Resolved {
    /// The one file that owns `slot`. Values in an including file win over its
    /// includes, so the root is checked first.
    pub fn file_for_slot(&self, slot: &str) -> PathBuf {
        let (section, key) = split_slot(slot);
        if key.is_empty() {
            return self.root.clone();
        }
        for file in self.animation_files.iter().chain(self.files.iter()) {
            if contains_key(file, section, key) {
                return file.clone();
            }
        }
        if section == "animation" {
            return self.preferred_animation.clone();
        }
        self.files
            .iter()
            .find(|file| contains_section(file, section))
            .cloned()
            .unwrap_or_else(|| self.root.clone())
    }

    pub fn animation_label(&self) -> String {
        self.animation_files
            .iter()
            .map(|file| file.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn sandbox(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("umbriel-locate-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/animations.toml"),
            "[animation.windows_in]\nenabled = true\neffect = \"nier-open\"\n",
        )
        .unwrap();
        fs::write(
            root.join("src/general.toml"),
            "[general]\nmod_key = \"Super\"\n",
        )
        .unwrap();
        fs::write(
            root.join("config.toml"),
            "[include]\nfiles = [\"src/general.toml\", \"src/animations.toml\"]\n",
        )
        .unwrap();
        root
    }

    #[test]
    fn finds_animation_file_between_root_and_includes() {
        let root = sandbox("detect");
        let resolved = detect(&root.join("config.toml").to_string_lossy()).unwrap();
        assert_eq!(resolved.root, canonical(&root.join("config.toml")));
        assert_eq!(
            resolved.file_for_slot("animation.windows_in"),
            root.join("src/animations.toml")
        );
        assert!(resolved.files.contains(&root.join("src/general.toml")));
        assert_eq!(
            resolved.file_for_slot("animation.dialogs"),
            root.join("src/animations.toml")
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn resolves_an_included_file_to_its_root() {
        let root = sandbox("included");
        let resolved = detect(&root.join("src/animations.toml").to_string_lossy()).unwrap();
        assert_eq!(resolved.root, canonical(&root.join("config.toml")));
        assert_eq!(
            resolved.file_for_slot("animation.windows_in"),
            root.join("src/animations.toml")
        );
        let _ = fs::remove_dir_all(&root);
    }
}
