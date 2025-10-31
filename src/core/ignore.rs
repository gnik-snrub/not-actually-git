use crate::core::repo::find_repo_root;

use glob::Pattern;
use std::path::Path;

fn load_ignore_patterns() -> std::io::Result<Vec<String>> {
    let mut patterns = Vec::new();
    let ignore_file_path = find_repo_root()?.join(".nagignore");
    if ignore_file_path.exists() {
        let contents = std::fs::read_to_string(ignore_file_path)?;
        for line in contents.lines() {
            if line.starts_with("#") || line.is_empty() {
                continue;
            }
            patterns.push(line.trim().to_string());
        }
    }
    Ok(patterns)
}

fn is_ignored(path: &Path, patterns: &Vec<String>) -> std::io::Result<bool> {
    let root = find_repo_root()?;
    let rel_path = path.strip_prefix(&root).unwrap_or(path);
    let normalized_path = rel_path.to_string_lossy().replace("\\", "/");

    let mut tracker: bool = false;
    for pattern in patterns {
        let is_negated = pattern.starts_with("!");
        let mut pattern_str = if is_negated { &pattern[1..] } else { &pattern };
        let owned: String;
        if pattern_str.ends_with('/') {
            owned = format!("{}**", pattern_str);
            pattern_str = &owned;
        }

        let pat = match Pattern::new(pattern_str) {
            Ok(p) => p,
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid ignore pattern '{}': {}", pattern_str, e.msg),
                ));
            }
        };
        if pat.matches(&normalized_path) {
            tracker = !is_negated;
        }
    }
    Ok(tracker)
}

pub fn should_ignore(path: &Path) -> std::io::Result<bool> {
    let patterns = load_ignore_patterns()?;
    Ok(is_ignored(path, &patterns)?)
}
