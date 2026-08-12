use crate::database::PisiDatabase;
use crate::package::{FileMetadata, InstalledPackage};
use crate::PisiError;
use chrono::Local;
use rust_i18n::t;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

pub struct DebManager {
    db: PisiDatabase,
}

impl DebManager {
    pub fn new(db: PisiDatabase) -> Self {
        DebManager { db }
    }

    /// Bir .deb paketini sisteme kurar.
    pub fn install_deb<P: AsRef<Path>>(&self, path: P, dest_root: &Path) -> Result<(), PisiError> {
        let path = path.as_ref();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        println!("{}", t!("deb_processing", name = file_name));

        // 1. Geçici bir çalışma dizini oluştur
        let temp_dir = tempfile::tempdir().map_err(crate::PisiError::IoError)?;
        let temp_path = temp_dir.path();

        // 2. .deb paketini (ar arşivi) aç
        // Not: 'ar' komutu binutils paketinden gelir.
        let status = Command::new("ar")
            .arg("x")
            .arg(path.canonicalize().map_err(crate::PisiError::IoError)?)
            .current_dir(temp_path)
            .status()
            .map_err(|e| crate::PisiError::RuntimeError(t!("deb_err_ar", error = e).to_string()))?;

        if !status.success() {
            return Err(crate::PisiError::RuntimeError(
                t!("deb_err_ar_fail").to_string(),
            ));
        }

        // 3. Veri arşivini bul (data.tar.gz, data.tar.xz veya data.tar.zst)
        let data_archive = ["data.tar.xz", "data.tar.gz", "data.tar.zst", "data.tar.bz2"]
            .iter()
            .find(|&&f| temp_path.join(f).exists())
            .ok_or_else(|| crate::PisiError::RuntimeError(t!("deb_err_no_data").to_string()))?;

        // 4. Dosyaları hedef dizine çıkart
        println!("{}", t!("deb_extracting"));
        let status = Command::new("tar")
            .arg("-xf")
            .arg(temp_path.join(data_archive))
            .arg("-C")
            .arg(dest_root)
            .status()
            .map_err(|e| {
                crate::PisiError::RuntimeError(t!("deb_err_tar", error = e).to_string())
            })?;

        if !status.success() {
            return Err(crate::PisiError::RuntimeError(
                t!("deb_err_tar_fail").to_string(),
            ));
        }

        // 5. Paket ismini ve versiyonunu tespit et (Dosya adından basitçe veya control dosyasından)
        // Şimdilik dosya adından basit bir tahmin yapalım, ileride control dosyasını okuyabiliriz.
        let pkg_name = file_name
            .split('_')
            .next()
            .unwrap_or(&file_name)
            .to_string();

        // 6. Veritabanına kaydet (Kaldırılabilmesi için)
        let mut installed_files = HashMap::new();

        // Tar içeriğini listele ve veritabanına dosyaları kaydet
        let output = Command::new("tar")
            .arg("-tf")
            .arg(temp_path.join(data_archive))
            .output()
            .map_err(crate::PisiError::IoError)?;

        let file_list = String::from_utf8_lossy(&output.stdout);
        for line in file_list.lines() {
            let clean_path = line.strip_prefix('.').unwrap_or(line);
            if clean_path.is_empty() || clean_path == "/" {
                continue;
            }

            let full_path = format!("/{}", clean_path.trim_start_matches('/'));
            installed_files.insert(
                full_path.clone(),
                FileMetadata {
                    mode: 0o644,
                    uid: 0,
                    gid: 0,
                    size: 0,
                },
            );
            self.db.register_file(&full_path, &pkg_name)?;
        }

        let pkg_info = InstalledPackage {
            name: pkg_name.clone(),
            description: t!("deb_description", name = file_name).to_string(),
            version: "deb-imported".to_string(),
            release: 1,
            package_hash: "deb-package".to_string(),
            install_date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            installed_files,
            total_size: 0,
            distribution_release: "1".to_string(),
            licenses: Vec::new(),
            provides: Vec::new(),
            post_remove: None,
            pre_remove: None,
            homepage: None,
            icon: None,
            screenshot: None,
            packager: None,
            install_tar_hash: None,
            package_format: None,
            build_host: None,
            distribution: None,
            configured: true,
            signature_verified: false,
        };

        self.db.install_package(&pkg_info)?;

        println!("{}", t!("deb_success", name = pkg_name));
        Ok(())
    }
}
