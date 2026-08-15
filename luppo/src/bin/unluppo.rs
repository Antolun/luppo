use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::exit;
use xz2::read::XzDecoder;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: unluppo <package.luppo> [target_dir]");
        exit(1);
    }

    let pkg_path = &args[1];
    if !Path::new(pkg_path).exists() {
        eprintln!("File not found: {}", pkg_path);
        exit(1);
    }

    let target = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from(".")
    };

    let file = match fs::File::open(pkg_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening package: {}", e);
            exit(1);
        }
    };

    let mut zip_archive = match zip::ZipArchive::new(file) {
        Ok(z) => z,
        Err(e) => {
            eprintln!("Error reading package (invalid ZIP): {}", e);
            exit(1);
        }
    };

    let install_dir = target.join("install");
    fs::create_dir_all(&install_dir).unwrap_or_else(|e| {
        eprintln!("Error creating install directory: {}", e);
        exit(1);
    });

    for i in 0..zip_archive.len() {
        let mut zip_file = match zip_archive.by_index(i) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Warning: could not read entry {}: {}", i, e);
                continue;
            }
        };

        let name = zip_file.name().to_string();

        if name == "install.tar.xz" {
            let mut compressed = Vec::new();
            if zip_file.read_to_end(&mut compressed).is_err() {
                eprintln!("Warning: could not read install.tar.xz");
                continue;
            }
            let decompressor = XzDecoder::new(&compressed[..]);
            let mut tar_archive = tar::Archive::new(decompressor);
            if let Err(e) = extract_tar(&mut tar_archive, &install_dir) {
                eprintln!("Warning: could not extract install.tar.xz: {}", e);
            }
        } else if name == "comar/" || name.starts_with("comar/") {
            let out_path = target.join(&name);
            if zip_file.name().ends_with('/') {
                fs::create_dir_all(&out_path).ok();
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).ok();
                }
                let mut content = Vec::new();
                if zip_file.read_to_end(&mut content).is_ok() {
                    fs::write(&out_path, &content).ok();
                }
            }
        } else if name == "metadata.xml" || name == "files.xml" {
            let out_path = target.join(&name);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            let mut content = Vec::new();
            if zip_file.read_to_end(&mut content).is_ok() {
                fs::write(&out_path, &content).ok();
            }
        }
    }
}

fn extract_tar<R: Read>(archive: &mut tar::Archive<R>, dest: &Path) -> Result<(), String> {
    for entry_result in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry_result.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?;
        let full_path = path.to_string_lossy().to_string();

        let rel_path = if full_path.starts_with("install/") {
            &full_path["install/".len()..]
        } else {
            &full_path
        };

        if rel_path.is_empty() {
            continue;
        }

        let out_path = dest.join(rel_path);

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let entry_type = entry.header().entry_type();
        if entry_type.is_symlink() {
            let link_target = entry.link_name().map_err(|e| e.to_string())?;
            if let Some(target) = link_target {
                std::os::unix::fs::symlink(&target, &out_path).map_err(|e| e.to_string())?;
            }
        } else if entry_type.is_file() {
            entry.unpack(&out_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
