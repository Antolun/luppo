use luppo_spec::models::{PathDef, LuppoSpec};
use rust_i18n::t;
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tar::Builder;
use xz2::write::XzEncoder;
use zip::ZipWriter;
use zip::write::FileOptions;

fn is_excluded_path(name: &str) -> bool {
    name.ends_with(".a") || name.ends_with(".la") || name.ends_with(".pyc") || name.ends_with(".pyo")
}

/// Representation of a file in files.xml
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub file_type: String,
    pub size: u64,
    pub uid: String,
    pub gid: String,
    pub mode: String,
    pub hash: String,
    pub permanent: bool,
}

/// Python Luppo'nin `package_filename()` fonksiyonuyla uyumlu paket adı üretir.
/// Format: <name>-<version>-<release>-<distro_id>-<arch>.luppo
pub fn make_package_filename(
    name: &str,
    version: &str,
    release: u32,
    distro_id: &str,
    arch: &str,
) -> String {
    format!(
        "{}-{}-{}-{}-{}.luppo",
        name, version, release, distro_id, arch
    )
}

/// Tüm install_root dizinini bir kez tarayıp dosya indeksi oluşturur.
/// Her dosyanın metaverisi (SHA1, boyut, izin vb.) önceden hesaplanır.
pub fn build_file_index(install_root: &Path) -> Result<Vec<FileInfo>, String> {
    let mut files = Vec::new();
    collect_files_recursive(install_root, install_root, None, &mut files)?;
    // Normalize all paths to avoid double slashes
    for fi in &mut files {
        fi.path = normalize_path(&fi.path);
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn file_matches_path_def(rel_path: &str, path_def: &PathDef) -> bool {
    let clean = path_def.path.trim_start_matches('/');
    // Glob pattern içeriyor mu?
    if clean.contains('*') || clean.contains('?') || clean.contains('[') {
        let pattern = format!("/{}", clean);
        glob::Pattern::new(&pattern)
            .map(|p| p.matches(&format!("/{}", rel_path)))
            .unwrap_or(false)
    } else if clean.ends_with('/') || clean.is_empty() {
        // Dizin: path rel_path ile başlıyor mu?
        let dir_prefix = clean.trim_end_matches('/');
        dir_prefix.is_empty() || rel_path.starts_with(dir_prefix)
    } else {
        // Önce tam eşleşme dene; olmazsa dizin öneki olarak kabul et
        if rel_path == clean {
            return true;
        }
        rel_path.starts_with(&format!("{}/", clean))
    }
}

fn is_subpath(parent: &str, child: &str) -> bool {
    let p_parts: Vec<&str> = parent.split('/').filter(|s| !s.is_empty()).collect();
    let c_parts: Vec<&str> = child.split('/').filter(|s| !s.is_empty()).collect();
    if p_parts.len() > c_parts.len() {
        return false;
    }
    for i in 0..p_parts.len() {
        if p_parts[i] != c_parts[i] {
            return false;
        }
    }
    true
}

fn is_excluded(path: &str, exclude_paths: &std::collections::HashSet<String>) -> bool {
    // path starts with '/' e.g., "/usr/lib/libreoffice/..."
    for excl in exclude_paths {
        // excl starts with '/' e.g., "/usr/lib/libreoffice/help" or contains glob like "/usr/share/icons/*"
        if excl.contains('*') || excl.contains('?') || excl.contains('[') {
            if glob::Pattern::new(excl).map(|p| p.matches(path)).unwrap_or(false) {
                return true;
            }
        }
        if is_subpath(excl, path) {
            return true;
        }
    }
    false
}

/// Önceden oluşturulmuş dosya indeksini PathDef filtresine göre daraltır.
/// Bu sayede her paket için ayrı ayrı dosya sistemi taranmaz.
/// Not: fileType yalnızca files.xml meta verisi içindir, dosya seçimini etkilemez.
fn normalize_path(path: &str) -> String {
    let trimmed = path.trim_start_matches('/');
    let normalized = Path::new(trimmed).components().collect::<PathBuf>();
    normalized.to_string_lossy().to_string()
}

fn filter_file_index(
    file_index: &[FileInfo],
    path_defs: &[PathDef],
    exclude_paths: &std::collections::HashSet<String>,
) -> Vec<FileInfo> {
    if path_defs.is_empty() {
        return file_index
            .iter()
            .filter(|fi| !is_excluded_path(&fi.path) && !is_excluded(&fi.path, exclude_paths))
            .cloned()
            .collect();
    }
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for path_def in path_defs {
        let ft_override = path_def.file_type.as_deref();
        for fi in file_index {
            if is_excluded_path(&fi.path) {
                continue;
            }
            if is_excluded(&fi.path, exclude_paths) {
                continue;
            }
            let rel = fi.path.trim_start_matches('/');
            if !file_matches_path_def(rel, path_def) {
                continue;
            }
            if seen.insert(fi.path.clone()) {
                let mut fi_clone = fi.clone();
                // Normalize path to ensure no double slashes
                fi_clone.path = normalize_path(&fi_clone.path);
                if let Some(ft) = ft_override {
                    fi_clone.file_type = ft.to_string();
                }
                result.push(fi_clone);
            }
        }
    }
    result.sort_by(|a, b| a.path.cmp(&b.path));
    result
}

pub fn check_path_collision(
    pkg_idx: usize,
    packages: &[luppo_spec::models::PackageDefinition],
) -> std::collections::HashSet<String> {
    let mut collisions = std::collections::HashSet::new();
    let current_pkg = &packages[pkg_idx];
    for (i, pkg) in packages.iter().enumerate() {
        if i == pkg_idx {
            continue;
        }
        for pinfo in &current_pkg.files.paths {
            for other_path in &pkg.files.paths {
                if is_subpath(&pinfo.path, &other_path.path) {
                    if collisions.insert(other_path.path.clone()) {
                        eprintln!("{}",
                            t!("package_path_collision_warning",
                                pkg = &current_pkg.name,
                                path = &pinfo.path,
                                other = &pkg.name,
                                other_path = &other_path.path
                            )
                        );
                    }
                } else if pinfo.path.contains('*') || pinfo.path.contains('?') || pinfo.path.contains('[') {
                    if glob::Pattern::new(&pinfo.path).map(|p| p.matches(&other_path.path)).unwrap_or(false) {
                        if collisions.insert(other_path.path.clone()) {
                            eprintln!("{}",
                                t!("package_path_collision_warning",
                                    pkg = &current_pkg.name,
                                    path = &pinfo.path,
                                    other = &pkg.name,
                                    other_path = &other_path.path
                                )
                            );
                        }
                    }
                }
            }
        }
    }
    collisions
}

/// Belirtilen paket için .luppo arşivi oluşturur.
///
/// - `install_root`: Bu paketin dosyalarının bulunduğu dizin
/// - `output_dir`: .luppo dosyasının yazılacağı dizin (pkg_dir)
/// - `distro_id`: Config'den gelen distribution_id ("p2")
/// - `arch`: Config'den gelen architecture ("x86_64")
/// - `specdir`: COMAR betiklerinin aranacağı dizin
/// - `file_index`: Önceden oluşturulmuş dosya indeksi (None = geleneksel tarama)
pub fn create_luppo_package(
    spec: &LuppoSpec,
    pkg_idx: usize,
    install_root: &Path,
    output_dir: &Path,
    distro_id: &str,
    arch: &str,
    specdir: &Path,
    file_index: Option<&[FileInfo]>,
    exclude_paths: &std::collections::HashSet<String>,
) -> Result<PathBuf, String> {
    let pkg = spec
        .packages
        .get(pkg_idx)
        .ok_or("Package index out of bounds")?;
    let (version, release) = spec
        .history
        .as_ref()
        .and_then(|h| h.updates.first())
        .map(|u| (u.version.as_str(), u.release))
        .unwrap_or(("0.0.0", 1));

    let luppo_filename = make_package_filename(&pkg.name, version, release, distro_id, arch);
    let luppo_path = output_dir.join(&luppo_filename);

    // 1. Dosyaları topla (PathDef filtresine göre)
    // Önceden oluşturulmuş indeks varsa onu kullan (çok daha hızlı)
    let file_infos = if let Some(index) = file_index {
        filter_file_index(index, &pkg.files.paths, exclude_paths)
    } else {
        collect_package_files(install_root, &pkg.files.paths, exclude_paths)?
    };

    println!("Package {}: paths={}, exclude_paths={}, collected={}", pkg.name, pkg.files.paths.len(), exclude_paths.len(), file_infos.len());

    if file_infos.is_empty() {
        eprintln!(
            "⚠ Uyarı: Paket '{}' için install dizininde eşleşen dosya bulunamadı, paket atlanıyor.",
            pkg.name
        );
        return Ok(PathBuf::new()); // boş path → çağıran tarafından filtrele
    }


    // 2. InstalledSize: Sıkıştırılmamış dosya boyutları toplamı (Python Luppo uyumu)
    let installed_size: u64 = file_infos.iter().map(|f| f.size).sum();

    // 3. files.xml içeriğini üret
    let files_xml = generate_files_xml(&file_infos);

    // 4. install.tar.xz oluştur (dosyalar "install/" prefix'li)
    let tar_xz_path = output_dir.join(format!("{}-install.tar.xz", pkg.name));
    create_tar_xz_from_files(install_root, &file_infos, &tar_xz_path)?;

    // 5. install.tar.xz hash'ini hesapla
    let (tar_xz_size, tar_xz_hash) = compute_file_stats(&tar_xz_path)?;

    // 6. metadata.xml üret
    let metadata_xml = generate_metadata_xml(
        spec,
        pkg_idx,
        &tar_xz_hash,
        tar_xz_size,
        installed_size,
        version,
        release,
        arch,
        distro_id,
    )?;

    // 7. .luppo ZIP arşivini oluştur
    create_zip_archive(
        &luppo_path,
        &tar_xz_path,
        &files_xml,
        &metadata_xml,
        specdir,
        pkg,
    )?;

    // Geçici tar.xz dosyasını temizle
    let _ = fs::remove_file(&tar_xz_path);

    println!("📦 Paket oluşturuldu: {}", luppo_path.display());
    Ok(luppo_path)
}

/// Python Luppo'nin `gen_files_xml()` mantığıyla uyumlu dosya toplama.
/// PathDef listesindeki yollara göre install_root içindeki dosyaları filtreler.
/// PathDef listesi boşsa tüm dosyaları toplar (fallback).
fn collect_package_files(
    install_root: &Path,
    path_defs: &[PathDef],
    exclude_paths: &std::collections::HashSet<String>,
) -> Result<Vec<FileInfo>, String> {
    let mut files = Vec::new();

    if path_defs.is_empty() {
        // Filtre yoksa tüm dosyaları al
        collect_files_recursive(install_root, install_root, None, &mut files)?;
        files.retain(|fi| !is_excluded(&fi.path, exclude_paths));
    } else {
        for path_def in path_defs {
            // Path tanımı "/" ile başlamalı. install_root + path_def.path
            let clean_path = path_def.path.trim_start_matches('/');
            let abs_path = install_root.join(clean_path);

            if abs_path.is_dir() {
                // Dizinse altındaki tüm dosyaları ekle
                collect_files_recursive(
                    install_root,
                    &abs_path,
                    path_def.file_type.as_deref(),
                    &mut files,
                )?;
            } else if abs_path.exists() {
                // Tek dosya
                let file_info =
                    build_file_info(install_root, &abs_path, path_def.file_type.as_deref())?;
                files.push(file_info);
            } else {
                // Glob: *.so, /usr/lib/*.la gibi
                let pattern = abs_path.to_str().unwrap_or("");
                for entry in glob::glob(pattern).map_err(|e| e.to_string())? {
                    match entry {
                        Ok(p) if p.is_file() => {
                            let fi =
                                build_file_info(install_root, &p, path_def.file_type.as_deref())?;
                            files.push(fi);
                        }
                        Ok(p) if p.is_dir() => {
                            collect_files_recursive(
                                install_root,
                                &p,
                                path_def.file_type.as_deref(),
                                &mut files,
                            )?;
                        }
                        _ => {}
                    }
                }
            }
        }
        files.retain(|fi| !is_excluded(&fi.path, exclude_paths));
    }

    // Normalize all paths to avoid double slashes
    for fi in &mut files {
        fi.path = normalize_path(&fi.path);
    }

    // Yola göre sırala (Python Luppo lzma sıkıştırmayı iyileştirmek için sıralıyor)
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn build_file_info(
    root: &Path,
    path: &Path,
    file_type_override: Option<&str>,
) -> Result<FileInfo, String> {
    use std::os::unix::fs::MetadataExt;

    let rel_path = path.strip_prefix(root).map_err(|e| e.to_string())?;

    // Normalize path to avoid double slashes from symlink traversal or filesystem quirks
    let rel_path_str: String = rel_path.to_string_lossy().into_owned();
    let normalized_rel = Path::new(&rel_path_str).components().collect::<PathBuf>();
    let rel_path_normalized = normalized_rel.to_string_lossy();

    let symlink_meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;

    let (size, hash, file_type, mode_val, uid, gid) = if symlink_meta.file_type().is_symlink() {
        (
            0,
            String::new(),
            "symlink".to_string(),
            symlink_meta.mode() & 0o7777,
            symlink_meta.uid(),
            symlink_meta.gid(),
        )
    } else {
        let (s, h) = compute_file_stats(path)?;
        let mode_val = symlink_meta.mode() & 0o7777;
        let ft = if let Some(ft) = file_type_override {
            ft.to_string()
        } else if mode_val & 0o111 != 0 {
            "executable".to_string()
        } else {
            "data".to_string()
        };
        (s, h, ft, mode_val, symlink_meta.uid(), symlink_meta.gid())
    };

    let mode = format!("{:04o}", mode_val);

    Ok(FileInfo {
        path: rel_path_normalized.to_string(),
        file_type,
        size,
        uid: uid.to_string(),
        gid: gid.to_string(),
        mode,
        hash,
        permanent: false,
    })
}

fn collect_files_recursive(
    root: &Path,
    current: &Path,
    file_type_override: Option<&str>,
    files: &mut Vec<FileInfo>,
) -> Result<(), String> {
    let meta = fs::symlink_metadata(current).map_err(|e| e.to_string())?;
    if meta.file_type().is_dir() {
        for entry in fs::read_dir(current).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            collect_files_recursive(root, &entry.path(), file_type_override, files)?;
        }
    } else {
        if let Some(name) = current.file_name().and_then(|n| n.to_str()) {
            if is_excluded_path(name) {
                return Ok(());
            }
        }
        let fi = build_file_info(root, current, file_type_override)?;
        files.push(fi);
    }
    Ok(())
}

fn create_tar_xz_from_files(
    install_root: &Path,
    file_infos: &[FileInfo],
    dst_file: &Path,
) -> Result<(), String> {
    let tar_xz = File::create(dst_file).map_err(|e| format!("Failed to create tar.xz: {}", e))?;
    let enc = XzEncoder::new(tar_xz, 9);
    let mut builder = Builder::new(enc);
    builder.follow_symlinks(false);

    for fi in file_infos {
        // files.xml'deki path "/" ile başlar → "install/usr/bin/..." olarak tar'a ekle
        let rel = fi.path.trim_start_matches('/');
        // Normalize to avoid double slashes
        let rel_normalized: String = Path::new(rel).components().collect::<PathBuf>().to_string_lossy().into_owned();
        let full_path = install_root.join(&rel_normalized);
        if fs::symlink_metadata(&full_path).is_ok() {
            let tar_path = format!("install/{}", rel_normalized);
            builder
                .append_path_with_name(&full_path, &tar_path)
                .map_err(|e| format!("tar append error for {}: {}", rel_normalized, e))?;
        }
    }

    builder
        .into_inner()
        .map_err(|e| format!("Failed to finish tar: {}", e))?
        .finish()
        .map_err(|e| format!("Failed to finish xz compression: {}", e))?;

    Ok(())
}

fn create_zip_archive(
    zip_path: &Path,
    tar_xz_path: &Path,
    files_xml: &str,
    metadata_xml: &str,
    specdir: &Path,
    _pkg: &luppo_spec::models::PackageDefinition,
) -> Result<(), String> {
    let file = File::create(zip_path).map_err(|e| format!("Failed to create .luppo file: {}", e))?;
    let mut zip = ZipWriter::new(file);
    let opts = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // metadata.xml
    zip.start_file("metadata.xml", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(metadata_xml.as_bytes())
        .map_err(|e| e.to_string())?;

    // files.xml
    zip.start_file("files.xml", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(files_xml.as_bytes())
        .map_err(|e| e.to_string())?;

    // install.tar.xz
    zip.start_file("install.tar.xz", opts)
        .map_err(|e| e.to_string())?;
    let mut tar_xz_file = File::open(tar_xz_path).map_err(|e| e.to_string())?;
    std::io::copy(&mut tar_xz_file, &mut zip).map_err(|e| e.to_string())?;

    // COMAR betikleri: specdir/comar/<script> → comar/<script>
    let comar_dir = specdir.join("comar");
    if comar_dir.is_dir() {
        for entry in fs::read_dir(&comar_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name();
            let script_name = name.to_string_lossy();
            let zip_entry_name = format!("comar/{}", script_name);
            zip.start_file(&zip_entry_name, opts)
                .map_err(|e| e.to_string())?;
            let mut f = File::open(entry.path()).map_err(|e| e.to_string())?;
            std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn compute_file_stats(path: &Path) -> Result<(u64, String), String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let size = file.metadata().map_err(|e| e.to_string())?.len();

    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok((size, format!("{:x}", hasher.finalize())))
}

fn generate_files_xml(files: &[FileInfo]) -> String {
    let mut xml = String::from("<?xml version=\"1.0\" ?>\n<Files>\n");
    for f in files {
        xml.push_str(&format!(
            "    <File>\n        <Path>{}</Path>\n        <Type>{}</Type>\n        <Size>{}</Size>\n        <Uid>{}</Uid>\n        <Gid>{}</Gid>\n        <Mode>{}</Mode>\n        <Hash>{}</Hash>\n",
            f.path, f.file_type, f.size, f.uid, f.gid, f.mode, f.hash
        ));
        if f.permanent {
            xml.push_str("        <Permanent>true</Permanent>\n");
        }
        xml.push_str("    </File>\n");
    }
    xml.push_str("</Files>\n");
    xml
}

#[allow(clippy::too_many_arguments)]
fn generate_metadata_xml(
    spec: &LuppoSpec,
    pkg_idx: usize,
    install_tar_hash: &str,
    package_size: u64,
    installed_size: u64,
    _version: &str,
    _release: u32,
    arch: &str,
    _distro_id: &str,
) -> Result<String, String> {
    let pkg = &spec.packages[pkg_idx];

    // Summary ve Description: pakette yoksa source'tan al (Python Luppo uyumu)
    let summary = if !pkg.summary.is_empty() {
        &pkg.summary
    } else {
        spec.source.summary.as_deref().unwrap_or("")
    };
    let description = if !pkg.description.is_empty() {
        &pkg.description
    } else {
        spec.source.description.as_deref().unwrap_or("")
    };

    let distribution = "LupuS";
    let distribution_release = "all";

    let mut xml = String::from("<?xml version=\"1.0\" ?>\n<LUPPO>\n");

    // <Source>
    xml.push_str("    <Source>\n");
    xml.push_str(&format!("        <Name>{}</Name>\n", spec.source.name));
    if let Some(ref hp) = spec.source.homepage {
        xml.push_str(&format!("        <Homepage>{}</Homepage>\n", hp));
    }
    if let Some(ref p) = spec.source.packager {
        xml.push_str("        <Packager>\n");
        xml.push_str(&format!("            <Name>{}</Name>\n", p.name));
        xml.push_str(&format!("            <Email>{}</Email>\n", p.email));
        xml.push_str("        </Packager>\n");
    }
    xml.push_str("    </Source>\n");

    // <Package>
    xml.push_str("    <Package>\n");
    xml.push_str(&format!("        <Name>{}</Name>\n", pkg.name));

    // Tüm benzersiz çeviri dillerini topla ve sırala
    let mut langs = std::collections::HashSet::new();
    for lang in pkg.translations.keys() {
        langs.insert(lang.clone());
    }
    for lang in spec.source.translations.keys() {
        langs.insert(lang.clone());
    }
    let mut sorted_langs: Vec<String> = langs.into_iter().collect();
    sorted_langs.sort();

    // 1. Özet Çevirileri
    if !summary.is_empty() {
        xml.push_str(&format!(
            "        <Summary xml:lang=\"en\">{}</Summary>\n",
            summary
        ));
    }
    for lang in &sorted_langs {
        if lang == "en" {
            continue;
        }
        if let Some(trans_sum) = pkg
            .translations
            .get(lang)
            .and_then(|t| t.summary.as_ref())
            .or_else(|| {
                spec.source
                    .translations
                    .get(lang)
                    .and_then(|t| t.summary.as_ref())
            })
        {
            if !trans_sum.is_empty() {
                xml.push_str(&format!(
                    "        <Summary xml:lang=\"{}\">{}</Summary>\n",
                    lang, trans_sum
                ));
            }
        }
    }

    // 2. Açıklama Çevirileri
    if !description.is_empty() {
        xml.push_str(&format!(
            "        <Description xml:lang=\"en\">{}</Description>\n",
            description
        ));
    }
    for lang in &sorted_langs {
        if lang == "en" {
            continue;
        }
        if let Some(trans_desc) = pkg
            .translations
            .get(lang)
            .and_then(|t| t.description.as_ref())
            .or_else(|| {
                spec.source
                    .translations
                    .get(lang)
                    .and_then(|t| t.description.as_ref())
            })
        {
            if !trans_desc.is_empty() {
                xml.push_str(&format!(
                    "        <Description xml:lang=\"{}\">{}</Description>\n",
                    lang, trans_desc
                ));
            }
        }
    }

    // Paket kopyası olarak Source tekrarı (Python Luppo uyumu)
    xml.push_str("        <Source>\n");
    xml.push_str(&format!("            <Name>{}</Name>\n", spec.source.name));
    if let Some(ref p) = spec.source.packager {
        xml.push_str("            <Packager>\n");
        xml.push_str(&format!("                <Name>{}</Name>\n", p.name));
        xml.push_str(&format!("                <Email>{}</Email>\n", p.email));
        xml.push_str("            </Packager>\n");
    }
    xml.push_str("        </Source>\n");

    // Lisans
    for lic in &spec.source.license {
        xml.push_str(&format!("        <License>{}</License>\n", lic));
    }

    // Provides (IsA ve COMAR)
    if let Some(provides) = &pkg.provides {
        let has_isa = !provides.isa.is_empty();
        let has_comar = !provides.comar.is_empty();
        if has_isa || has_comar {
            xml.push_str("        <Provides>\n");
            for isa in &provides.isa {
                xml.push_str(&format!("            <IsA>{}</IsA>\n", isa));
            }
            for comar in &provides.comar {
                if let Some(ref name_attr) = comar.name {
                    xml.push_str(&format!(
                        "            <COMAR script=\"{}\" name=\"{}\">{}</COMAR>\n",
                        comar.script, name_attr, comar.provide
                    ));
                } else {
                    xml.push_str(&format!(
                        "            <COMAR script=\"{}\">{}</COMAR>\n",
                        comar.script, comar.provide
                    ));
                }
            }
            xml.push_str("        </Provides>\n");
        }
    }

    xml.push_str(&format!(
        "        <Distribution>{}</Distribution>\n",
        distribution
    ));
    xml.push_str(&format!(
        "        <DistributionRelease>{}</DistributionRelease>\n",
        distribution_release
    ));
    xml.push_str(&format!("        <Architecture>{}</Architecture>\n", arch));
    xml.push_str(&format!(
        "        <InstalledSize>{}</InstalledSize>\n",
        installed_size
    ));
    xml.push_str(&format!(
        "        <PackageSize>{}</PackageSize>\n",
        package_size
    ));
    xml.push_str(&format!(
        "        <PackageHash>{}</PackageHash>\n",
        install_tar_hash
    ));
    xml.push_str(&format!(
        "        <InstallTarHash>{}</InstallTarHash>\n",
        install_tar_hash
    ));
    xml.push_str("        <PackageFormat>1.2</PackageFormat>\n");

    // History
    if let Some(ref h) = spec.history {
        xml.push_str("        <History>\n");
        for u in &h.updates {
            xml.push_str(&format!("            <Update release=\"{}\">\n", u.release));
            xml.push_str(&format!("                <Date>{}</Date>\n", u.date));
            xml.push_str(&format!(
                "                <Version>{}</Version>\n",
                u.version
            ));
            xml.push_str(&format!(
                "                <Comment>{}</Comment>\n",
                u.comment
            ));
            xml.push_str(&format!("                <Name>{}</Name>\n", u.committer));
            xml.push_str(&format!("                <Email>{}</Email>\n", u.email));
            if let Some(ref t) = u.type_ {
                xml.push_str(&format!("                <Type>{}</Type>\n", t));
            }
            if let Some(ref r) = u.requires {
                xml.push_str("                <Requires>\n");
                if r == "systemRestart" {
                    xml.push_str(&format!("                    <Action package=\"kernel\">{}</Action>\n", r));
                } else {
                    xml.push_str(&format!("                    <Action>{}</Action>\n", r));
                }
                xml.push_str("                </Requires>\n");
            }
            xml.push_str("            </Update>\n");
        }
        xml.push_str("        </History>\n");
    }

    xml.push_str("    </Package>\n");
    xml.push_str("</LUPPO>\n");
    Ok(xml)
}
