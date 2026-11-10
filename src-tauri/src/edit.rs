use crate::model::SlotSel;
use toml_edit::{value, DocumentMut, InlineTable, Item, Table, Value};

pub struct SlotEdit<'a> {
    pub slot: &'a str,
    pub sel: &'a SlotSel,
}

pub struct Preset<'a> {
    pub name: &'a str,
    pub kind: &'a str,
    pub shader: &'a str,
    pub radius: Option<i64>,
}

/// Rewrites only the keys of the given slots. Everything else in `text` —
/// comments, ordering, other tables — is preserved as-is.
pub fn apply(
    text: &str,
    edits: &[SlotEdit<'_>],
    presets: &[Preset<'_>],
) -> Result<String, String> {
    let mut doc: DocumentMut = text
        .parse()
        .map_err(|err| format!("El archivo no es TOML válido: {err}"))?;
    for edit in edits {
        apply_slot(&mut doc, edit)?;
    }
    for preset in presets {
        apply_preset(&mut doc, preset)?;
    }
    Ok(doc.to_string())
}

fn split_slot(slot: &str) -> (&str, &str) {
    match slot.split_once('.') {
        Some((section, key)) => (section, key),
        None => (slot, ""),
    }
}

fn section_mut<'a>(doc: &'a mut DocumentMut, section: &str) -> Result<&'a mut Item, String> {
    if !doc.contains_key(section) {
        doc.insert(section, Item::Table(Table::new()));
    }
    doc.get_mut(section)
        .ok_or_else(|| format!("No se pudo abrir la sección [{section}]."))
}

/// Returns the child table `key` of `parent`, creating it when missing. An
/// inline parent gets an inline child so the file stays valid TOML.
fn child_mut<'a>(parent: &'a mut Item, key: &str, label: &str) -> Result<&'a mut Item, String> {
    let missing = !parent
        .as_table_like()
        .map(|table| table.contains_key(key))
        .unwrap_or(false);
    if missing {
        let new_item = match parent {
            Item::Table(_) => Item::Table(Table::new()),
            Item::Value(Value::InlineTable(_)) => {
                Item::Value(Value::InlineTable(InlineTable::new()))
            }
            _ => return Err(format!("La sección [{label}] no es una tabla editable.")),
        };
        parent
            .as_table_like_mut()
            .ok_or_else(|| format!("La sección [{label}] no es una tabla editable."))?
            .insert(key, new_item);
    }
    parent
        .as_table_like_mut()
        .and_then(|table| table.get_mut(key))
        .ok_or_else(|| format!("{label}.{key} no es una tabla editable."))
}

fn apply_slot(doc: &mut DocumentMut, edit: &SlotEdit<'_>) -> Result<(), String> {
    let (section, key) = split_slot(edit.slot);
    if key.is_empty() {
        return Err(format!("Destino no válido: {}", edit.slot));
    }
    let section_item = section_mut(doc, section)?;

    if section == "effects" {
        let table = section_item
            .as_table_like_mut()
            .ok_or_else(|| "La sección [effects] no es una tabla editable.".to_string())?;
        table.insert(key, value(edit.sel.effect.clone()));
        return Ok(());
    }

    let slot_item = child_mut(section_item, key, section)?;
    let table = slot_item
        .as_table_like_mut()
        .ok_or_else(|| format!("{}.{key} no es una tabla editable.", section))?;
    table.insert("enabled", value(edit.sel.enabled));
    if edit.sel.effect.is_empty() {
        // Clearing a slot disables the event and drops only its effect key.
        table.remove("effect");
    } else {
        table.insert("effect", value(edit.sel.effect.clone()));
        if let Some(duration) = edit.sel.duration_ms {
            table.insert("duration_ms", value(i64::from(duration)));
        }
        if let Some(curve) = &edit.sel.curve {
            table.insert("curve", value(curve.clone()));
        }
    }
    Ok(())
}

fn apply_preset(doc: &mut DocumentMut, preset: &Preset<'_>) -> Result<(), String> {
    let effects = section_mut(doc, "effects")?;
    let group = child_mut(effects, "preset", "effects")?;
    let entry = child_mut(group, preset.name, "effects.preset")?;
    let table = entry
        .as_table_like_mut()
        .ok_or_else(|| format!("effects.preset.{} no es una tabla editable.", preset.name))?;
    if !table.contains_key("kind") {
        table.insert("kind", value(preset.kind.to_string()));
    }
    if !table.contains_key("shader") {
        table.insert("shader", value(preset.shader.to_string()));
    }
    if let Some(radius) = preset.radius {
        if !table.contains_key("radius") {
            table.insert("radius", value(radius));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"# my animations , keep this comment
[effects.preset.nier-open]
kind = "animation"
shader = "shaders/tri-shatter/open.glsl"

[animation]
enabled = true
duration_ms = 300
curve = "easeout"

[animation.windows_in]
enabled = true
duration_ms = 500
effect = "nier-open"
dim = 0.25

[general]
mod_key = "Super"
"#;

    fn sel(effect: &str, curve: Option<&str>, duration: Option<u32>) -> SlotSel {
        SlotSel {
            effect: effect.into(),
            duration_ms: duration,
            curve: curve.map(str::to_string),
            lib: true,
            kind: "animation".into(),
            enabled: true,
        }
    }

    #[test]
    fn edits_only_the_target_slot_and_keeps_everything_else() {
        let slot = sel("us-aurora-veil", Some("easeoutcubic"), Some(320));
        let preset = Preset {
            name: "us-aurora-veil",
            kind: "animation",
            shader: "/data/shaders/us-aurora-veil.glsl",
            radius: None,
        };
        let out = apply(
            SAMPLE,
            &[SlotEdit {
                slot: "animation.windows_in",
                sel: &slot,
            }],
            &[preset],
        )
        .unwrap();

        assert!(out.contains("# my animations , keep this comment"));
        assert!(out.contains("[effects.preset.nier-open]"));
        assert!(out.contains("[general]\nmod_key = \"Super\""));
        assert!(out.contains("[animation.windows_in]"));
        assert!(out.contains("effect = \"us-aurora-veil\""));
        assert!(out.contains("duration_ms = 320"));
        assert!(out.contains("curve = \"easeoutcubic\""));
        assert!(out.contains("dim = 0.25"));
        assert!(out.contains("[effects.preset.us-aurora-veil]\nkind = \"animation\"\nshader = \"/data/shaders/us-aurora-veil.glsl\""));
        // Untouched animation keys survive.
        assert!(out.contains("[animation]\nenabled = true\nduration_ms = 300\ncurve = \"easeout\""));
        // The old effect of the edited slot is the only value replaced.
        assert!(!out.contains("effect = \"nier-open\""));
    }

    #[test]
    fn clearing_a_slot_only_disables_it() {
        let slot = SlotSel {
            effect: String::new(),
            duration_ms: None,
            curve: None,
            lib: false,
            kind: "animation".into(),
            enabled: false,
        };
        let out = apply(
            SAMPLE,
            &[SlotEdit {
                slot: "animation.windows_in",
                sel: &slot,
            }],
            &[],
        )
        .unwrap();
        assert!(out.contains("[animation.windows_in]"));
        assert!(out.contains("enabled = false"));
        assert!(out.contains("dim = 0.25"));
        assert!(!out.contains("effect = \"nier-open\""));
        assert!(out.contains("[animation]\nenabled = true\nduration_ms = 300"));
    }

    #[test]
    fn creates_missing_slot_without_touching_others() {
        let slot = sel("us-x", Some("easeout"), Some(200));
        let out = apply(
            SAMPLE,
            &[SlotEdit {
                slot: "animation.dialogs",
                sel: &slot,
            }],
            &[],
        )
        .unwrap();
        assert!(out.contains("[animation.dialogs]\nenabled = true\neffect = \"us-x\""));
        assert!(out.contains("[animation.windows_in]\nenabled = true\nduration_ms = 500"));
    }

    #[test]
    fn effect_slots_are_written_as_plain_strings() {
        let slot = sel("glow-naranja", None, None);
        let mut slot = slot;
        slot.kind = "cursor".into();
        let out = apply(
            " [effects]\ncursor = \"\"\nborder = \"a\"\n",
            &[SlotEdit {
                slot: "effects.cursor",
                sel: &slot,
            }],
            &[],
        )
        .unwrap();
        assert!(out.contains("cursor = \"glow-naranja\""));
        assert!(out.contains("border = \"a\""));
    }
}
