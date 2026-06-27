//! Education mode example: demonstrate lifecycle stage detection.

/// A simplified stage detector for demonstration purposes.
pub fn count_markdown_files(dir: &std::path::Path) -> usize {
    if !dir.is_dir() {
        return 0;
    }
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.path().extension().map_or(false, |x| x == "md"))
                .count()
        })
        .unwrap_or(0)
}

/// Check if a file contains a marker string.
pub fn file_contains(path: &std::path::Path, marker: &str) -> bool {
    std::fs::read_to_string(path)
        .map(|c| c.contains(marker))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn count_zero_for_nonexistent_dir() {
        assert_eq!(count_markdown_files(Path::new("/nonexistent/path")), 0);
    }

    #[test]
    fn file_contains_returns_false_for_missing_file() {
        assert!(!file_contains(Path::new("/nonexistent/file.md"), "ADMITTED"));
    }
}
