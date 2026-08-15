use luppo_spec::models::LuppoSpec;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

pub fn reset_history(path: &Path) -> Result<(), String> {
    let canonical_path = path.canonicalize().map_err(|e| e.to_string())?;
    let parent = canonical_path
        .parent()
        .ok_or("Cannot get parent directory")?;
    let backup_dir = parent.join("backup");

    if backup_dir.exists() {
        let mut suffix = 0;
        let base_name = "backup";
        if let Ok(entries) = fs::read_dir(parent) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with(base_name) {
                    let ext = name.replacen(base_name, "", 1);
                    if let Ok(num) = ext.parse::<u32>() {
                        if num > suffix {
                            suffix = num;
                        }
                    }
                }
            }
        }
        suffix += 1;
        let suffix_str = if suffix < 10 {
            format!("0{}", suffix)
        } else {
            suffix.to_string()
        };
        let new_backup_name = format!("{}{}", base_name, suffix_str);
        let new_backup_path = parent.join(new_backup_name);
        fs::rename(&backup_dir, &new_backup_path).map_err(|e| e.to_string())?;
    }

    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

    // ── XML lopec dosyalarını bul ve işle (varsayılan) ──
    let mut lopec_files = Vec::new();
    find_lopec_files(&canonical_path, "lopec.xml", &mut lopec_files).map_err(|e| e.to_string())?;

    for f in lopec_files {
        let relative = f.strip_prefix(&canonical_path).unwrap();
        let backup_file_path = backup_dir.join(relative);

        if let Some(parent) = backup_file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let original_content = fs::read_to_string(&f).map_err(|e| e.to_string())?;
        fs::write(&backup_file_path, &original_content).map_err(|e| e.to_string())?;

        let reset_content = do_reset_history_xml(&original_content);
        fs::write(&f, reset_content).map_err(|e| e.to_string())?;
        println!("Reset history for {}", f.display());
    }

    // KDL lopec dosyalarını bul ve işle
    let mut kdl_files = Vec::new();
    find_lopec_files(&canonical_path, "lopec.kdl", &mut kdl_files).map_err(|e| e.to_string())?;
    for f in kdl_files {
        let relative = f.strip_prefix(&canonical_path).unwrap();
        let backup_file_path = backup_dir.join(relative);
        if let Some(parent) = backup_file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let original = fs::read_to_string(&f).map_err(|e| e.to_string())?;
        fs::write(&backup_file_path, &original).map_err(|e| e.to_string())?;
        let reset = do_reset_history_kdl(&original)?;
        fs::write(&f, reset).map_err(|e| e.to_string())?;
        println!("Reset history for {}", f.display());
    }

    Ok(())
}

fn find_lopec_files(dir: &Path, filename: &str, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            find_lopec_files(&path, filename, files)?;
        } else if path.file_name().and_then(|n| n.to_str()) == Some(filename) {
            files.push(path);
        }
    }
    Ok(())
}

// ── XML reset (regex tabanlı) ──

fn do_reset_history_xml(spec: &str) -> String {
    let release_regex = Regex::new(r#"(?i)(.*release\s*=\s*)["']\d+["'](.*)"#).unwrap();
    let mut newspec = Vec::new();
    let mut history = false;
    let mut update1 = false;
    let mut skipline = false;

    for line in spec.lines() {
        let check_line = line.to_string();
        if check_line.contains("</History>") {
            history = false;
        }

        if check_line.contains("<History>") {
            history = true;
        }

        if history {
            if check_line.contains("</Update>") {
                if !update1 {
                    newspec.push(check_line.clone());
                }
                update1 = true;
            }

            if update1 {
                continue;
            } else {
                let mut modified_line = check_line.clone();
                if check_line.contains("<Update") {
                    modified_line = release_regex
                        .replace(&check_line, r#"${1}"1"${2}"#)
                        .to_string();
                } else if check_line.contains("<Comment") && !check_line.contains("</Comment") {
                    skipline = true;
                }

                if check_line.contains("</Comment") {
                    skipline = false;
                    modified_line = "            <Comment>First release</Comment>".to_string();
                }

                if !skipline {
                    newspec.push(modified_line);
                }
            }
        } else {
            newspec.push(check_line);
        }
    }

    newspec.join("\n") + "\n"
}

// ── KDL reset (metin tabanlı) ──

fn do_reset_history_kdl(content: &str) -> Result<String, String> {
    let spec: LuppoSpec = luppo_spec::kdl::parse_kdl_spec_from_str(content)?;

    let history = match spec.history {
        Some(ref h) => h,
        None => return Ok(content.to_string()),
    };

    let first = match history.updates.first() {
        Some(u) => u,
        None => return Ok(content.to_string()),
    };

    // Find History { ... } block by brace counting
    let mut result = String::new();
    let mut depth: i32 = 0;
    let mut in_history = false;
    let mut indent = String::new();

    for line in content.lines() {
        if in_history {
            depth += line.chars().filter(|&c| c == '{').count() as i32;
            depth -= line.chars().filter(|&c| c == '}').count() as i32;

            if depth <= 0 {
                in_history = false;
                let block = make_history_block(first, &indent);
                result.push_str(&block);
            }
        } else {
            let trimmed = line.trim();
            if trimmed.starts_with("History") && trimmed.contains('{') {
                in_history = true;
                indent = line.chars().take_while(|&c| c == ' ').collect();
                depth = line.chars().filter(|&c| c == '{').count() as i32;
                depth -= line.chars().filter(|&c| c == '}').count() as i32;
                if depth <= 0 {
                    in_history = false;
                    result.push_str(&make_history_block(first, &indent));
                }
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
    }

    Ok(result)
}

fn make_history_block(first: &luppo_spec::models::Update, indent: &str) -> String {
    let ci = format!("{indent}    "); // child indent
    let gci = format!("{ci}    "); // grandchild indent
    format!(
        "{indent}History {{\n\
         {ci}Update release=1 date=\"{}\" {{\n\
         {gci}Version \"{}\"\n\
         {gci}Comment \"First release\"\n\
         {gci}Name \"{}\"\n\
         {gci}Email \"{}\"\n\
         {ci}}}\n\
         {indent}}}\n",
        first.date, first.version, first.committer, first.email
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset_history_kdl_roundtrip() {
        let input = r#"
LuppoPackage {
    Source {
        Name "testpkg"
    }
    History {
        Update release=3 date="2024-01-01" {
            Version "1.0.0"
            Comment "Some update"
            Name "dev"
            Email "dev@test.com"
        }
        Update release=2 date="2023-01-01" {
            Version "0.9.0"
            Comment "Initial"
            Name "dev"
            Email "dev@test.com"
        }
    }
}
"#;
        let result = do_reset_history_kdl(input).unwrap();
        let spec = luppo_spec::kdl::parse_kdl_spec_from_str(&result).unwrap();
        let hist = spec.history.expect("history should exist");
        assert_eq!(hist.updates.len(), 1);
        assert_eq!(hist.updates[0].release, 1);
        assert_eq!(hist.updates[0].comment, "First release");
    }
}
