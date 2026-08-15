use crate::actionsapi::core::install;
use rust_i18n::t;
use std::fs;
use std::path::Path;

/// Bin dosyalarını /usr/bin dizinine kurar.
pub fn dobin<P: AsRef<Path>>(dest_root: &str, source: P) -> Result<(), String> {
    let path = Path::new(dest_root).join("usr/bin");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    install(source, path)
}

/// Sistem bin dosyalarını /usr/bin dizinine kurar (usrmerge).
pub fn dosbin<P: AsRef<Path>>(dest_root: &str, source: P) -> Result<(), String> {
    let sbindir = std::env::var("LUPPO_SBINDIR").unwrap_or_else(|_| "usr/bin".to_string());
    let target_dir = Path::new(dest_root).join(&sbindir);
    fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

    let filename = source
        .as_ref()
        .file_name()
        .map(|n| n.to_string_lossy().to_string());

    install(source, &target_dir)?;

    if sbindir == "usr/sbin" {
        return Ok(());
    }

    let sbin_link_dir = Path::new(dest_root).join("usr/sbin");
    if let Some(parent) = sbin_link_dir.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    if let Some(ref fname) = filename {
        let link_path = sbin_link_dir.join(fname);
        if !link_path.exists() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(format!("../bin/{}", fname), &link_path).map_err(|e| {
                t!(
                    "api_symlink_err",
                    error = format!("{} -> usr/sbin: {}", fname, e)
                )
                .to_string()
            })?;
        }
    }

    Ok(())
}

/// Kütüphaneleri /usr/lib veya emul32'de /usr/lib32 dizinine kurar.
pub fn dolib<P: AsRef<Path>>(dest_root: &str, source: P) -> Result<(), String> {
    let lib_subdir = if std::env::var("LUPPO_BUILD_TYPE").as_deref() == Ok("emul32") {
        "usr/lib32"
    } else {
        "usr/lib"
    };
    let path = Path::new(dest_root).join(lib_subdir);
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    install(source, path)
}

/// Manual sayfalarını /usr/share/man dizinine kurar.
pub fn doman<P: AsRef<Path>>(dest_root: &str, source: P) -> Result<(), String> {
    let path = Path::new(dest_root).join("usr/share/man");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    let src_str = source.as_ref().to_string_lossy();
    if src_str.contains('*') || src_str.contains('?') {
        if let Ok(entries) = glob::glob(&src_str) {
            for entry in entries.flatten() {
                install(entry, &path)?;
            }
        }
        Ok(())
    } else {
        install(source, path)
    }
}

/// Info sayfalarını /usr/share/info dizinine kurar.
pub fn doinfo<P: AsRef<Path>>(dest_root: &str, source: P) -> Result<(), String> {
    let path = Path::new(dest_root).join("usr/share/info");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    let src_str = source.as_ref().to_string_lossy();
    if src_str.contains('*') || src_str.contains('?') {
        if let Ok(entries) = glob::glob(&src_str) {
            for entry in entries.flatten() {
                install(entry, &path)?;
            }
        }
        Ok(())
    } else {
        install(source, path)
    }
}

/// Dokümantasyon dosyalarını /usr/share/doc/<paket_adi> dizinine kurar.
pub fn dodoc<P: AsRef<Path>>(dest_root: &str, pkg_name: &str, source: P) -> Result<(), String> {
    let path = Path::new(dest_root).join("usr/share/doc").join(pkg_name);
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    let src_str = source.as_ref().to_string_lossy();
    if src_str.contains('*') || src_str.contains('?') {
        if let Ok(entries) = glob::glob(&src_str) {
            for entry in entries.flatten() {
                install(entry, &path)?;
            }
        }
        Ok(())
    } else {
        install(source, path)
    }
}

/// C/C++ başlık dosyalarını /usr/include dizinine kurar.
pub fn install_headers<P: AsRef<Path>>(dest_root: &str, source: P) -> Result<(), String> {
    let path = Path::new(dest_root).join("usr/include");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;

    let src_str = source.as_ref().to_string_lossy();
    if src_str.contains('*') || src_str.contains('?') {
        if let Ok(entries) = glob::glob(&src_str) {
            for entry in entries.flatten() {
                install(&entry, &path)?;
            }
        }
        Ok(())
    } else {
        install(source, path)
    }
}

/// Bir dosya içindeki metni arar ve değiştirir (sed -i alternatifi).
pub fn dosed<P: AsRef<Path>>(
    path: P,
    search: &str,
    replace: &str,
    delete_line: bool,
) -> Result<(), String> {
    let p = path.as_ref();
    println!(
        "-> dosed: {} ({} -> {})",
        p.display(),
        search,
        if delete_line { "DELETE" } else { replace }
    );

    let content = fs::read_to_string(p)
        .map_err(|e| t!("api_err_dosed_open", path = p.display(), error = e).to_string())?;

    let re = regex::Regex::new(search).map_err(|e| t!("api_err_regex", error = e).to_string())?;

    // Python regex backreferences (\1..\9) → Rust regex ($1..$9)
    let mut rust_replace = replace.to_string();
    for i in (1..=9).rev() {
        let from = format!("\\{}", i);
        let to = format!("${}", i);
        rust_replace = rust_replace.replace(&from, &to);
    }
    eprintln!("[dosed debug] replace={:?} rust_replace={:?}", replace, rust_replace);
    let new_content = if delete_line {
        content
            .lines()
            .filter(|line| !re.is_match(line))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        re.replace_all(&content, &rust_replace).to_string()
    };

    fs::write(p, new_content)
        .map_err(|e| t!("api_err_dosed_write", path = p.display(), error = e).to_string())?;
    Ok(())
}
