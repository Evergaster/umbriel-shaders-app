use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SLOTS: &[&str] = &[
    "animation.overview",
    "animation.scratchpad",
    "animation.border",
    "animation.dim_unfocused",
    "animation.layers",
    "animation.windows_in",
    "animation.windows_out",
    "animation.windows_move",
    "animation.workspaces",
    "animation.dialogs",
    "effects.border",
    "effects.window",
    "effects.cursor",
    "effects.screen",
];

pub fn slot_kind(slot: &str) -> Option<&'static str> {
    match slot {
        "effects.border" => Some("border"),
        "effects.window" => Some("window"),
        "effects.cursor" => Some("cursor"),
        "effects.screen" => Some("screen"),
        s if s.starts_with("animation.") => Some("animation"),
        _ => None,
    }
}

pub const CURVES: &[&str] = &[
    "linear",
    "ease",
    "easein",
    "easeout",
    "easeinout",
    "easeincubic",
    "easeoutcubic",
    "easeinoutcubic",
    "snappy",
    "overshoot",
    "smooth",
];

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub library_path: String,
    pub original_config: String,
    pub language: String,
    pub reduce_motion: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SlotSel {
    pub effect: String,
    #[serde(default)]
    pub duration_ms: Option<u32>,
    #[serde(default)]
    pub curve: Option<String>,
    #[serde(default)]
    pub lib: bool,
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_kind() -> String {
    "animation".into()
}

fn default_enabled() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct State {
    pub selection: BTreeMap<String, SlotSel>,
    pub history: Vec<Snapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub at: u64,
    pub label: String,
    pub selection: BTreeMap<String, SlotSel>,
    #[serde(default)]
    pub files: Vec<FileChange>,
}

/// Exact bytes of one configuration file before and after an edit, so undo and
/// restore can put the file back without regenerating it.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileChange {
    pub path: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub source: String,
    pub path: String,
    pub size: u64,
    pub modified: u64,
    pub entry_fn: String,
    pub used_by: Vec<String>,
    pub official: bool,
    pub description: String,
    pub author: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SlotState {
    pub key: String,
    pub kind: String,
    pub effect: String,
    pub duration_ms: Option<u32>,
    pub curve: Option<String>,
    pub enabled: bool,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub entries: Vec<Entry>,
    pub slots: Vec<SlotState>,
    pub library_path: String,
    pub original_config: String,
    pub library_bytes: u64,
    pub generated: String,
    pub can_undo: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyReq {
    pub slot: String,
    pub entry_id: String,
    pub duration_ms: Option<u32>,
    pub curve: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
    pub ok: bool,
    pub message: String,
    pub generated: String,
}
