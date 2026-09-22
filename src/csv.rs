//! Minimal CSV writer/reader for benchmark matrices.

use std::fs;
use std::path::Path;

pub fn write_csv(path: &Path, header: &[&str], rows: &[Vec<String>]) -> Result<(), String> {
    let mut out = String::new();
    out.push_str(&header.join(","));
    out.push('\n');
    for row in rows {
        out.push_str(&row.join(","));
        out.push('\n');
    }
    fs::write(path, out).map_err(|e| format!("cannot write {}: {}", path.display(), e))
}

/// Parse a small CSV (no quoting, no embedded commas) into trimmed fields.
pub fn read_csv(text: &str) -> Vec<Vec<String>> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.split(',').map(|field| field.trim().to_string()).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("sortlab-csv-{}.csv", std::process::id()));
        write_csv(
            &path,
            &["algo", "pattern", "micros"],
            &[vec!["merge".into(), "random".into(), "123.5".into()]],
        )
        .unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let rows = read_csv(&text);
        assert_eq!(rows[0], vec!["algo", "pattern", "micros"]);
        assert_eq!(rows[1], vec!["merge", "random", "123.5"]);
        let _ = fs::remove_file(&path);
    }
}
