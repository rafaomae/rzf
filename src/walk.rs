use std::{fs, io, path::Path};

pub fn walk_dir(root: &Path) -> io::Result<Vec<String>> {
    let mut files: Vec<String> = Vec::new();

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;

        if entry_type.is_dir() {
            if entry.path().ends_with(".git") {
                continue;
            }

            let mut dir_files = walk_dir(entry.path().as_path())?;
            files.append(&mut dir_files);
            continue;
        }

        if let Some(file) = entry.path().to_str() {
            let file = file.strip_prefix("./").unwrap_or(file);
            files.push(file.to_string());
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn create_temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("walk-{name}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn walks_current_dir() {
        let dir = create_temp_dir("current");

        std::fs::write(dir.join("a.txt"), "").unwrap();
        std::fs::write(dir.join("b.txt"), "").unwrap();
        std::fs::create_dir_all(dir.join("test")).unwrap();
        std::fs::write(dir.join("test/c.txt"), "").unwrap();

        let result = walk_dir(&dir).unwrap();
        assert!(result.iter().any(|p| p.ends_with("a.txt")));
        assert!(result.iter().any(|p| p.ends_with("b.txt")));
        assert!(result.iter().any(|p| p.ends_with("test/c.txt")));

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn does_not_include_directories() {
        let dir = create_temp_dir("not-include-dir");

        std::fs::write(dir.join("j.txt"), "").unwrap();
        std::fs::write(dir.join("k.txt"), "").unwrap();
        std::fs::create_dir_all(dir.join("test")).unwrap();
        std::fs::write(dir.join("test/l.txt"), "").unwrap();

        let result = walk_dir(&dir).unwrap();
        for path in &result {
            let p = Path::new(path);
            assert!(p.is_file(), "non-file in result: {:?}", p);
        }

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn walks_current_dir_not_include_git() {
        let dir = create_temp_dir("not-include-git");

        std::fs::write(dir.join("a.txt"), "").unwrap();
        std::fs::write(dir.join("b.txt"), "").unwrap();
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::write(dir.join(".git/l.txt"), "").unwrap();

        let result = walk_dir(&dir).unwrap();
        assert!(result.iter().any(|p| p.ends_with("a.txt")));
        assert!(result.iter().any(|p| p.ends_with("b.txt")));
        assert!(!result.iter().any(|p| p.ends_with(".git/l.txt")));

        std::fs::remove_dir_all(dir).unwrap();
    }
}
