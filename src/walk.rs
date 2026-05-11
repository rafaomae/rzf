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
    use std::path::Path;

    #[test]
    fn walks_current_dir() {
        let result = walk_dir(Path::new(".")).unwrap();
        assert!(result.iter().any(|p| p.ends_with("Cargo.toml")));
        assert!(result.iter().any(|p| p.contains("main.rs")));
    }

    #[test]
    fn does_not_include_directories() {
        let result = walk_dir(Path::new(".")).unwrap();
        for path in &result {
            let p = Path::new(path);
            assert!(p.is_file(), "non-file in result: {:?}", p);
        }
    }
}
