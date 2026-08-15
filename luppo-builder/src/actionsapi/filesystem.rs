use crate::actionsapi::core::install;
use crate::actionsapi::core::run_command;
use rust_i18n::t;
use std::fs;
use std::path::Path;



/// Hedef dizin oluşturur (luppotools.dodir muadili).
pub fn dodir(dest_root: &str, dir: &str) -> Result<(), String> {
    let path = Path::new(dest_root).join(dir.trim_start_matches('/'));
    println!("{}", t!("api_dodir", path = path.display()));

    if path.symlink_metadata().is_ok() {
        if path.is_dir() {
            return Ok(());
        }
        fs::remove_file(&path).map_err(|e| {
            format!("{}: {}: {}", t!("api_dodir", path = path.display()), path.display(), e)
        })?;
    }
    fs::create_dir_all(&path).map_err(|e| {
        format!("{}: {}: {}", t!("api_dodir", path = path.display()), path.display(), e)
    })
}

/// Sembolik bağ oluşturur (luppotools.dosym muadili).
pub fn dosym(dest_root: &str, source: &str, destination: &str) -> Result<(), String> {
    let dest_path = Path::new(dest_root).join(destination.trim_start_matches('/'));
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    println!(
        "{}",
        t!("api_dosym", src = source, dest = dest_path.display())
    );
    #[cfg(unix)]
    return std::os::unix::fs::symlink(source, &dest_path)
        .map_err(|e| t!("api_symlink_err", error = e).to_string());
    #[cfg(not(unix))]
    Err(t!("api_symlink_unix_only").to_string())
}

/// Dosya taşıma (luppotools.domove muadili).
pub fn domove(dest_root: &str, source: &str, destination: &str) -> Result<(), String> {
    let dst_dir = Path::new(dest_root).join(destination.trim_start_matches('/'));
    fs::create_dir_all(&dst_dir).map_err(|e| e.to_string())?;

    let src_full = Path::new(dest_root).join(source.trim_start_matches('/'));

    let has_glob = source.contains('*') || source.contains('?') || source.contains('[');
    if has_glob {
        let parent = src_full.parent().unwrap_or(Path::new("."));
        let pattern = parent.join(src_full.file_name().unwrap_or_default());
        let pattern_str = pattern.to_string_lossy();
        let entries: Vec<_> = glob::glob(&pattern_str)
            .map_err(|e| format!("Glob pattern hatası: {}", e))?
            .filter_map(|r| r.ok())
            .collect();
        if entries.is_empty() {
            return Err(t!("api_move_err", src = source, error = "Glob eşleşmesi yok").to_string());
        }
        for entry in &entries {
            let fname = entry
                .file_name()
                .ok_or_else(|| "Dosya adı okunamadı".to_string())?;
            let dst_path = dst_dir.join(fname);
            println!(
                "{}",
                t!("api_domove", src = entry.display(), dest = dst_path.display())
            );
            fs::rename(entry, &dst_path)
                .map_err(|e| t!("api_move_err", src = entry.display(), error = e).to_string())?;
        }
        return Ok(());
    }

    let filename = src_full
        .file_name()
        .ok_or_else(|| "Kaynak dosya adı okunamadı".to_string())?;
    let dst_path = dst_dir.join(filename);

    println!(
        "{}",
        t!(
            "api_domove",
            src = src_full.display(),
            dest = dst_path.display()
        )
    );
    fs::rename(&src_full, &dst_path)
        .map_err(|e| t!("api_move_err", src = src_full.display(), error = e).to_string())
}

/// Dosya kopyalama (luppotools.insinto muadili).
/// Kaynak yol glob pattern'i içerebilir (örn. "sysui/*.xml")..
pub fn insinto(dest_root: &str, dest_dir: &str, source: &str) -> Result<(), String> {
    let full_dest = Path::new(dest_root).join(dest_dir.trim_start_matches('/'));
    fs::create_dir_all(&full_dest).map_err(|e| e.to_string())?;

    let src_path = Path::new(source);
    let filename = src_path
        .file_name()
        .ok_or_else(|| "Kaynak dosya adı okunamadı".to_string())?;

    // Glob pattern içeriyor mu? (*, ?, [)
    let has_glob = source.contains('*') || source.contains('?') || source.contains('[');
    if has_glob {
        let parent = src_path.parent().unwrap_or(Path::new("."));
        let pattern = parent.join(filename);
        let pattern_str = pattern.to_string_lossy();
        let entries: Vec<_> = glob::glob(&pattern_str)
            .map_err(|e| format!("Glob pattern hatası: {}", e))?
            .filter_map(|r| r.ok())
            .collect();
        if entries.is_empty() {
            return Err(t!("api_copy_err", error = format!("Glob eşleşmesi yok: {}", source)).to_string());
        }
        for entry in &entries {
            let fname = entry
                .file_name()
                .ok_or_else(|| "Dosya adı okunamadı".to_string())?;
            let dst_path = full_dest.join(fname);
            println!(
                "{}",
                t!("api_insinto", src = entry.display().to_string(), dest = dst_path.display())
            );
            if entry.is_symlink() {
                let target = std::fs::read_link(entry)
                    .map_err(|e| t!("api_copy_err", error = e).to_string())?;
                std::os::unix::fs::symlink(&target, &dst_path)
                    .map_err(|e| t!("api_copy_err", error = e).to_string())?;
            } else if entry.is_dir() {
                run_command("cp", &["-pPR", entry.to_str().unwrap(), dst_path.to_str().unwrap()])
                    .map_err(|e| t!("api_copy_err", error = e).to_string())?;
            } else {
                fs::copy(entry, &dst_path)
                    .map_err(|e| t!("api_copy_err", error = e).to_string())?;
            }
        }
        Ok(())
    } else if src_path.is_symlink() {
        let dst_path = full_dest.join(filename);
        println!(
            "{}",
            t!("api_insinto", src = source, dest = dst_path.display())
        );
        let target = std::fs::read_link(&src_path)
            .map_err(|e| t!("api_copy_err", error = e).to_string())?;
        std::os::unix::fs::symlink(&target, &dst_path)
            .map_err(|e| t!("api_copy_err", error = e).to_string())
    } else if src_path.is_dir() {
        let dst_path = full_dest.join(filename);
        println!(
            "{}",
            t!("api_insinto", src = source, dest = dst_path.display())
        );
        run_command("cp", &["-pPR", source, dst_path.to_str().unwrap()])
    } else {
        let dst_path = full_dest.join(filename);
        println!(
            "{}",
            t!("api_insinto", src = source, dest = dst_path.display())
        );
        fs::copy(src_path, &dst_path)
            .map(|_| ())
            .map_err(|e| t!("api_copy_err", error = e).to_string())
    }
}

/// Çalıştırılabilir dosya kurulumu (luppotools.doexe muadili).
pub fn doexe<P: AsRef<std::path::Path>>(
    dest_root: &str,
    source: P,
    dest_dir: &str,
) -> Result<(), String> {
    let full_dest = Path::new(dest_root).join(dest_dir.trim_start_matches('/'));
    fs::create_dir_all(&full_dest).map_err(|e| e.to_string())?;
    let src = source.as_ref();
    let dst = full_dest.join(src.file_name().ok_or("Geçersiz dosya adı")?);
    println!(
        "{}",
        t!("api_doexe", src = src.display(), dest = dst.display())
    );
    run_command(
        "install",
        &["-m0755", src.to_str().unwrap(), dst.to_str().unwrap()],
    )
}

/// Statik kütüphane kurulumu (luppotools.dolib_a muadili).
pub fn dolib_a<P: AsRef<std::path::Path>>(
    dest_root: &str,
    source: P,
    dest_dir: &str,
) -> Result<(), String> {
    let full_dest = Path::new(dest_root).join(dest_dir.trim_start_matches('/'));
    fs::create_dir_all(&full_dest).map_err(|e| e.to_string())?;
    let src = source.as_ref();
    let dst = full_dest.join(src.file_name().ok_or("Geçersiz dosya adı")?);
    println!(
        "{}",
        t!("api_dolib_a", src = src.display(), dest = dst.display())
    );
    run_command(
        "install",
        &["-m0644", src.to_str().unwrap(), dst.to_str().unwrap()],
    )
}

/// Dinamik kütüphane kurulumu (luppotools.dolib_so muadili).
pub fn dolib_so<P: AsRef<std::path::Path>>(
    dest_root: &str,
    source: P,
    dest_dir: &str,
) -> Result<(), String> {
    let full_dest = Path::new(dest_root).join(dest_dir.trim_start_matches('/'));
    fs::create_dir_all(&full_dest).map_err(|e| e.to_string())?;
    let src = source.as_ref();
    let dst = full_dest.join(src.file_name().ok_or("Geçersiz dosya adı")?);
    println!(
        "{}",
        t!("api_dolib_so", src = src.display(), dest = dst.display())
    );
    run_command(
        "install",
        &["-m0755", src.to_str().unwrap(), dst.to_str().unwrap()],
    )
}

/// Dizin silme (luppotools.removeDir muadili).
pub fn remove_dir(dest_root: &str, dir: &str) -> Result<(), String> {
    let path = Path::new(dest_root).join(dir.trim_start_matches('/'));
    println!("{}", t!("api_remove_dir", path = path.display()));
    if path.exists() && path.is_dir() {
        fs::remove_dir_all(&path)
            .map_err(|e| t!("api_remove_dir_err", path = path.display(), error = e).to_string())
    } else {
        Ok(())
    }
}

/// Pixmap dosyası kurulumu (luppotools.dopixmaps muadili).
pub fn dopixmaps<P: AsRef<std::path::Path>>(dest_root: &str, source: P) -> Result<(), String> {
    let path = Path::new(dest_root).join("usr/share/pixmaps");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    install(source, path)
}
