use crate::actionsapi::core::run_command;
use rust_i18n::t;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;

/// ELF e_type sabitleri (ELF başlığı offset 16, 2 bayt Little Endian)
const ET_REL: u16 = 1;  // Relocatable (static object)
const ET_EXEC: u16 = 2; // Executable
const ET_DYN: u16 = 3;  // Shared object / PIE executable

/// ELF dosyasının türünü döndürür. ELF değilse None döner.
fn elf_type(path: &Path) -> Option<u16> {
    if let Ok(mut file) = fs::File::open(path) {
        let mut header = [0u8; 18];
        if file.read_exact(&mut header).is_ok() && &header[0..4] == b"\x7fELF" {
            // e_type: offset 16, 2 bayt little-endian
            let e_type = u16::from_le_bytes([header[16], header[17]]);
            return Some(e_type);
        }
    }
    None
}

/// Statik arşiv mi? (ar formatı: "!<arch>\n")
fn is_static_archive(path: &Path) -> bool {
    if let Ok(mut file) = fs::File::open(path) {
        let mut magic = [0u8; 8];
        if file.read_exact(&mut magic).is_ok() {
            return &magic == b"!<arch>\n";
        }
    }
    false
}

/// UsrMerge: /sbin ve /usr/sbin içindeki dosyaları /usr/bin'e taşır,
/// özgün konumlarına sembolik bağlantı oluşturur.
/// /bin içindeki dosyaları da /usr/bin'e taşır.
pub fn merge_usr_dirs(install_root: &str) -> Result<(), String> {
    let root = Path::new(install_root);
    let usr_bin = root.join("usr/bin");
    let usr_sbin = root.join("usr/sbin");
    let bin_dir = root.join("bin");
    let sbin_dir = root.join("sbin");

    let sbindir_override = std::env::var("LUPPO_SBINDIR");
    let merge_disabled = sbindir_override.as_deref() == Ok("usr/sbin");

    if bin_dir.exists() {
        fs::create_dir_all(&usr_bin).map_err(|e| e.to_string())?;
        for entry in fs::read_dir(&bin_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let file_name = entry.file_name();
            let src = entry.path();
            if src.is_file() || src.is_symlink() {
                let dst = usr_bin.join(&file_name);
                if !dst.exists() {
                    fs::rename(&src, &dst).map_err(|e| e.to_string())?;
                }
            }
        }
        if !bin_dir.is_symlink() {
            let _ = std::fs::remove_dir(&bin_dir);
            #[cfg(unix)]
            std::os::unix::fs::symlink("usr/bin", &bin_dir).ok();
        }
    }

    if sbin_dir.exists() {
        fs::create_dir_all(&usr_bin).map_err(|e| e.to_string())?;
        for entry in fs::read_dir(&sbin_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let file_name = entry.file_name();
            let src = entry.path();
            if src.is_file() || src.is_symlink() {
                let dst = usr_bin.join(&file_name);
                if !dst.exists() {
                    fs::rename(&src, &dst).map_err(|e| e.to_string())?;
                }
            }
        }
        if !sbin_dir.is_symlink() {
            let _ = std::fs::remove_dir(&sbin_dir);
            #[cfg(unix)]
            std::os::unix::fs::symlink("usr/bin", &sbin_dir).ok();
        }
    }

    if !merge_disabled && usr_sbin.exists() {
        fs::create_dir_all(&usr_bin).map_err(|e| e.to_string())?;
        let mut has_entries = false;
        for entry in fs::read_dir(&usr_sbin).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let file_name = entry.file_name();
            let src = entry.path();
            if src.is_file() || src.is_symlink() {
                has_entries = true;
                let dst = usr_bin.join(&file_name);
                if !dst.exists() {
                    fs::rename(&src, &dst).map_err(|e| {
                        format!(
                            "merge_usr_dirs: {} -> {}: {}",
                            src.display(),
                            dst.display(),
                            e
                        )
                    })?;
                }
                #[cfg(unix)]
                std::os::unix::fs::symlink(format!("../bin/{}", file_name.to_string_lossy()), &src)
                    .ok();
            }
        }
        if has_entries {
            println!("  ✔ usrmerge: /usr/sbin → /usr/bin (sembolik bağlar oluşturuldu)");
        }
    }

    if !merge_disabled && usr_sbin.exists() {
        for entry in fs::read_dir(&usr_sbin).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_type().map(|t| t.is_symlink()).unwrap_or(false) {
                continue;
            }
            let file_name = entry.file_name();
            let src = entry.path();
            if src.is_file() {
                let dst = usr_bin.join(&file_name);
                if !dst.exists() {
                    fs::rename(&src, &dst).map_err(|e| e.to_string())?;
                }
                #[cfg(unix)]
                std::os::unix::fs::symlink(format!("../bin/{}", file_name.to_string_lossy()), &src)
                    .ok();
            }
        }
    }

    println!("{}", t!("api_usrmerge_done"));
    Ok(())
}

/// Bir dosyayı türüne göre uygun strip parametresiyle temizler.
/// - ET_EXEC (çalıştırılabilir): --strip-all
/// - ET_DYN (paylaşımlı kütüphane / PIE): --strip-unneeded
/// - ET_REL (relocatable) ve statik arşiv (.a): --strip-debug
pub fn strip_file<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let p = path.as_ref();
    let path_str = p.to_str().ok_or("Geçersiz yol")?;

    // Önce statik arşiv kontrolü (.a dosyaları)
    if is_static_archive(p) {
        return run_command("strip", &["--strip-debug", path_str]);
    }

    // ELF türüne göre uygun parametreyi seç
    match elf_type(p) {
        Some(ET_EXEC) => {
            run_command("strip", &["--strip-all", path_str])
        }
        Some(ET_DYN) => {
            // Paylaşımlı kütüphane veya PIE executable:
            // --strip-all dinamik symbol table'ı kaldırabilir, --strip-unneeded güvenlidir.
            run_command("strip", &["--strip-unneeded", path_str])
        }
        Some(ET_REL) => {
            run_command("strip", &["--strip-debug", path_str])
        }
        Some(_) | None => {
            // Tanınmayan ELF türü veya ELF değil: atla
            Ok(())
        }
    }
}

/// Geriye dönük uyumluluk için `strip` ismi korunuyor; artık türe göre seçim yapar.
pub fn strip<P: AsRef<Path>>(path: P) -> Result<(), String> {
    strip_file(path)
}

/// Kurulum dizinindeki tüm ELF ikili ve statik arşiv dosyalarını otomatik temizler (strip).
/// `install_root`: strip_dir'in ilk çağrıldığı kök dizin (NoStrip eşleştirmesi için sabit kalır).
/// `exclude_prefixes`: Bu öneklerle başlayan yollar strip edilmez (NoStrip).
pub fn strip_dir<P: AsRef<Path>>(dest_dir: P, exclude_prefixes: &[String]) -> Result<(), String> {
    let root = dest_dir.as_ref();
    println!("{}", t!("api_strip_dir", path = root.display()));
    strip_dir_impl(root, root, exclude_prefixes)
}

/// Gerçek rekürsif yardımcı fonksiyon.
/// `install_root`: Orijinal kök dizin (NoStrip yolu hesaplaması için sabit tutulur).
/// `current_dir`:  O an taranan dizin.
fn strip_dir_impl(
    install_root: &Path,
    current_dir: &Path,
    exclude_prefixes: &[String],
) -> Result<(), String> {
    if !current_dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(current_dir).map_err(|e| e.to_string())?.flatten() {
        let path = entry.path();

        if path.is_dir() && !path.is_symlink() {
            // Alt dizine in; install_root sabit kalır
            strip_dir_impl(install_root, &path, exclude_prefixes)?;
        } else if path.is_file() {
            // NoStrip kontrolü: dosyanın install_root'a göre mutlak yolu
            let rel = path
                .strip_prefix(install_root)
                .unwrap_or(&path)
                .to_string_lossy();
            let abs_path = format!("/{}", rel);

            let excluded = exclude_prefixes.iter().any(|prefix| abs_path.starts_with(prefix.as_str()));
            if excluded {
                continue;
            }

            // ELF mi?
            if elf_type(&path).is_some() {
                if let Err(e) = strip_file(&path) {
                    eprintln!("strip uyarısı: {} — {}", path.display(), e);
                }
                continue;
            }

            // Statik arşiv (.a) mi?
            if is_static_archive(&path) {
                if let Err(e) = run_command(
                    "strip",
                    &["--strip-debug", path.to_str().unwrap_or("")],
                ) {
                    eprintln!("strip uyarısı (.a): {} — {}", path.display(), e);
                }
            }
        }
    }
    Ok(())
}

/// GNU Config güncellemesi (config.sub ve config.guess).
pub fn gnuconfig_update() -> Result<(), String> {
    println!("{}", t!("api_gnuconfig_update"));

    let search_dirs = vec![".".to_string(), "..".to_string(), "../..".to_string()];

    for dir in &search_dirs {
        let dir_path = Path::new(dir);
        for entry in fs::read_dir(dir_path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str == "config.sub" || name_str == "config.guess" {
                let path = entry.path();
                if path.is_file() {
                    let system_path = Path::new("/usr/share/gnuconfig").join(name_str.as_ref());
                    if system_path.exists() {
                        fs::copy(&system_path, &path).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }
    println!("{}", t!("api_gnuconfig_done"));
    Ok(())
}

/// .pc (pkg-config) dosyalarındaki gereksiz DESTDIR kayıtlarını düzeltir.
pub fn fix_pkgconfig(dest_root: &str) -> Result<(), String> {
    let root = Path::new(dest_root);
    let pkgconfig_dirs = vec![
        root.join("usr/lib/pkgconfig"),
        root.join("usr/share/pkgconfig"),
        root.join("usr/lib64/pkgconfig"),
    ];

    let destdir = format!("{}", dest_root);

    for dir in &pkgconfig_dirs {
        if !dir.exists() || !dir.is_dir() {
            continue;
        }
        let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_symlink() {
                continue;
            }
            if path.extension().map(|e| e == "pc").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    let new_content = content.replace(&destdir, "");
                    if new_content != content {
                        let _ = fs::write(&path, new_content);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Info dizini (dir) dosyasını günceller.
pub fn fix_info_dir(dest_root: &str) -> Result<(), String> {
    let info_dir = Path::new(dest_root).join("usr/share/info");
    if !info_dir.exists() || !info_dir.is_dir() {
        return Ok(());
    }

    let dir_file = info_dir.join("dir");
    if dir_file.exists() {
        fs::remove_file(&dir_file).map_err(|e| e.to_string())?;
    }

    let status = Command::new("install-info")
        .arg("--info-dir")
        .arg(info_dir.to_str().ok_or("Geçersiz yol")?)
        .status()
        .map_err(|e| e.to_string())?;

    if !status.success() {
        Command::new("install-info")
            .args(["--dir-file", dir_file.to_str().ok_or("Geçersiz yol")?])
            .status()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// HTML dokümantasyon dosyalarını /usr/share/doc/<pkg>/html/ altına kurar.
pub fn dohtml(dest_root: &str, sources: &[&str], dest_dir: Option<&str>) -> Result<(), String> {
    let pkg_name = std::env::var("SRC_NAME").unwrap_or_else(|_| "package".to_string());
    let html_base = Path::new(dest_root)
        .join("usr/share/doc")
        .join(&pkg_name)
        .join("html");

    let target = if let Some(sub) = dest_dir {
        html_base.join(sub)
    } else {
        html_base
    };

    fs::create_dir_all(&target).map_err(|e| e.to_string())?;

    for src in sources {
        let src_path = Path::new(src);
        let has_glob = src.contains('*') || src.contains('?') || src.contains('[');
        if has_glob {
            let entries: Vec<_> = glob::glob(src)
                .map_err(|e| format!("Glob hatası: {}", e))?
                .filter_map(|r| r.ok())
                .collect();
            for entry in &entries {
                let fname = entry.file_name().ok_or("Geçersiz dosya adı")?;
                let dst = target.join(fname);
                if entry.is_dir() {
                    run_command("cp", &["-pPR", entry.to_str().unwrap(), dst.to_str().unwrap()])?;
                } else {
                    fs::copy(entry, &dst).map_err(|e| e.to_string())?;
                }
            }
        } else if src_path.is_dir() {
            let entries = fs::read_dir(src_path).map_err(|e| e.to_string())?;
            for entry in entries.flatten() {
                let path = entry.path();
                let fname = path.file_name().ok_or("Geçersiz dosya adı")?;
                let dst = target.join(fname);
                fs::copy(&path, &dst).map_err(|e| e.to_string())?;
            }
        } else {
            let fname = src_path.file_name().ok_or("Geçersiz dosya adı")?;
            let dst = target.join(fname);
            fs::copy(src_path, &dst).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// .mo locale dosyasını doğru konuma kurar (.po dosyasını msgfmt ile derler).
pub fn domo(
    dest_root: &str,
    source: &str,
    locale: &str,
    destination_file: &str,
) -> Result<(), String> {
    let target_dir = Path::new(dest_root)
        .join("usr/share/locale")
        .join(locale)
        .join("LC_MESSAGES");
    fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
    run_command("msgfmt", &[source, "-o", "messages.mo"])?;
    fs::rename("messages.mo", target_dir.join(destination_file)).map_err(|e| e.to_string())
}

/// Yeni dokümantasyon kurulumu (hedef dosya adı belirtilebilir).
pub fn newdoc(dest_root: &str, source: &str, destination: &str) -> Result<(), String> {
    let pkg_name = std::env::var("SRC_NAME").unwrap_or_else(|_| "package".to_string());
    let dest_file = Path::new(destination)
        .file_name()
        .ok_or("Geçersiz hedef dosya")?
        .to_str()
        .unwrap();
    let target_dir = Path::new(dest_root).join("usr/share/doc").join(pkg_name);
    fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
    fs::copy(source, target_dir.join(dest_file))
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Yeni man sayfası kurulumu (hedef dosya adı belirtilebilir).
pub fn newman(dest_root: &str, source: &str, destination: &str) -> Result<(), String> {
    let dest_path = Path::new(destination);
    let section = dest_path
        .extension()
        .ok_or("Man sayfasının uzantısı (section) yok")?
        .to_str()
        .unwrap();
    let target_dir = Path::new(dest_root)
        .join("usr/share/man")
        .join(format!("man{}", section));
    fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
    fs::copy(source, target_dir.join(dest_path.file_name().unwrap()))
        .map(|_| ())
        .map_err(|e| e.to_string())
}
