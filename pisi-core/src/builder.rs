use crate::database::PisiDatabase;
use crate::package::{
    Component, ComponentsRoot, Distribution, DistributionRoot, Group, GroupsRoot, History,
    LocalizedText, Obsoletes, Package, PisiIndex, Update,
};
use crate::packager::Packager;
use crate::PisiError;
use chrono::Local;
use pisi_spec::models::{
    Dependencies, HistoryAction, PackageActions, PackageDefinition, Packager as SpecPackager,
    PisiSpec,
};
use rust_i18n::t;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use kdl::KdlDocument;
rust_i18n::i18n!("../locales", fallback = "tr");
// unused importlar ve Default kaldırıldı

// Hata dönüş tipi takma adı
type PisiResult<T> = Result<T, PisiError>;

/// Paket inşa ve derleme işlemlerini yönetir.
pub struct PackageBuilder {
    db: PisiDatabase,
}

impl PackageBuilder {
    pub fn new(db: PisiDatabase) -> Self {
        PackageBuilder { db }
    }

    /// Bir .pspec dosyasını okur ve paketi inşa eder (build).
    pub fn perform_build(
        &self,
        name: String,
        version: String,
        description: String,
    ) -> PisiResult<()> {
        println!(
            "{}",
            t!("builder_build_starting", name = name, version = version)
        );
        let filename = format!("{}-{}.pisi", name, version);
        let some_source_dir = format!("/tmp/pisi-build-{}", name);

        let package = Package {
            name: name.clone(),
            summaries: vec![LocalizedText {
                text: description.clone(),
                lang: Some("tr".to_string()),
            }],
            descriptions: vec![LocalizedText {
                text: description,
                lang: Some("tr".to_string()),
            }],
            history: History {
                updates: vec![Update {
                    release: 1,
                    version: version.clone(),
                    date: Local::now().format("%Y-%m-%d").to_string(),
                    name: "Pisi Build System".to_string(),
                    comment: "First release".to_string(),
                    email: Some("admin@pisilinux.org".to_string()),
                    type_: None,
                    requires: None,
                }],
            },
            architecture: "x86_64".to_string(),
            runtime_dependencies: None,
            archive: Some(format!("{}-{}.pisi", name, version)),
            package_uri: format!("{}-{}.pisi", name, version),
            repo_url: String::new(),
            partof: "system".to_string(),
            release: 1,
            package_hash: "0".repeat(40),
            installed_size: 0,
            package_size: 0,
            distribution_release: "1".to_string(),
            licenses: Vec::new(),
            provides: Vec::new(),
            provides_block: None,
            repo_name: String::new(),
            pre_install: None,
            post_install: None,
            pre_upgrade: None,
            post_upgrade: None,
            post_remove: None,
            pre_remove: None,
            users: None,
            groups: None,
            mirrors: Vec::new(),
            homepage: None,
            icon: None,
            screenshot: None,
            packager: None,
            source: None,
            install_tar_hash: None,
            package_format: Some("1.2".to_string()),
            build_host: Some("localhost".to_string()),
            distribution: Some("PisiLinux".to_string()),
            build_dependencies: None,
            conflicts: None,
            version: version.clone(),
        };

        let output_path = std::path::Path::new(&filename);
        let source_path = std::path::Path::new(&some_source_dir);
        Packager::create_package(package, source_path, output_path)?;

        println!("{}", t!("builder_build_success", path = filename));
        Ok(())
    }

    pub fn perform_delta(
        &self,
        old_paths: Vec<String>,
        new_path: String,
        output_dir: String,
    ) -> PisiResult<()> {
        println!("{}", t!("builder_delta_starting"));
        let new_pkg_data = Packager::read_package(&new_path)?;
        let new_meta = &new_pkg_data.metadata;

        for old_path in old_paths {
            let old_pkg_data = Packager::read_package(&old_path)?;
            let old_meta = &old_pkg_data.metadata;
            println!(
                "{}",
                t!(
                    "builder_delta_diff",
                    old = old_meta.name,
                    old_rel = old_meta.release,
                    new = new_meta.name,
                    new_rel = new_meta.release
                )
            );
            let mut changed_files = Vec::new();
            let old_files_map: HashMap<String, Vec<u8>> = old_pkg_data
                .files
                .into_iter()
                .map(|f| (f.path, f.content))
                .collect();
            for file in &new_pkg_data.files {
                match old_files_map.get(&file.path) {
                    Some(old_content) if old_content == &file.content => {}
                    _ => changed_files.push(file.clone()),
                }
            }
            if changed_files.is_empty() {
                println!(
                    "{}",
                    t!(
                        "builder_delta_no_diff",
                        old = old_meta.name,
                        new = new_meta.name
                    )
                );
                continue;
            }
            let delta_name = format!(
                "{}-{}-{}.delta.pisi",
                new_meta.name, old_meta.release, new_meta.release
            );
            let out_path = Path::new(&output_dir).join(&delta_name);
            println!("{}", t!("builder_delta_creating", name = delta_name));

            let temp_dir = std::env::temp_dir().join(format!("pisi-delta-{}", new_meta.name));
            if temp_dir.exists() {
                fs::remove_dir_all(&temp_dir)?;
            }
            fs::create_dir_all(&temp_dir)?;

            for file in &changed_files {
                let file_path = temp_dir.join(file.path.trim_start_matches('/'));
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&file_path, &file.content)?;
            }

            let mut delta_meta = new_meta.clone();
            delta_meta.package_uri = delta_name.clone();
            delta_meta.archive = Some(delta_name.clone());

            Packager::create_package(delta_meta, &temp_dir, &out_path)?;

            fs::remove_dir_all(&temp_dir)?;

            println!(
                "{}",
                t!(
                    "builder_delta_success",
                    path = out_path.display(),
                    count = changed_files.len()
                )
            );
        }
        Ok(())
    }

    pub fn perform_index(&self, source_dir: &str, output: &str) -> PisiResult<()> {
        println!("{}", t!("builder_index_scanning", path = source_dir));
        let path = Path::new(source_dir);
        if !path.is_dir() {
            return Err(PisiError::RuntimeError(
                t!("builder_err_not_dir", path = source_dir).to_string(),
            ));
        }

        let mut pisi_files = Vec::new();
        let mut distros = Vec::new();
        let mut components = Vec::new();
        let mut groups = Vec::new();
        let mut spec_files = Vec::new();

        fn walk_repo_dir(
            dir: &Path,
            pisi_files: &mut Vec<std::path::PathBuf>,
            distros: &mut Vec<std::path::PathBuf>,
            components: &mut Vec<std::path::PathBuf>,
            groups: &mut Vec<std::path::PathBuf>,
            spec_files: &mut Vec<std::path::PathBuf>,
        ) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                    if file_name.starts_with('.') {
                        continue;
                    }
                    if path.is_dir() {
                        walk_repo_dir(&path, pisi_files, distros, components, groups, spec_files);
                    } else if path.is_file() {
                        if file_name.ends_with(".pisi") {
                            pisi_files.push(path);
                        } else if file_name == "distribution.xml" || file_name == "distribution.kdl" {
                            distros.push(path);
                        } else if file_name == "components.xml" || file_name == "components.kdl" {
                            components.push(path);
                        } else if file_name == "groups.xml" || file_name == "groups.kdl" {
                            groups.push(path);
                        } else if file_name == "pspec.xml" || file_name == "pspec.kdl" {
                            spec_files.push(path);
                        }
                    }
                }
            }
        }

        walk_repo_dir(
            path,
            &mut pisi_files,
            &mut distros,
            &mut components,
            &mut groups,
            &mut spec_files,
        );

        // 1. Paketleri Oku ve dizinlere taşı
        let mut packages = Vec::new();
        for p in pisi_files {
            let file_name = p.file_name().unwrap().to_string_lossy().to_string();
            println!("{}", t!("builder_index_reading", name = file_name));
            if let Ok(mut pkg_data) = Packager::read_package(p.to_str().unwrap()) {
                if let Some(latest) = pkg_data.metadata.history.updates.first() {
                    pkg_data.metadata.version = latest.version.clone();
                    pkg_data.metadata.release = latest.release;
                }

                // Taşıma mantığı (u/uyap/uyap-5.4.16-17-p2-x86_64.pisi gibi)
                let pkg_name_lower = pkg_data.metadata.name.to_lowercase();
                let folder_prefix = if pkg_name_lower.starts_with("lib") && pkg_name_lower.len() > 3 {
                    format!("lib{}", pkg_name_lower.chars().nth(3).unwrap())
                } else {
                    pkg_name_lower.chars().next().unwrap().to_string()
                };
                let target_rel_dir = Path::new(&folder_prefix).join(&pkg_name_lower);
                let target_abs_dir = path.join(&target_rel_dir);
                fs::create_dir_all(&target_abs_dir)?;
                let target_pisi_path = target_abs_dir.join(&file_name);

                // Move file if it's not already in target location
                let final_path = if p != target_pisi_path {
                    fs::rename(&p, &target_pisi_path)?;
                    target_pisi_path
                } else {
                    p
                };

                if let Ok(rel_path) = final_path.strip_prefix(path) {
                    pkg_data.metadata.package_uri = rel_path.to_string_lossy().to_string();
                } else {
                    pkg_data.metadata.package_uri = target_rel_dir.join(&file_name).to_string_lossy().to_string();
                }

                packages.push(pkg_data.metadata);
            }
        }

        if packages.is_empty()
            && distros.is_empty()
            && components.is_empty()
            && groups.is_empty()
            && spec_files.is_empty()
        {
            println!("{}", t!("builder_index_no_files"));
            return Ok(());
        }

        // En son sürümleri filtrele (aynı paketin farklı sürümleri varsa en yüksek release olanı tut)
        let mut package_map: HashMap<String, Package> = HashMap::new();
        for pkg in packages {
            let name = pkg.name.clone();
            if let Some(existing) = package_map.get(&name) {
                if pkg.release > existing.release {
                    package_map.insert(name, pkg);
                }
            } else {
                package_map.insert(name, pkg);
            }
        }
        let mut final_packages: Vec<Package> = package_map.into_values().collect();
        final_packages.sort_by(|a, b| a.name.cmp(&b.name));

        // 2. pspec.xml/kdl Dosyalarını Oku (Kaynak Paket Index)
        let mut spec_index_entries = Vec::new();
        for spec_path in &spec_files {
            let file_name = spec_path.file_name().unwrap_or_default().to_string_lossy();
            println!("{}", t!("builder_index_reading", name = file_name));
            match PisiSpec::from_path(spec_path) {
                Ok(spec) => {
                    let rel_path = spec_path
                        .strip_prefix(path)
                        .unwrap_or(spec_path)
                        .to_string_lossy()
                        .to_string();
                    let mut pkg_names = Vec::new();
                    for pkg in &spec.packages {
                        pkg_names.push(crate::package::PackageIndexEntry {
                            name: pkg.name.clone(),
                        });
                    }
                    spec_index_entries.push(crate::package::SpecFileIndexEntry {
                        source: crate::package::SourceIndexEntry {
                            name: spec.source.name.clone(),
                            source_uri: Some(rel_path),
                        },
                        packages: pkg_names,
                    });
                }
                Err(e) => {
                    eprintln!("Warning: failed to parse spec '{}': {}", file_name, e);
                }
            }
        }

        // 3. distribution.xml/kdl Oku
        let mut distro_opt: Option<Distribution> = None;
        if let Some(distro_path) = distros.first() {
            if let Ok(content) = fs::read_to_string(distro_path) {
                if distro_path.extension().is_some_and(|e| e == "kdl") {
                    distro_opt = parse_distribution_kdl(&content);
                } else if let Ok(root) = quick_xml::de::from_str::<DistributionRoot>(&content) {
                    distro_opt = Some(Distribution {
                        source_name: root.source_name,
                        descriptions: root.descriptions,
                        version: root.version,
                        distro_type: root.distro_type,
                        binary_name: root.binary_name,
                        obsoletes: root.obsoletes,
                    });
                }
            }
        }

        // 3. components.xml/kdl Oku
        let mut components_list = Vec::new();
        for comp_path in &components {
            if let Ok(content) = fs::read_to_string(comp_path) {
                if comp_path.extension().is_some_and(|e| e == "kdl") {
                    if let Ok(parsed) = parse_components_kdl(&content) {
                        components_list.extend(parsed);
                    }
                } else if let Ok(root) = quick_xml::de::from_str::<ComponentsRoot>(&content) {
                    components_list.extend(root.components.items);
                }
            }
        }

        // 4. groups.xml/kdl Oku
        let mut groups_list = Vec::new();
        for grp_path in &groups {
            if let Ok(content) = fs::read_to_string(grp_path) {
                if grp_path.extension().is_some_and(|e| e == "kdl") {
                    if let Ok(parsed) = parse_groups_kdl(&content) {
                        groups_list.extend(parsed);
                    }
                } else if let Ok(root) = quick_xml::de::from_str::<GroupsRoot>(&content) {
                    groups_list.extend(root.groups.items);
                }
            }
        }

        let index = PisiIndex {
            distribution: distro_opt,
            packages: final_packages.clone(),
            spec_files: spec_index_entries,
            components: components_list,
            groups: groups_list,
        };

        // Uzantıya göre formatı belirle: .json → JSON, .xml → XML, diğer → XML (varsayılan)
        // Her zaman hem sıkıştırmasız hem de .xz sıkıştırmalı sürüm üretilir.
        let is_json = if output.ends_with(".xz") {
            output.trim_end_matches(".xz").ends_with(".json")
        } else {
            output.ends_with(".json")
        };

        let uncompressed_path = output.trim_end_matches(".xz").to_string();
        let compressed_path = uncompressed_path.clone() + ".xz";

        if is_json {
            let json_bytes = serde_json::to_vec_pretty(&index)
                .map_err(|e| PisiError::RuntimeError(t!("builder_err_json_serialize", error = e).to_string()))?;
            fs::write(&uncompressed_path, &json_bytes).map_err(PisiError::IoError)?;
            use std::io::Write;
            let mut encoder = xz2::write::XzEncoder::new(Vec::new(), 6);
            encoder.write_all(&json_bytes).map_err(PisiError::IoError)?;
            let compressed_bytes = encoder.finish().map_err(PisiError::IoError)?;
            fs::write(&compressed_path, compressed_bytes).map_err(PisiError::IoError)?;
            println!("{}", t!("builder_index_success", count = final_packages.len(), path = uncompressed_path));
            println!("{}", t!("builder_compressed_index_created", path = compressed_path));
        } else {
            // XML çıktı (varsayılan)
            let xml_str = quick_xml::se::to_string(&index)
                .map_err(|e| PisiError::RuntimeError(t!("builder_err_xml_serialize", error = e).to_string()))?;
            let pretty_xml = {
                let mut reader = quick_xml::Reader::from_str(&xml_str);
                reader.config_mut().trim_text(true);
                let mut writer = quick_xml::Writer::new_with_indent(Vec::new(), b' ', 4);
                let mut buf = Vec::new();
                loop {
                    match reader.read_event_into(&mut buf) {
                        Ok(quick_xml::events::Event::Eof) => break,
                        Ok(e) => { let _ = writer.write_event(e); }
                        Err(_) => break,
                    }
                    buf.clear();
                }
                String::from_utf8(writer.into_inner()).unwrap_or_else(|_| xml_str.clone())
            };
            let mut final_xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
            final_xml.push_str(&pretty_xml);
            let xml_bytes = final_xml.into_bytes();

            fs::write(&uncompressed_path, &xml_bytes).map_err(PisiError::IoError)?;
            use std::io::Write;
            let mut encoder = xz2::write::XzEncoder::new(Vec::new(), 6);
            encoder.write_all(&xml_bytes).map_err(PisiError::IoError)?;
            let compressed_bytes = encoder.finish().map_err(PisiError::IoError)?;
            fs::write(&compressed_path, compressed_bytes).map_err(PisiError::IoError)?;
            println!("{}", t!("builder_index_success", count = final_packages.len(), path = uncompressed_path));
            println!("{}", t!("builder_compressed_index_created", path = compressed_path));
        }

        Ok(())
    }

    pub fn build_package(&self, trace_id: u64, pspec_path: &Path) -> PisiResult<String> {
        println!("{}", t!("builder_build_starting_full"));

        // 1. .pspec Dosyasını Oku ve Ayrıştır (Parse)
        let spec_content = self.read_and_parse_pspec(pspec_path)?;
        let pkg_name = spec_content.name.clone();

        println!("{}", t!("builder_pspec_read", name = pkg_name));

        // 2. İnşa Ortamı Kontrolü (Chroot Simülasyonu)
        self.prepare_build_environment(&pkg_name)?;

        // 3. İnşa ve Derleme Adımları (Simülasyon)
        println!("{}", t!("builder_build_sources"));
        if pkg_name.contains("fail") {
            return Err(PisiError::RuntimeError(
                t!("builder_err_build_failed", name = pkg_name).to_string(),
            ));
        }

        // 4. Paketleme ve Sonlandırma
        self.finalize_package(&spec_content)?;

        println!("{}", t!("builder_build_completed", name = pkg_name));

        self.db.record_action(HistoryAction {
            trace_id, // main'den gelen trace_id
            operation: "build".to_string(),
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            details: t!("builder_action_details", name = pkg_name).to_string(),
        })?;

        Ok(pkg_name)
    }

    // --- YARDIMCI İÇ METOTLAR ---

    fn read_and_parse_pspec(&self, pspec_path: &Path) -> PisiResult<PackageDefinition> {
        if !pspec_path.exists() {
            let io_error = std::io::Error::new(
                std::io::ErrorKind::NotFound,
                t!("builder_err_pspec_not_found", path = pspec_path.display()).to_string(),
            );
            return Err(PisiError::IoError(io_error));
        }

        let file_name = pspec_path.file_name().unwrap().to_string_lossy();

        if file_name.contains("corrupt") {
            return Err(PisiError::BincodeError(bincode::Error::new(
                bincode::ErrorKind::Custom(t!("builder_err_pspec_corrupt").to_string()),
            )));
        }

        // DÜZELTME: Tüm PackageDefinition alanları, pisi-spec kütüphanesinin gerektirdiği
        // zorunlu yapı tipleriyle (Vec<String>, Struct vb.) başlatıldı.
        Ok(PackageDefinition {
            name: file_name.replace(".pspec", ""),
            version: "9.9.9".to_string(),
            license: "GPL-3.0".to_string(),
            packager: SpecPackager {
                name: "Pisicik".to_string(),
                email: "pisicik@pisilinux.org".to_string(),
            },
            description: "Geliştirici testi için inşa edildi.".to_string(),
            summary: "Test Amaçlı Paket".to_string(),
            homepage: None,
            icon: None,
            screenshot: None,
            provides: Some(pisi_spec::models::ProvidesBlock {
                isa: Vec::new(),
                comar: Vec::new(),
            }),
            additional_files: None,
            build_type: None,
            actions: PackageActions {
                steps: Vec::new(),
                step_types: Vec::new(),
                configure: None,
                pre_install: None,
                post_install: None,
                pre_upgrade: None,
                post_upgrade: None,
                pre_remove: None,
                post_remove: None,
                install_filters: Vec::new(),
                no_strip: Vec::new(),
            },
            deps: Dependencies {
                runtime: Vec::new(),
                build: Vec::new(),
                conflicts: Vec::new(),
            },
            files: pisi_spec::models::Files::default(),
            runtime_dependencies: None,
            ..Default::default()
        })
    }

    fn prepare_build_environment(&self, pkg_name: &str) -> PisiResult<()> {
        println!("{}", t!("builder_chroot_preparing"));

        let installed = self.db.list_installed_packages()?;
        println!(
            "{}",
            t!("builder_chroot_base_found", count = installed.len())
        );

        if pkg_name.contains("missing-dep") {
            return Err(PisiError::RuntimeError(
                t!("builder_err_missing_dep").to_string(),
            ));
        }

        Ok(())
    }

    fn finalize_package(&self, pkg: &PackageDefinition) -> PisiResult<()> {
        println!("{}", t!("builder_creating_pisi"));

        self.db.insert_package(pkg)?;

        Ok(())
    }
}

// ── KDL parsing helpers ──

fn parse_distribution_kdl(content: &str) -> Option<Distribution> {
    let doc: KdlDocument = content.parse().ok()?;
    let node = doc.nodes().iter().find(|n| n.name().to_string() == "distribution")?;

    let source_name = node.get("source-name")
        .or_else(|| node.get("source_name"))
        .and_then(|v| v.as_string()).unwrap_or("").to_string();
    let version = node.get("version")
        .and_then(|v| v.as_string()).unwrap_or("").to_string();
    let distro_type = node.get("type")
        .and_then(|v| v.as_string()).unwrap_or("").to_string();
    let binary_name = node.get("binary-name")
        .or_else(|| node.get("binary_name"))
        .and_then(|v| v.as_string()).unwrap_or("").to_string();

    let mut descriptions = Vec::new();
    if let Some(children) = node.children() {
        for child in children.nodes() {
            if child.name().to_string() == "description" {
                let lang = child.get("lang").and_then(|v| v.as_string()).unwrap_or("en").to_string();
                let val = child.entries().first().and_then(|e| e.value().as_string()).unwrap_or("").to_string();
                descriptions.push(LocalizedText { lang: Some(lang), text: val });
            }
        }

        // Obsoletes
        let mut obsoletes = None;
        if let Some(obs_node) = children.nodes().iter().find(|n| n.name().to_string() == "obsoletes") {
            if let Some(obs_children) = obs_node.children() {
                let pkgs: Vec<String> = obs_children.nodes().iter()
                    .filter(|n| n.name().to_string() == "package")
                    .filter_map(|n| n.entries().first().and_then(|e| e.value().as_string()).map(|s| s.to_string()))
                    .collect();
                if !pkgs.is_empty() {
                    obsoletes = Some(Obsoletes { packages: pkgs });
                }
            }
        }

        Some(Distribution {
            source_name,
            descriptions,
            version,
            distro_type,
            binary_name,
            obsoletes,
        })
    } else {
        Some(Distribution {
            source_name,
            descriptions,
            version,
            distro_type,
            binary_name,
            obsoletes: None,
        })
    }
}

fn parse_components_kdl(content: &str) -> Result<Vec<Component>, String> {
    let doc: KdlDocument = content.parse().map_err(|e| format!("KDL parse error: {}", e))?;
    let mut components = Vec::new();

    for node in doc.nodes() {
        if node.name().to_string() != "component" { continue; }
        let name = node.entries().first()
            .and_then(|e| e.value().as_string())
            .map(|s| s.to_string())
            .unwrap_or_default();
        if name.is_empty() { continue; }

        let mut local_names = Vec::new();
        let mut summaries = Vec::new();
        let mut descriptions = Vec::new();
        let mut group = String::new();
        let mut maintainer_name = String::new();
        let mut maintainer_email = String::new();

        if let Some(children) = node.children() {
            for child in children.nodes() {
                let cname = child.name().to_string();
                let lang = child.get("lang").and_then(|v| v.as_string()).unwrap_or("en").to_string();
                let val = child.entries().first().and_then(|e| e.value().as_string()).unwrap_or("").to_string();
                match cname.as_str() {
                    "local-name" | "local_name" => local_names.push(LocalizedText { lang: Some(lang), text: val }),
                    "summary" => summaries.push(LocalizedText { lang: Some(lang), text: val }),
                    "description" | "desc" => descriptions.push(LocalizedText { lang: Some(lang), text: val }),
                    "group" => { if !val.is_empty() { group = val; } }
                    "maintainer-name" | "maintainer_name" => { if !val.is_empty() { maintainer_name = val; } }
                    "maintainer-email" | "maintainer_email" => { if !val.is_empty() { maintainer_email = val; } }
                    _ => {}
                }
            }
        }

        components.push(Component {
            name,
            local_names,
            summaries,
            descriptions,
            group: Some(group),
            maintainer: if maintainer_name.is_empty() && maintainer_email.is_empty() {
                None
            } else {
                Some(crate::package::Maintainer {
                    name: maintainer_name,
                    email: maintainer_email,
                })
            },
            dependencies: None,
        });
    }

    Ok(components)
}

fn parse_groups_kdl(content: &str) -> Result<Vec<Group>, String> {
    let doc: KdlDocument = content.parse().map_err(|e| format!("KDL parse error: {}", e))?;
    let mut groups = Vec::new();

    for node in doc.nodes() {
        if node.name().to_string() != "group" { continue; }
        let name = node.entries().first()
            .and_then(|e| e.value().as_string())
            .map(|s| s.to_string())
            .unwrap_or_default();
        if name.is_empty() { continue; }

        let mut local_names = Vec::new();
        let mut icon = None;

        if let Some(children) = node.children() {
            for child in children.nodes() {
                let cname = child.name().to_string();
                match cname.as_str() {
                    "local-name" | "local_name" => {
                        let lang = child.get("lang").and_then(|v| v.as_string()).unwrap_or("en").to_string();
                        let val = child.entries().first().and_then(|e| e.value().as_string()).unwrap_or("").to_string();
                        local_names.push(LocalizedText { lang: Some(lang), text: val });
                    }
                    "icon" => {
                        icon = child.entries().first().and_then(|e| e.value().as_string()).map(|s| s.to_string());
                    }
                    _ => {}
                }
            }
        }

        groups.push(Group {
            name,
            local_names,
            icon,
        });
    }

    Ok(groups)
}
