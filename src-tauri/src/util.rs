use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn expand(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().to_string();
        }
    }
    path.to_string()
}

pub fn slug(input: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for ch in input.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
            dash = false;
        } else if !out.is_empty() && !dash {
            out.push('-');
            dash = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "shader".into()
    } else {
        trimmed
    }
}

pub fn atomic_write(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, contents).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn run(program: &str, args: &[&str]) -> Result<(i32, String, String), String> {
    let out = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("{program}: {e}"))?;
    Ok((
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    ))
}

pub fn detect_entry(source: &str) -> String {
    for name in ["animation", "cursor", "border", "window", "screen"] {
        if source.contains(&format!("vec4 {name}(vec2")) {
            return name.into();
        }
    }
    for line in source.lines() {
        if let Some(pos) = line.find("vec4 ") {
            let rest = &line[pos + 5..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return name;
            }
        }
    }
    "animation".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_keeps_names_path_safe() {
        assert_eq!(slug("Aurora Veil!"), "aurora-veil");
        assert_eq!(slug("../etc/passwd"), "etc-passwd");
        assert_eq!(slug("///"), "shader");
    }

    #[test]
    fn entry_function_prefers_known_names() {
        assert_eq!(
            detect_entry("vec4 border(vec2 uv) { return vec4(1.0); }"),
            "border"
        );
        assert_eq!(
            detect_entry("vec4 custom(vec2 uv) { return vec4(1.0); }"),
            "custom"
        );
        assert_eq!(detect_entry("// nothing"), "animation");
    }
}
