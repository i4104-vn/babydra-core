//! Filter and pattern matching utilities for File Chooser.

use std::path::Path;
use super::types::FileFilter;

/// Checks if a filename matches a simple glob pattern like `*.png` or `foo.*` or `*`.
pub fn matches_pattern(filename: &str, pattern: &str) -> bool {
    let pat = pattern.trim().to_lowercase();
    let name = filename.to_lowercase();

    if pat == "*" || pat == "*.*" {
        return true;
    }

    if let Some(ext) = pat.strip_prefix("*.") {
        return name.ends_with(&format!(".{ext}"));
    }

    if let Some(prefix) = pat.strip_suffix(".*") {
        return name.starts_with(prefix);
    }

    // Exact match fallback
    name == pat
}

/// Checks whether a file path satisfies the conditions of a `FileFilter`.
pub fn matches_filter(path: &Path, filter: &FileFilter) -> bool {
    // If filter has no patterns and no mime types, matches all
    if filter.patterns.is_empty() && filter.mime_types.is_empty() {
        return true;
    }

    let filename = match path.file_name().and_then(|f| f.to_str()) {
        Some(name) => name,
        None => return false,
    };

    // Check glob patterns
    for pat in &filter.patterns {
        if matches_pattern(filename, pat) {
            return true;
        }
    }

    // Check MIME types if patterns did not match
    if !filter.mime_types.is_empty() {
        let guessed = mime_guess::from_path(path);
        for mime in &filter.mime_types {
            let mime_lower = mime.to_lowercase();
            if guessed.iter().any(|g| g.as_ref().to_lowercase() == mime_lower) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matching() {
        assert!(matches_pattern("photo.PNG", "*.png"));
        assert!(matches_pattern("photo.jpeg", "*.jpeg"));
        assert!(!matches_pattern("photo.jpg", "*.png"));
        assert!(matches_pattern("anything.txt", "*"));
        assert!(matches_pattern("anything.txt", "*.*"));
    }

    #[test]
    fn test_filter_matching() {
        let filter = FileFilter::images();
        assert!(matches_filter(Path::new("/tmp/test.png"), &filter));
        assert!(matches_filter(Path::new("/tmp/test.JPG"), &filter));
        assert!(!matches_filter(Path::new("/tmp/test.pdf"), &filter));
    }
}

