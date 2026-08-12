use crate::error::PisiResult;
use crate::package::{FileData, FilesXmlRoot, Package, PisiPackageData, PisiRoot};
use crate::PisiError;
use md5;
use rust_i18n::t;
use sha1::{Digest, Sha1};
use sha2::Sha256;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use tar::Archive as TarArchive;
use tar::{Builder as TarBuilder, Entry as TarEntry};
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;
use zip::ZipArchive;

rust_i18n::i18n!("../locales", fallback = "tr");

pub struct Packager;

impl Packager {
    /// Bir .pisi (ZIP) veya .deb (ar) paketini okur, metadata ve dosyaları ayrıştırır.
    pub fn read_package(path: &str) -> PisiResult<PisiPackageData> {
        if path.ends_with(".deb") {
            return Self::read_deb_package(path);
        }

        let mut file = std::fs::File::open(path).map_err(PisiError::IoError)?;
        let mut magic = [0u8; 8];
        if file.read_exact(&mut magic).is_ok() && &magic == b"!<arch>\n" {
            return Self::read_deb_package(path);
        }

        // Re-open/seek to 0 if it was checked
        let file = std::fs::File::open(path).map_err(PisiError::IoError)?;

        // ZIP Arşivini aç (Pisi paketleri PK.. imzalı birer ZIP'tir)
        let mut zip_archive = ZipArchive::new(file).map_err(|e| {
            PisiError::RuntimeError(t!("packager_error_zip", error = e).to_string())
        })?;

        let mut metadata: Option<Package> = None;
        let mut files_xml = None;
        let mut files = Vec::new();

        for i in 0..zip_archive.len() {
            let mut zip_file = zip_archive
                .by_index(i)
                .map_err(|e| PisiError::IoError(std::io::Error::other(e)))?;

            let name = zip_file.name().to_string();

            // 1. metadata.xml Okuma
            if name.ends_with("metadata.xml") {
                let mut content = String::new();
                zip_file
                    .read_to_string(&mut content)
                    .map_err(PisiError::IoError)?;

                // XML -> PisiRoot (PISI etiketi) -> Package
                let root: PisiRoot = quick_xml::de::from_str(&content).map_err(|e| {
                    PisiError::RuntimeError(
                        t!("packager_error_xml_parse", error = format!("{:?}", e)).to_string(),
                    )
                })?;

                metadata = Some(root.package);
            }
            // 2. files.xml Okuma
            else if name.ends_with("files.xml") {
                let mut content = String::new();
                zip_file
                    .read_to_string(&mut content)
                    .map_err(PisiError::IoError)?;

                match quick_xml::de::from_str::<FilesXmlRoot>(&content) {
                    Ok(root) => files_xml = Some(root.files),
                    Err(e) => println!("{}", t!("packager_warn_files_xml", error = e)),
                }
            }
            // 3. install.tar.xz Okuma (Gerçek sistem dosyaları)
            else if name.ends_with("install.tar.xz") {
                let decompressor = XzDecoder::new(zip_file);
                let mut tar_archive = TarArchive::new(decompressor);

                for tar_entry_result in tar_archive.entries().map_err(PisiError::IoError)? {
                    let mut tar_entry: TarEntry<_> =
                        tar_entry_result.map_err(PisiError::IoError)?;

                    let entry_type = tar_entry.header().entry_type();
                    if entry_type.is_file() || entry_type.is_symlink() {
                        let full_path = tar_entry
                            .path()
                            .map_err(PisiError::IoError)?
                            .to_str()
                            .unwrap_or("unknown")
                            .to_string();

                        // 'install/' ön ekini temizle (install/usr/bin/nano -> usr/bin/nano)
                        let clean_path = if full_path.starts_with("install/") {
                            full_path.replacen("install/", "", 1)
                        } else {
                            full_path
                        };

                        let header = tar_entry.header();
                        let mode = header.mode().unwrap_or(0o644);
                        let uid = header.uid().unwrap_or(0);
                        let gid = header.gid().unwrap_or(0);

                        let mut content = Vec::new();
                        let mut symlink_target = None;

                        if entry_type.is_symlink() {
                            if let Ok(Some(link_path)) = tar_entry.link_name() {
                                symlink_target = Some(link_path.to_string_lossy().to_string());
                            }
                        } else {
                            tar_entry
                                .read_to_end(&mut content)
                                .map_err(PisiError::IoError)?;
                        }

                        files.push(FileData {
                            path: clean_path,
                            size: content.len() as u64,
                            content,
                            mode,
                            uid,
                            gid,
                            symlink_target,
                        });
                    }
                }
            }
        }

        let metadata = metadata
            .ok_or_else(|| PisiError::RuntimeError(t!("packager_error_no_metadata").to_string()))?;

        Ok(PisiPackageData {
            metadata,
            files,
            files_xml,
        })
    }

    /// Belirtilen dizindeki dosyaları .pisi paketine dönüştürür.
    pub fn create_package(
        mut metadata: Package,
        source_dir: &std::path::Path,
        output_path: &std::path::Path,
    ) -> PisiResult<()> {
        let file = File::create(output_path).map_err(PisiError::IoError)?;
        let mut zip = zip::ZipWriter::new(file);

        // 1. install.tar.xz oluştur (Önce oluşturuyoruz ki hash hesaplayabilelim)
        let mut tar_xz_data = Vec::new();
        {
            let encoder = XzEncoder::new(&mut tar_xz_data, 9);
            let mut tar_builder = TarBuilder::new(encoder);

            // source_dir içindeki tüm dosyaları "install/" prefixi ile ekle
            if source_dir.exists() {
                tar_builder
                    .append_dir_all("install", source_dir)
                    .map_err(PisiError::IoError)?;
            }
            tar_builder.finish().map_err(PisiError::IoError)?;
        }

        // 2. Payload (install.tar.xz) için çeşitli özetleri hesapla
        let sha1_hash = format!("{:x}", Sha1::digest(&tar_xz_data));
        let sha256_hash = format!("{:x}", Sha256::digest(&tar_xz_data));
        let md5_hash = format!("{:x}", md5::compute(&tar_xz_data));

        // Pisi standardı gereği SHA1 ana hash olarak atanır
        metadata.package_hash = sha1_hash.clone();
        println!(
            "{}\n  SHA1:   {}\n  SHA256: {}\n  MD5:    {}",
            t!("packager_hashes_title"),
            sha1_hash,
            sha256_hash,
            md5_hash
        );

        // 3. metadata.xml oluştur (Artık gerçek hash'e sahip)
        let pisi_root = PisiRoot { source: None, package: metadata };
        let xml_content = quick_xml::se::to_string(&pisi_root).map_err(|e| {
            PisiError::RuntimeError(t!("packager_error_xml_gen", error = e).to_string())
        })?;

        zip.start_file("metadata.xml", zip::write::FileOptions::default())
            .map_err(|e| PisiError::RuntimeError(e.to_string()))?;
        zip.write_all(xml_content.as_bytes())
            .map_err(PisiError::IoError)?;

        // 4. install.tar.xz dosyasını ZIP'e ekle
        zip.start_file("install.tar.xz", zip::write::FileOptions::default())
            .map_err(|e| PisiError::RuntimeError(e.to_string()))?;
        zip.write_all(&tar_xz_data).map_err(PisiError::IoError)?;

        zip.finish()
            .map_err(|e| PisiError::RuntimeError(e.to_string()))?;

        // 5. Oluşturulan paketin (.pisi) genel özetlerini hesapla
        if let Ok(data) = std::fs::read(output_path) {
            println!(
                "{}",
                t!(
                    "packager_pkg_hashes_title",
                    name = output_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                )
            );
            println!("   SHA1:   {:x}", Sha1::digest(&data));
            println!("   SHA256: {:x}", Sha256::digest(&data));
            println!("   MD5:    {:x}", md5::compute(&data));
        }

        println!("{}", t!("packager_success", path = output_path.display()));
        Ok(())
    }

    /// Parses a Unix `ar` archive from raw bytes
    pub fn parse_ar(data: &[u8]) -> PisiResult<HashMap<String, Vec<u8>>> {
        if data.len() < 8 || &data[0..8] != b"!<arch>\n" {
            return Err(PisiError::RuntimeError(
                t!("packager_err_invalid_ar_sig").to_string(),
            ));
        }
        let mut files = HashMap::new();
        let mut offset = 8;
        while offset < data.len() {
            if offset + 60 > data.len() {
                break;
            }
            let header = &data[offset..offset + 60];
            let name_str = std::str::from_utf8(&header[0..16])
                .map_err(|e| {
                    PisiError::RuntimeError(t!("packager_err_ar_invalid_name", error = e).to_string())
                })?
                .trim();
            let name = name_str.trim_end_matches('/').to_string();

            let size_str = std::str::from_utf8(&header[48..58])
                .map_err(|e| {
                    PisiError::RuntimeError(t!("packager_err_ar_invalid_size", error = e).to_string())
                })?
                .trim();
            let size: usize = size_str.parse().map_err(|e| {
                PisiError::RuntimeError(t!(
                    "packager_err_ar_size_parse",
                    size = size_str,
                    error = e
                ).to_string())
            })?;

            offset += 60;
            if offset + size > data.len() {
                return Err(PisiError::RuntimeError(t!("packager_err_ar_truncated").to_string()));
            }
            let file_data = data[offset..offset + size].to_vec();
            files.insert(name, file_data);

            offset += size;
            if !size.is_multiple_of(2) {
                offset += 1;
            }
        }
        Ok(files)
    }

    /// Decompresses and extracts a tar archive from raw bytes, detecting compression type automatically
    #[allow(clippy::type_complexity)]
    pub fn decompress_and_unpack_tar(
        data: &[u8],
    ) -> PisiResult<Vec<(String, Vec<u8>, u32, u64, u64, Option<String>)>> {
        use std::io::Read;

        let decoded: Box<dyn Read> =
            if data.starts_with(&[0x1f, 0x8b]) {
                Box::new(flate2::read::GzDecoder::new(data))
            } else if data.starts_with(&[0xfd, 0x37, 0x7a, 0x58, 0x5a, 0x00]) {
                Box::new(xz2::read::XzDecoder::new(data))
            } else if data.starts_with(&[0x28, 0xb5, 0x2f, 0xfd]) {
                Box::new(zstd::stream::Decoder::new(data).map_err(|e| {
                    PisiError::RuntimeError(t!("packager_err_zstd_decode", error = e).to_string())
                })?)
            } else {
                Box::new(data)
            };

        let mut archive = tar::Archive::new(decoded);
        let mut files = Vec::new();
        for entry_result in archive
            .entries()
            .map_err(|e| PisiError::RuntimeError(t!("packager_err_tar_entries", error = e).to_string()))?
        {
            let mut entry = entry_result
                .map_err(|e| PisiError::RuntimeError(t!("packager_err_tar_entry", error = e).to_string()))?;
            let path = entry
                .path()
                .map_err(|e| PisiError::RuntimeError(t!("packager_err_tar_path", error = e).to_string()))?
                .to_string_lossy()
                .into_owned();

            let clean_path = if path.starts_with('.') {
                path.trim_start_matches('.').to_string()
            } else if !path.starts_with('/') {
                format!("/{}", path)
            } else {
                path
            };

            let _size = entry.size();
            let mode = entry.header().mode().unwrap_or(0o644);
            let uid = entry.header().uid().unwrap_or(0);
            let gid = entry.header().gid().unwrap_or(0);

            let symlink_target = if entry.header().entry_type().is_symlink() {
                entry
                    .link_name()
                    .map_err(|e| {
                        PisiError::RuntimeError(t!("packager_err_symlink_target", error = e).to_string())
                    })?
                    .map(|p| p.to_string_lossy().into_owned())
            } else {
                None
            };

            let mut content = Vec::new();
            if symlink_target.is_none() && entry.header().entry_type().is_file() {
                entry.read_to_end(&mut content).map_err(|e| {
                    PisiError::RuntimeError(t!("packager_err_tar_content", error = e).to_string())
                })?;
            }

            files.push((clean_path, content, mode, uid, gid, symlink_target));
        }
        Ok(files)
    }

    /// Parses a Debian `control` file format into key-value map
    pub fn parse_control_file(content: &str) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        let mut current_field = String::new();
        let mut current_value = String::new();

        for line in content.lines() {
            if line.starts_with(' ') || line.starts_with('\t') {
                current_value.push('\n');
                current_value.push_str(line.trim());
            } else if let Some(colon_pos) = line.find(':') {
                if !current_field.is_empty() {
                    fields.insert(current_field.clone(), current_value.trim().to_string());
                }
                current_field = line[0..colon_pos].trim().to_string();
                current_value = line[colon_pos + 1..].trim().to_string();
            }
        }
        if !current_field.is_empty() {
            fields.insert(current_field, current_value.trim().to_string());
        }
        fields
    }

    /// Reads a .deb package and constructs a unified PisiPackageData structure dynamically
    pub fn read_deb_package(path: &str) -> PisiResult<PisiPackageData> {
        let file_data = std::fs::read(path).map_err(PisiError::IoError)?;
        let ar_files = Self::parse_ar(&file_data)?;

        // control.tar arşivini bul ve aç
        let control_tar_key = ar_files
            .keys()
            .find(|k| k.starts_with("control.tar"))
            .ok_or_else(|| {
                PisiError::RuntimeError("deb paketi içinde control.tar bulunamadı".to_string())
            })?;
        let control_tar_data = ar_files.get(control_tar_key).unwrap();
        let control_files = Self::decompress_and_unpack_tar(control_tar_data)?;

        let mut control_content = String::new();
        let mut preinst = None;
        let mut postinst = None;
        let mut postrm = None;

        for (path, content, _, _, _, _) in &control_files {
            if path.ends_with("/control") {
                control_content = String::from_utf8_lossy(content).into_owned();
            } else if path.ends_with("/preinst") {
                preinst = Some(String::from_utf8_lossy(content).into_owned());
            } else if path.ends_with("/postinst") {
                postinst = Some(String::from_utf8_lossy(content).into_owned());
            } else if path.ends_with("/postrm") {
                postrm = Some(String::from_utf8_lossy(content).into_owned());
            }
        }

        if control_content.is_empty() {
            return Err(PisiError::RuntimeError(
                "deb control arşivi içinde control dosyası bulunamadı".to_string(),
            ));
        }

        let control_fields = Self::parse_control_file(&control_content);
        let name = control_fields.get("Package").cloned().ok_or_else(|| {
            PisiError::RuntimeError("control dosyasında Package adı bulunamadı".to_string())
        })?;
        let version = control_fields
            .get("Version")
            .cloned()
            .unwrap_or_else(|| "1.0.0".to_string());
        let architecture = control_fields
            .get("Architecture")
            .cloned()
            .unwrap_or_else(|| "x86_64".to_string());
        let section = control_fields
            .get("Section")
            .cloned()
            .unwrap_or_else(|| "system.base".to_string());
        let homepage = control_fields.get("Homepage").cloned();

        let maintainer_raw = control_fields
            .get("Maintainer")
            .cloned()
            .unwrap_or_else(|| "Debian Maintainer <maintainer@debian.org>".to_string());

        let (m_name, m_email) = if let Some(bracket_start) = maintainer_raw.find('<') {
            let m_name = maintainer_raw[0..bracket_start].trim().to_string();
            let m_email = if let Some(bracket_end) = maintainer_raw.find('>') {
                maintainer_raw[bracket_start + 1..bracket_end]
                    .trim()
                    .to_string()
            } else {
                maintainer_raw[bracket_start + 1..].trim().to_string()
            };
            (m_name, Some(m_email))
        } else {
            (maintainer_raw, None)
        };

        let raw_desc = control_fields
            .get("Description")
            .cloned()
            .unwrap_or_default();
        let mut desc_lines = raw_desc.lines();
        let summary = desc_lines.next().unwrap_or("").trim().to_string();
        let description = desc_lines.collect::<Vec<_>>().join("\n").trim().to_string();

        let depends_str = control_fields.get("Depends").cloned().unwrap_or_default();
        let mut dependencies = Vec::new();
        for dep in depends_str.split(',') {
            let dep_name = dep.split('(').next().unwrap_or("").trim();
            if !dep_name.is_empty() {
                dependencies.push(dep_name.to_string());
            }
        }

        // data.tar arşivini bul ve aç
        let data_tar_key = ar_files
            .keys()
            .find(|k| k.starts_with("data.tar"))
            .ok_or_else(|| {
                PisiError::RuntimeError("deb paketi içinde data.tar bulunamadı".to_string())
            })?;
        let data_tar_data = ar_files.get(data_tar_key).unwrap();
        let data_files = Self::decompress_and_unpack_tar(data_tar_data)?;

        let mut files = Vec::new();
        let mut file_xml_entries = Vec::new();

        for (path, content, mode, uid, gid, symlink_target) in data_files {
            if symlink_target.is_none() && content.is_empty() && path.ends_with('/') {
                continue;
            }

            let size = content.len() as u64;

            let hash = if symlink_target.is_none() {
                let mut hasher = Sha1::new();
                hasher.update(&content);
                Some(format!("{:x}", hasher.finalize()))
            } else {
                None
            };

            files.push(FileData {
                content,
                path: path.clone(),
                size,
                mode,
                uid,
                gid,
                symlink_target: symlink_target.clone(),
            });

            file_xml_entries.push(crate::package::FileXmlEntry {
                path,
                file_type: if symlink_target.is_some() {
                    "symlink".to_string()
                } else {
                    "regular".to_string()
                },
                size,
                uid,
                gid,
                mode: format!("{:o}", mode),
                hash,
            });
        }

        let update = crate::package::Update {
            release: 1,
            version: version.clone(),
            date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            comment: "Debian package converted to PiSi".to_string(),
            name: m_name.clone(),
            email: m_email.clone(),
            type_: None,
            requires: None,
        };

        let metadata = Package {
            name: name.clone(),
            release: 1,
            archive: None,
            summaries: vec![crate::package::LocalizedText {
                lang: Some("en".to_string()),
                text: summary,
            }],
            descriptions: vec![crate::package::LocalizedText {
                lang: Some("en".to_string()),
                text: description,
            }],
            history: crate::package::History {
                updates: vec![update],
            },
            architecture,
            runtime_dependencies: Some(crate::package::RuntimeDependencies { dependencies }),
            conflicts: None,
            build_dependencies: None,
            provides: Vec::new(),
            provides_block: None,
            licenses: vec!["Proprietary".to_string()],
            partof: section,
            package_uri: format!("{}_deb.pisi", name),
            repo_url: String::new(),
            package_hash: String::new(),
            installed_size: files.iter().map(|f| f.size).sum(),
            package_size: file_data.len() as u64,
            distribution_release: "2.0".to_string(),
            repo_name: String::new(),
            pre_install: preinst,
            post_install: postinst,
            pre_upgrade: None,
            post_upgrade: None,
            post_remove: postrm,
            pre_remove: None,
            users: None,
            groups: None,
            mirrors: Vec::new(),
            homepage,
            icon: None,
            screenshot: None,
            packager: Some(crate::package::Packager {
                name: m_name,
                email: m_email.unwrap_or_else(|| "info@pisilinux.org".to_string()),
            }),
            source: None,
            install_tar_hash: None,
            package_format: Some("1.2".to_string()),
            build_host: Some("local".to_string()),
            distribution: Some("PisiLinux".to_string()),
            version,
        };

        Ok(PisiPackageData {
            metadata,
            files,
            files_xml: Some(file_xml_entries),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_deb_package() {
        let deb_path = "/home/pisicik/uhapsigner_2.0.5_amd64.deb";
        if std::path::Path::new(deb_path).exists() {
            let pkg_data =
                Packager::read_package(deb_path).expect("deb package should be read successfully");
            assert_eq!(pkg_data.metadata.name, "uhapsigner");
            assert_eq!(pkg_data.metadata.version, "2.0.5");
            assert!(
                !pkg_data.files.is_empty(),
                "Parsed files list should not be empty"
            );
            println!(
                "Successfully parsed deb file: {} with {} files",
                pkg_data.metadata.name,
                pkg_data.files.len()
            );
        }
    }
}
