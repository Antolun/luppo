use crate::comar::{ComarAction, ComarManager};
use crate::database::LuppoDatabase;
use crate::package::{FileMetadata, FileXmlEntry, FilesXmlRoot, InstalledPackage, Package, LuppoRoot, SourceInfo};
use crate::packager::Packager;
use crate::repo::Repository;
use crate::resolver::{PackageResolver, LuppoRepo};
use crate::LuppoError;
use chrono::Local;
use indicatif::{ProgressBar, ProgressStyle};
use luppo_spec::models::{HistoryAction, PackageDefinition};
use rayon::prelude::*;
use rust_i18n::t;
use sha1::{Digest, Sha1};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::io::{self, Read, Write};
use zip::ZipArchive;
use std::path::{Path, PathBuf};
use std::process::Command;

type LuppoResult<T> = Result<T, LuppoError>;

pub struct Installer {
    db: LuppoDatabase,
    config: crate::config::Config,
}

impl Installer {
    pub fn new(db: LuppoDatabase, config: crate::config::Config) -> Self {
        Installer { db, config }
    }

    /// Veritabanını yedekle
    pub fn backup_db(&self, backup_dir: &PathBuf) -> LuppoResult<()> {
        self.db.backup_to_dir(backup_dir)
    }

    /// Veritabanını yedekten geri yükle
    pub fn restore_db(&self, backup_dir: &PathBuf) -> LuppoResult<()> {
        self.db.restore_from_dir(backup_dir)
    }

    /// Veritabanı bütünlüğünü kontrol et
    pub fn verify_db(&self) -> LuppoResult<()> {
        self.db.verify_integrity()
    }

    fn dest_dir(&self) -> &Path {
        &self.config.general.destination_directory
    }

    /// SELinux etiketleme desteği - restorecon çalıştır
    fn restorecon_paths(&self, paths: &[String]) -> LuppoResult<()> {
        let selinux_enabled = Path::new("/sys/fs/selinux/enforce").exists()
            && fs::read_to_string("/sys/fs/selinux/enforce")
                .map(|s| s.trim() == "1")
                .unwrap_or(false);

        if !selinux_enabled {
            return Ok(());
        }

        let dest_dir = self.dest_dir();
        for rel_path in paths {
            let full_path = dest_dir.join(rel_path.trim_start_matches('/'));
            if full_path.exists() {
                let status = Command::new("restorecon")
                    .args(["-F", "-v", full_path.to_str().unwrap_or("")])
                    .status()
                    .map_err(|e| LuppoError::RuntimeError(format!("restorecon failed: {}", e)))?;
                if !status.success() {
                    eprintln!(
                        "{}",
                        t!(
                            "install_selinux_restorecon_warn",
                            path = full_path.display()
                        )
                    );
                }
            }
        }
        Ok(())
    }

    /// Sistem kullanıcılarını oluştur (spec dosyasındaki Users bölümünden)
    fn create_users(
        &self,
        users: &luppo_spec::models::UsersWrapper,
    ) -> LuppoResult<()> {
        for user in &users.users {
            if Self::user_exists(&user.name) {
                println!(
                    "{}",
                    t!(
                        "install_user_exists",
                        name = user.name
                    )
                );
                continue;
            }

            let mut cmd = Command::new("useradd");
            cmd.args(["-r", "-s", "/usr/sbin/nologin"]); // System user, no shell

            if let Some(home) = &user.home {
                cmd.arg("-d").arg(home);
                cmd.arg("-m"); // Create home directory
            } else {
                cmd.arg("-M"); // Don't create home
            }

            if let Some(shell) = &user.shell {
                cmd.arg("-s").arg(shell);
            } else {
                cmd.arg("-s").arg("/usr/sbin/nologin");
            }

            if let Some(uid) = user.uid {
                cmd.arg("-u").arg(uid.to_string());
            }

            if let Some(gid) = user.gid {
                cmd.arg("-g").arg(gid.to_string());
            }

            cmd.arg(&user.name);

            println!(
                "{}",
                t!(
                    "install_user_creating",
                    name = user.name
                )
            );

            let status = cmd.status().map_err(|e| LuppoError::RuntimeError(e.to_string()))?;
            if !status.success() {
                return Err(LuppoError::RuntimeError(
                    t!(
                        "install_user_create_failed",
                        name = user.name
                    ).to_string()
                ));
            }
        }
        Ok(())
    }

    /// Sistem gruplarını oluştur (spec dosyasındaki Groups bölümünden)
    fn create_groups(
        &self,
        groups: &luppo_spec::models::GroupsWrapper,
    ) -> LuppoResult<()> {
        for group in &groups.groups {
            if Self::group_exists(&group.name) {
                println!(
                    "{}",
                    t!(
                        "install_group_exists",
                        name = group.name
                    )
                );
                continue;
            }

            let mut cmd = Command::new("groupadd");
            cmd.arg("-r"); // System group

            if let Some(gid) = group.gid {
                cmd.arg("-g").arg(gid.to_string());
            }

            cmd.arg(&group.name);

            println!(
                "{}",
                t!(
                    "install_group_creating",
                    name = group.name
                )
            );

            let status = cmd.status().map_err(|e| LuppoError::RuntimeError(e.to_string()))?;
            if !status.success() {
                return Err(LuppoError::RuntimeError(
                    t!(
                        "install_group_create_failed",
                        name = group.name
                    ).to_string()
                ));
            }
        }
        Ok(())
    }

    fn user_exists(name: &str) -> bool {
        Command::new("id")
            .arg("-u")
            .arg(name)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn group_exists(name: &str) -> bool {
        Command::new("getent")
            .arg("group")
            .arg(name)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Basit paket kurulumu (Test veya manuel girişler için)
    pub fn install_packages(&self, packages: &[String]) -> LuppoResult<()> {
        println!("{}", t!("install_analyzing", count = packages.len()));

        for package_name in packages {
            let version = "1.0.0";
            println!(
                "{}",
                t!(
                    "install_installing",
                    package = package_name,
                    version = version
                )
            );

            let mut files = HashMap::new();
            let binary_path = format!("/usr/bin/{}", package_name);
            files.insert(
                binary_path.clone(),
                FileMetadata {
                    mode: 0o755,
                    uid: 0,
                    gid: 0,
                    size: 0,
                },
            );

            let pkg_info = InstalledPackage {
                name: package_name.clone(),
                description: t!("install_system_desc", package = package_name).to_string(),
                version: version.to_string(),
                release: 1,
                package_hash: "0000000000000000000000000000000000000000".to_string(), // Varsayılan hash
                install_date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                installed_files: files,
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
            self.db.register_file(&binary_path, package_name)?;
            println!(
                "{}",
                t!("success_package_installed", package = package_name)
            );
        }

        Ok(())
    }

    /// Belirtilen dosyaların başka bir paket tarafından sahiplenilip sahiplenilmediğini kontrol eder.
    pub fn check_file_conflicts(&self, pkg_name: &str, files: &HashSet<String>) -> LuppoResult<()> {
        for file_path in files {
            if let Ok(Some(owner)) = self.db.find_package_by_file(file_path) {
                if owner != pkg_name {
                    return Err(LuppoError::RuntimeError(
                        t!(
                            "install_file_conflict",
                            path = file_path,
                            owner = owner,
                            package = pkg_name
                        )
                        .to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Luppo dosyalarından gerçek kurulum yapan ana fonksiyon
    #[allow(clippy::too_many_arguments)]
    pub fn install_package_from_data(
        &self,
        pkg_name: &str,
        version: &str,
        release: u32,
        distribution_release: &str,
        hash: &str,
        description: &str,
        installed_files: HashMap<String, FileMetadata>,
        total_size: u64,
        configured: bool,
        signature_verified: bool,
    ) -> LuppoResult<()> {
        let pkg_info = InstalledPackage {
            name: pkg_name.to_string(),
            version: version.to_string(),
            release,
            package_hash: hash.to_string(),
            description: description.to_string(),
            install_date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            installed_files,
            total_size,
            distribution_release: distribution_release.to_string(),
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
            configured,
            signature_verified,
        };

        // Veritabanına dosya sahipliğini kaydet
        for file_path in pkg_info.installed_files.keys() {
            let _ = self.db.register_file(file_path, &pkg_info.name);
        }

        self.db.install_package(&pkg_info)?;
        Ok(())
    }

    // --- KALDIRMA MANTIKLARI ---

    pub fn remove_packages(
        &self,
        package_names: &[String],
        trace_id: u64,
        ignore_comar: bool,
    ) -> LuppoResult<()> {
        println!("{}", t!("remove_starting"));

        let comar = ComarManager::new(self.dest_dir());
        let mut affected_files = Vec::new();

        for name in package_names {
            match self.db.get_installed_package(name)? {
                Some(pkg) => {
                    affected_files.extend(pkg.installed_files.keys().cloned());

                    // 0. Paket kaldırma öncesi betiği (pre-remove) çalıştır
                    if let Some(script) = &pkg.pre_remove {
                        self.run_pre_remove(
                            &pkg.name,
                            script,
                            self.dest_dir().to_str().unwrap_or("/"),
                        )?;
                    }

                    if !ignore_comar {
                        if let Err(e) = comar.run_package_script(&pkg.name, ComarAction::PreRemove)
                        {
                            eprintln!(
                                "{}",
                                t!(
                                    "installer_comar_pre_remove_err",
                                    package = pkg.name,
                                    error = format!("{:?}", e)
                                )
                            );
                        }
                    }

                    // 1. Dosyalari diskten temizle
                    self.remove_package_files(&pkg)?;

                    // 2. Paket kaldırma sonrası betiği (post-remove) çalıştır
                    if let Some(script) = &pkg.post_remove {
                        self.run_post_remove(
                            &pkg.name,
                            script,
                            self.dest_dir().to_str().unwrap_or("/"),
                        )?;
                    }

                    // 3. Veritabanindan paketi sil
                    let pkg_dir = self.config.directories.packages_dir
                        .join(format!("{}-{}-{}", pkg.name, pkg.version, pkg.release));
                    let _ = fs::remove_dir_all(&pkg_dir);
                    self.db.remove_package(name)?;

                    if !ignore_comar {
                        if let Err(e) = comar.run_package_script(&pkg.name, ComarAction::PostRemove)
                        {
                            eprintln!(
                                "{}",
                                t!(
                                    "installer_comar_post_remove_err",
                                    package = pkg.name,
                                    error = format!("{:?}", e)
                                )
                            );
                        }
                        let _ = comar.remove_package(&pkg.name);
                    }

                    // 4. History Kaydı (Her paket için tek tek ve dogru turde)
                    self.db.record_action(HistoryAction {
                        trace_id,
                        operation: "remove".to_string(),
                        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        details: format!("{} v{}", pkg.name, pkg.version),
                    })?;

                    println!("{}", t!("remove_cleaned", package = name));
                }
                None => {
                    println!("{}", t!("remove_skipped_not_installed", package = name));
                }
            }
        }

        if !ignore_comar && !affected_files.is_empty() {
            if let Err(e) = comar.run_system_triggers(&affected_files) {
                eprintln!(
                    "{}",
                    t!("installer_comar_trigger_err", error = format!("{:?}", e))
                );
            }
        }

        Ok(())
    }

    /// Paket kurulumu öncesi betiği çalıştırır
    pub fn run_pre_install(&self, pkg_name: &str, script: &str, rootfs: &str) -> LuppoResult<()> {
        println!("{}", t!("script_pre_install", package = pkg_name));

        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(script)
            .env("LUPPO_DESTDIR", rootfs)
            .status()
            .map_err(|e| LuppoError::RuntimeError(t!("error_script_run", error = e).to_string()))?;

        if !status.success() {
            eprintln!(
                "{}",
                t!(
                    "installer_script_err",
                    package = pkg_name,
                    script = "pre-install",
                    code = format!("{:?}", status.code())
                )
            );
        }
        Ok(())
    }

    /// Paket kurulumu sonrası betiği çalıştırır
    pub fn run_post_install(&self, pkg_name: &str, script: &str, rootfs: &str) -> LuppoResult<()> {
        println!("{}", t!("script_post_install", package = pkg_name));

        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(script)
            .env("LUPPO_DESTDIR", rootfs)
            .status()
            .map_err(|e| LuppoError::RuntimeError(t!("error_script_run", error = e).to_string()))?;

        if !status.success() {
            eprintln!(
                "{}",
                t!(
                    "installer_script_err",
                    package = pkg_name,
                    script = "post-install",
                    code = format!("{:?}", status.code())
                )
            );
        }
        Ok(())
    }

    /// Paket güncellemesi öncesi betiği çalıştırır
    pub fn run_pre_upgrade(&self, pkg_name: &str, script: &str, rootfs: &str) -> LuppoResult<()> {
        println!("{}", t!("script_pre_upgrade", package = pkg_name));

        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(script)
            .env("LUPPO_DESTDIR", rootfs)
            .status()
            .map_err(|e| LuppoError::RuntimeError(t!("error_script_run", error = e).to_string()))?;

        if !status.success() {
            eprintln!(
                "{}",
                t!(
                    "installer_script_err",
                    package = pkg_name,
                    script = "pre-upgrade",
                    code = format!("{:?}", status.code())
                )
            );
        }
        Ok(())
    }

    /// Paket güncellemesi sonrası betiği çalıştırır
    pub fn run_post_upgrade(&self, pkg_name: &str, script: &str, rootfs: &str) -> LuppoResult<()> {
        println!("{}", t!("script_post_upgrade", package = pkg_name));

        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(script)
            .env("LUPPO_DESTDIR", rootfs)
            .status()
            .map_err(|e| LuppoError::RuntimeError(t!("error_script_run", error = e).to_string()))?;

        if !status.success() {
            eprintln!(
                "{}",
                t!(
                    "installer_script_err",
                    package = pkg_name,
                    script = "post-upgrade",
                    code = format!("{:?}", status.code())
                )
            );
        }
        Ok(())
    }

    /// Paket kaldırma öncesi betiği çalıştırır
    pub fn run_pre_remove(&self, pkg_name: &str, script: &str, rootfs: &str) -> LuppoResult<()> {
        println!("{}", t!("script_pre_remove", package = pkg_name));

        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(script)
            .env("LUPPO_DESTDIR", rootfs)
            .status()
            .map_err(|e| LuppoError::RuntimeError(t!("error_script_run", error = e).to_string()))?;

        if !status.success() {
            eprintln!(
                "{}",
                t!(
                    "installer_script_err",
                    package = pkg_name,
                    script = "pre-remove",
                    code = format!("{:?}", status.code())
                )
            );
        }
        Ok(())
    }

    /// Paket kaldırma sonrası betiği çalıştırır
    pub fn run_post_remove(&self, pkg_name: &str, script: &str, rootfs: &str) -> LuppoResult<()> {
        println!("{}", t!("script_post_remove", package = pkg_name));

        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(script)
            .env("LUPPO_DESTDIR", rootfs)
            .status()
            .map_err(|e| LuppoError::RuntimeError(t!("error_script_run", error = e).to_string()))?;

        if !status.success() {
            eprintln!(
                "{}",
                t!(
                    "installer_script_err",
                    package = pkg_name,
                    script = "post-remove",
                    code = format!("{:?}", status.code())
                )
            );
        }
        Ok(())
    }

    fn remove_package_files(&self, pkg: &InstalledPackage) -> LuppoResult<()> {
        println!(
            "{}",
            t!(
                "install_files_cleaning",
                package = pkg.name,
                count = pkg.installed_files.len()
            )
        );

        // 1. Dosya silme işlemlerini paralel olarak gerçekleştir ve başarıyla silinenleri topla
        let removed_files: Vec<&String> = pkg
            .installed_files
            .par_iter()
            .filter_map(|(file, _meta)| {
                let relative_path = file.trim_start_matches('/');
                let full_path = self.dest_dir().join(relative_path);

                if full_path.exists() {
                    if let Err(e) = fs::remove_file(&full_path) {
                        eprintln!(
                            "{}",
                            t!(
                                "installer_delete_err",
                                path = format!("{:?}", full_path),
                                error = e
                            )
                        );
                        return None;
                    }
                    return Some(file);
                }
                None
            })
            .collect();

        // 2. Veritabanı kayıtlarını ana kanal üzerinden güvenli (sıralı) şekilde sil
        for file in removed_files {
            let _ = self.db.remove_file_entry(file);
        }

        // 3. Boş kalan dizinleri topla (Hiyerarşik temizlik için)
        let mut dirs_to_check = HashSet::new();
        for file in pkg.installed_files.keys() {
            let relative_path = file.trim_start_matches('/');
            let mut current = self
                .dest_dir()
                .join(relative_path)
                .parent()
                .map(|p| p.to_path_buf());

            while let Some(dir) = current {
                if dir == self.dest_dir() || !dir.exists() {
                    break;
                }
                dirs_to_check.insert(dir.clone());
                current = dir.parent().map(|p| p.to_path_buf());
            }
        }

        // 4. Dizinleri derinliğe göre sırala ve temizle (En derinden başla)
        let mut sorted_dirs: Vec<_> = dirs_to_check.into_iter().collect();
        sorted_dirs.sort_by_key(|b| std::cmp::Reverse(b.components().count()));

        for dir in sorted_dirs {
            self.cleanup_if_empty(&dir);
        }

        Ok(())
    }

    fn cleanup_if_empty(&self, path: &Path) {
        let protected_paths = [
            self.dest_dir().join("usr/bin"),
            self.dest_dir().join("usr/share"),
            self.dest_dir().join("usr/lib"),
            self.dest_dir().join("etc"),
            self.dest_dir().join("var"),
            self.dest_dir().join("usr"),
            self.dest_dir().to_path_buf(),
        ];

        if protected_paths.iter().any(|p| path == p) {
            return;
        }

        if path.exists() && path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                if entries.count() == 0 && fs::remove_dir(path).is_ok() {
                    if let Some(parent) = path.parent() {
                        self.cleanup_if_empty(parent);
                    }
                }
            }
        }
    }

    // --- YARDIMCI MANTIKLAR ---

    pub fn calculate_remove_chain(&self, target_pkg: &str) -> LuppoResult<Vec<String>> {
        let mut to_remove = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(target_pkg.to_string());

        while let Some(current) = queue.pop_front() {
            if !to_remove.contains(&current) {
                to_remove.push(current.clone());
                let dependents = self.get_reverse_dependencies(&current)?;
                for dep in dependents {
                    queue.push_back(dep);
                }
            }
        }
        Ok(to_remove)
    }

    pub fn calculate_install_order(
        &self,
        initial_package_names: &[String],
    ) -> LuppoResult<Vec<String>> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();

        for pkg in initial_package_names {
            self.resolve_dependencies(pkg, &mut order, &mut visited);
        }

        Ok(order)
    }

    pub fn resolve_dependencies(
        &self,
        pkg_name: &str,
        order: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) {
        if visited.contains(pkg_name) {
            return;
        }
        visited.insert(pkg_name.to_string());

        let deps = self.get_package_dependencies(pkg_name);
        for dep in deps {
            self.resolve_dependencies(&dep, order, visited);
        }
        order.push(pkg_name.to_string());
    }

    pub fn get_reverse_dependencies(&self, package_name: &str) -> LuppoResult<Vec<String>> {
        let mut dependents = Vec::new();
        let installed = self.db.list_installed_packages()?;
        for pkg in installed {
            let deps = self.get_package_dependencies(&pkg.name);
            if deps.contains(&package_name.to_string()) {
                dependents.push(pkg.name);
            }
        }
        Ok(dependents)
    }

    pub fn get_package_dependencies(&self, pkg_name: &str) -> Vec<String> {
        if let Ok(Some(repo_pkg)) = self.db.get_available_package(pkg_name) {
            return repo_pkg
                .runtime_dependencies
                .map(|rd| rd.dependencies.clone())
                .unwrap_or_default();
        }
        vec![]
    }

    pub fn find_orphaned_packages(&self) -> LuppoResult<Vec<String>> {
        let all_installed = self.db.list_installed_packages()?;

        // Veritabanı erişimini paralel döngü dışına taşıyarak güvenliği ve hızı artırıyoruz
        let all_available = self.db.list_available_packages()?;
        let available_map: HashMap<String, Vec<String>> = all_available
            .into_iter()
            .map(|pkg| {
                (
                    pkg.name,
                    pkg.runtime_dependencies
                        .map(|rd| rd.dependencies)
                        .unwrap_or_default(),
                )
            })
            .collect();

        // 1. Tüm bağımlılıkları paralel olarak topla (Artık veritabanına dokunmuyor, haritadan okuyor)
        let required_dependencies: HashSet<String> = all_installed
            .par_iter()
            .flat_map(|pkg| available_map.get(&pkg.name).cloned().unwrap_or_default())
            .collect();

        let orphans = all_installed
            .into_iter()
            .filter(|pkg| {
                !required_dependencies.contains(&pkg.name)
                    && !["libc", "ssl", "base", "kernel"].contains(&pkg.name.as_str())
            })
            .map(|pkg| pkg.name)
            .collect();

        Ok(orphans)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn perform_install(
        &self,
        package_names: Vec<String>,
        trace_id: u64,
        force: bool,
        yes_all: bool,
        limit_kb: Option<usize>,
        auth: Option<(String, String)>,
        download_only: bool,
        ignore_check: bool,
        ignore_comar: bool,
        ignore_file_conflict: bool,
        ignore_package_conflict: bool,
        reinstall: bool,
        ignore_dependency: bool,
        reporter: Option<&dyn crate::progress::ProgressReporter>,
    ) -> LuppoResult<()> {
        let repo_manager = Repository::new(self.db.clone(), self.config.clone());
        // 1. Resolver ve Repo Hazırlığı
        let mut local_luppo_paths: HashMap<String, PathBuf> = HashMap::new();
        let mut local_packages: HashMap<String, crate::package::Package> = HashMap::new();
        let mut resolved_package_queries = Vec::new();

        let mut repo = LuppoRepo::new(self.db.clone());

        for name in &package_names {
            let path = std::path::Path::new(name);
            if name.ends_with(".luppo") || path.is_file() {
                let mut pkg_data = Packager::read_package(name)?;

                // Calculate the SHA1 hash of the local .luppo file itself for the integrity check
                let mut file = fs::File::open(path).map_err(LuppoError::IoError)?;
                let mut hasher = Sha1::new();
                io::copy(&mut file, &mut hasher).map_err(LuppoError::IoError)?;
                let luppo_file_sha1 = format!("{:x}", hasher.finalize());
                pkg_data.metadata.package_hash = luppo_file_sha1;

                let runtime_deps_list = pkg_data
                    .metadata
                    .runtime_dependencies
                    .as_ref()
                    .map(|rd| rd.dependencies.clone())
                    .unwrap_or_default();

                let pkg_def = PackageDefinition {
                    name: pkg_data.metadata.name.clone(),
                    version: pkg_data
                        .metadata
                        .history
                        .updates
                        .first()
                        .map(|u| u.version.clone())
                        .unwrap_or_else(|| "1.0.0".to_string()),
                    summary: pkg_data
                        .metadata
                        .summaries
                        .first()
                        .map(|s| s.text.clone())
                        .unwrap_or_default(),
                    description: pkg_data
                        .metadata
                        .descriptions
                        .first()
                        .map(|d| d.text.clone())
                        .unwrap_or_default(),
                    homepage: None,
                    icon: None,
                    screenshot: None,
                    provides: Some(luppo_spec::models::ProvidesBlock {
                        isa: pkg_data.metadata.provides.clone(),
                        comar: Vec::new(),
                    }),
                    additional_files: None,
                    build_type: None,
                    license: pkg_data
                        .metadata
                        .licenses
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "GPL".to_string()),
                    packager: luppo_spec::models::Packager {
                        name: pkg_data
                            .metadata
                            .history
                            .updates
                            .first()
                            .map(|u| u.name.clone())
                            .unwrap_or_else(|| "Luppo Community".to_string()),
                        email: pkg_data
                            .metadata
                            .history
                            .updates
                            .first()
                            .and_then(|u| u.email.clone())
                            .unwrap_or_else(|| "info@antolun.com".to_string()),
                    },
                    deps: luppo_spec::models::Dependencies {
                        runtime: runtime_deps_list.clone(),
                        conflicts: pkg_data
                            .metadata
                            .conflicts
                            .as_ref()
                            .map(|c| c.packages.clone())
                            .unwrap_or_default(),
                        build: pkg_data
                            .metadata
                            .build_dependencies
                            .as_ref()
                            .map(|b| b.dependencies.clone())
                            .unwrap_or_default(),
                    },
                    actions: luppo_spec::models::PackageActions {
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
                    files: luppo_spec::models::Files::default(),
                    runtime_dependencies: Some(luppo_spec::models::RuntimeDeps {
                        dependencies: runtime_deps_list
                            .into_iter()
                            .map(|d| luppo_spec::models::Dependency {
                                name: d,
                                ..Default::default()
                            })
                            .collect(),
                        ..Default::default()
                    }),
                    ..Default::default()
                };

                resolved_package_queries.push(pkg_def.name.clone());
                local_luppo_paths.insert(pkg_def.name.clone(), PathBuf::from(name));
                local_packages.insert(pkg_def.name.clone(), pkg_data.metadata);
                repo.packages.insert(pkg_def.name.clone(), pkg_def);
            } else {
                resolved_package_queries.push(name.clone());
            }
        }

        for name in &resolved_package_queries {
            if !local_packages.contains_key(name) && self.db.get_available_package(name)?.is_none() {
                return Err(LuppoError::RuntimeError(
                    t!("resolver_error_not_found", name = name).to_string(),
                ));
            }
        }

        let mut resolver = PackageResolver::new(self.db.clone(), repo.clone());
        resolver.ignore_package_conflict = ignore_package_conflict;
        resolver.reinstall = reinstall;
        resolver.ignore_dependency = ignore_dependency;

        let mut installation_plan: Vec<PackageDefinition> = Vec::new();

        println!(
            "{}",
            t!("install_analyzing", count = resolved_package_queries.len())
        );

        // Çakışma durumunda otomatik kaldırma ve yeniden deneme döngüsü
        loop {
            match resolver.resolve_deps(&resolved_package_queries) {
                Ok(plan) => {
                    for pkg in plan {
                        if !installation_plan.iter().any(|p| p.name == pkg.name) {
                            installation_plan.push(pkg);
                        }
                    }
                    break;
                }
                Err(LuppoError::InstalledConflict {
                    package,
                    conflicting_package,
                }) => {
                    println!(
                        "{}",
                        t!(
                            "install_conflict_detected",
                            package = package,
                            conflicting = conflicting_package
                        )
                    );

                    let should_remove = if yes_all {
                        true
                    } else {
                        print!(
                            "{}",
                            t!("install_remove_conflict_q", package = conflicting_package)
                        );
                        io::stdout().flush().unwrap();
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();
                        let input = input.trim().to_lowercase();
                        input == "e" || input == "y"
                    };

                    if should_remove {
                        println!(
                            "{}",
                            t!(
                                "install_removing_conflicting",
                                package = conflicting_package
                            )
                        );
                        self.perform_remove(
                            conflicting_package,
                            trace_id,
                            yes_all,
                            ignore_comar,
                            false,
                            false,
                            reporter,
                        )?;

                        // Çözümleyiciyi sıfırla ve tekrar dene
                        resolver = PackageResolver::new(self.db.clone(), repo.clone());
                        resolver.ignore_package_conflict = ignore_package_conflict;
                        continue;
                    } else {
                        return Err(LuppoError::RuntimeError(
                            t!("error_install_cancelled_conflict", package = package).to_string(),
                        ));
                    }
                }
                Err(e) => {
                    println!(
                        "{}",
                        t!("installer_deps_solve_err", error = format!("{:?}", e))
                    );
                    return Err(e);
                }
            }
        }

        if installation_plan.is_empty() {
            println!("{}", t!("install_no_new_packages"));
            return Ok(());
        }

        // --- KURULUM ÖZETİ VE ONAY ---
        println!("{}", t!("install_flow_title"));
        println!("{}", "━".repeat(60));
        for (i, pkg) in installation_plan.iter().enumerate() {
            let symbol = if i == 0 {
                "🏁"
            } else if i == installation_plan.len() - 1 {
                "🎁"
            } else {
                "🔗"
            };
            println!(
                "  {} [{:>2}] {:<30} v{}",
                symbol,
                i + 1,
                pkg.name,
                pkg.version
            );
            if i < installation_plan.len() - 1 {
                println!("      ┃");
            }
        }
        println!("{}", "━".repeat(60));
        println!(
            "{}",
            t!("install_total_count", count = installation_plan.len())
        );

        if !yes_all {
            print!("{}", t!("install_continue_q"));
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();

            if !input.is_empty() && input != "e" && input != "y" {
                println!("{}", t!("install_user_cancelled"));
                return Ok(());
            }
        } else {
            println!("{}", t!("installer_yes_all"));
        }

        // --- AŞAMA 1: TOPLU İNDİRME (BATCH DOWNLOAD) ---
        println!("{}", t!("install_downloading"));
        let mut downloaded_packages = Vec::new();
        let all_available = self.db.list_available_packages()?;

        for pkg_def in &installation_plan {
            if let Some(local_path) = local_luppo_paths.get(&pkg_def.name) {
                let local_pkg = local_packages.get(&pkg_def.name).cloned().unwrap();
                downloaded_packages.push((pkg_def.clone(), local_pkg, local_path.clone()));
            } else {
                let full_pkg_data = all_available
                    .iter()
                    .find(|p| p.name == pkg_def.name)
                    .ok_or_else(|| {
                        LuppoError::RuntimeError(
                            t!("error_pkg_not_found_repo", package = pkg_def.name).to_string(),
                        )
                    })?;

                let downloaded_path =
                    repo_manager.fetch_package(full_pkg_data, None, limit_kb, auth.clone(), reporter)?;
                downloaded_packages.push((pkg_def.clone(), full_pkg_data.clone(), downloaded_path));
            }
        }

        if download_only {
            println!("{}", t!("installer_download_only"));
            return Ok(());
        }

        // --- AŞAMA 1.5: DOĞRULAMA ---
        if !ignore_check {
            println!("{}", t!("install_verifying"));
            for (_, full_pkg_data, luppo_path) in &downloaded_packages {
                let mut file = fs::File::open(luppo_path).map_err(LuppoError::IoError)?;
                let mut hasher = Sha1::new();
                io::copy(&mut file, &mut hasher).map_err(LuppoError::IoError)?;
                let hash = format!("{:x}", hasher.finalize());
                if hash != full_pkg_data.package_hash {
                    return Err(LuppoError::RuntimeError(
                        t!(
                            "error_integrity_hash_mismatch",
                            package = full_pkg_data.name
                        )
                        .to_string(),
                    ));
                }
            }
        }

        // --- AŞAMA 2: TOPLU KURULUM (BATCH INSTALL) ---
        let total_to_install = downloaded_packages.len();
        println!("{}", t!("install_starting", count = total_to_install));

        let mut all_affected_files = Vec::new();

        for (index, (pkg_def, full_pkg_data, luppo_path)) in
            downloaded_packages.into_iter().enumerate()
        {
            let current_count = index + 1;
            let pkg_name = pkg_def.name.clone();
            let new_version = pkg_def.version.clone();
            let mut is_upgrade = false;

            if let Ok(Some(old_pkg)) = self.db.get_installed_package(&pkg_name) {
                if old_pkg.version == new_version && !force && !reinstall {
                    println!(
                        "{}",
                        t!(
                            "install_already_uptodate",
                            current = current_count,
                            total = total_to_install,
                            package = pkg_name
                        )
                    );
                    continue;
                }
                is_upgrade = true;
            }

            println!(
                "{}",
                t!(
                    "install_batch_count",
                    current = current_count,
                    total = total_to_install,
                    package = pkg_name
                )
            );

            // --- GPG İMZA DOĞRULAMA ---
            let signature_verified = self.verify_package_signature(&luppo_path)?;

            let is_delta_pkg = luppo_path
                .to_str()
                .map(|s| s.contains(".delta.luppo"))
                .unwrap_or(false);

            let mut package_data = Packager::read_package(luppo_path.to_str().unwrap())?;

            // SELinux etiketleme için dosya yollarını topla (dosya kurulumundan önce)
            let selinux_paths: Vec<String> = package_data.files.iter().map(|f| f.path.clone()).collect();

            // --- DELTA UYGULAMA ---
            // Eğer bu bir .delta.luppo ise, sadece değişen dosyaları içerir.
            // Kurulu paketin mevcut dosyalarını diskten okuyarak eksik dosyaları tamamlıyoruz.
            if is_delta_pkg {
                if let Ok(Some(old_pkg)) = self.db.get_installed_package(&pkg_name) {
                    println!("{}", t!("install_delta_applying", package = pkg_name));
                    let delta_paths: std::collections::HashSet<String> =
                        package_data.files.iter().map(|f| f.path.clone()).collect();

                    for (old_file_path, old_file_meta) in &old_pkg.installed_files {
                        // Delta'da olmayan (değişmemiş) dosyaları disk üzerinden okuyarak listeye ekle
                        if !delta_paths.contains(old_file_path) {
                            let disk_path =
                                self.dest_dir().join(old_file_path.trim_start_matches('/'));
                            if disk_path.exists() && disk_path.is_file() {
                                if let Ok(content) = fs::read(&disk_path) {
                                    package_data.files.push(crate::package::FileData {
                                        path: old_file_path.clone(),
                                        content,
                                        mode: old_file_meta.mode,
                                        uid: old_file_meta.uid,
                                        gid: old_file_meta.gid,
                                        size: old_file_meta.size,
                                        symlink_target: None,
                                    });
                                }
                            }
                        }
                    }
                } else {
                    return Err(LuppoError::RuntimeError(
                        t!("install_delta_no_base", package = pkg_name).to_string(),
                    ));
                }
            }

            let files_to_check: HashSet<String> =
                package_data.files.iter().map(|f| f.path.clone()).collect();
            if !force && !ignore_file_conflict {
                self.check_file_conflicts(&pkg_name, &files_to_check)?;
            }

            if !ignore_comar {
                if is_upgrade {
                    if let Some(script) = &package_data.metadata.pre_upgrade {
                        self.run_pre_upgrade(
                            &pkg_name,
                            script,
                            self.dest_dir().to_str().unwrap_or("/"),
                        )?;
                    }
                } else if let Some(script) = &package_data.metadata.pre_install {
                    self.run_pre_install(
                        &pkg_name,
                        script,
                        self.dest_dir().to_str().unwrap_or("/"),
                    )?;
                }
            }

            // --- KULLANICI/GRUP OLUŞTURMA (pre-install'dan önce) ---
            if let Some(users) = &package_data.metadata.users {
                self.create_users(users)?;
            }
            if let Some(groups) = &package_data.metadata.groups {
                self.create_groups(groups)?;
            }

            let files_xml_opt = package_data.files_xml.clone();
            let total_files = package_data.files.len() as u64;
            let pb = ProgressBar::new(total_files);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap(),
            );

            if let Some(r) = reporter {
                r.on_message(&t!("installer_extracting", pkg = &pkg_name));
            }

            let mut installed_files = HashMap::new();
            let mut total_extracted_size: u64 = 0;
            let mut files_done: u64 = 0;

            for file in package_data.files {
                let target_path = self.dest_dir().join(file.path.trim_start_matches('/'));
                if file.path.starts_with("/etc/") && target_path.exists() {
                    let backup_path =
                        PathBuf::from(format!("{}.config-backup", target_path.display()));
                    let _ = fs::rename(&target_path, &backup_path);
                }
                if let Some(parent) = target_path.parent() {
                    if parent.exists() && !parent.is_dir() {
                        let _ = fs::remove_file(parent);
                    } else if parent.is_symlink() && !parent.exists() {
                        // Broken symlink
                        let _ = fs::remove_file(parent);
                    }
                    fs::create_dir_all(parent)?;
                }
                total_extracted_size += file.content.len() as u64;

                // Dosya, symlink veya dizin zaten varsa kaldır
                if target_path.exists() || target_path.is_symlink() {
                    if target_path.is_dir() {
                        let _ = fs::remove_dir_all(&target_path);
                    } else {
                        let _ = fs::remove_file(&target_path);
                    }
                }

                if let Some(ref link_target) = file.symlink_target {
                    #[cfg(unix)]
                    {
                        std::os::unix::fs::symlink(link_target, &target_path)
                            .map_err(LuppoError::IoError)?;
                    }
                } else {
                    fs::write(&target_path, &file.content)?;
                }

                // --- DOSYA İZİNLERİNİ VE SAHİPLİĞİNİ UYGULA ---

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if file.symlink_target.is_none() {
                        let _ = fs::set_permissions(
                            &target_path,
                            fs::Permissions::from_mode(file.mode),
                        );
                    }

                    // uid/gid uygulama (chown)
                    let uid = file.uid as u32;
                    let gid = file.gid as u32;
                    if file.symlink_target.is_some() {
                        let _ = crate::util::lchown_path(target_path.as_path(), uid, gid);
                    } else {
                        let _ = std::os::unix::fs::chown(
                            target_path.as_path(),
                            Some(uid),
                            Some(gid),
                        );
                    };
                }

                installed_files.insert(
                    file.path.clone(),
                    FileMetadata {
                        mode: file.mode,
                        uid: file.uid,
                        gid: file.gid,
                        size: file.size,
                    },
                );
                files_done += 1;
                pb.inc(1);
                if let Some(r) = reporter {
                    if total_files > 0 {
                        r.on_progress(files_done as f32 / total_files as f32, None, None);
                    }
                }
            }
            pb.finish_with_message(t!("installer_pkg_finished", pkg = pkg_name).to_string());
            if let Some(r) = reporter {
                r.on_finish(&t!("installer_pkg_finished", pkg = pkg_name));
            }

            // --- SELINUX ETİKETLEME (restorecon) ---
            let _ = self.restorecon_paths(&selinux_paths);

            // --- DELTA OPTİMİZASYONLARI & DOSYA İZLEME (FILE TRACKING) ---
            if let Some(files_xml) = files_xml_opt {
                installed_files.clear(); // Sadece değişen dosyaları değil, tam listeyi alıyoruz
                for fx in files_xml {
                    let mode = u32::from_str_radix(&fx.mode, 8).unwrap_or(0o644);
                    installed_files.insert(
                        fx.path.clone(),
                        FileMetadata {
                            mode,
                            uid: fx.uid,
                            gid: fx.gid,
                            size: fx.size,
                        },
                    );
                }

                if is_upgrade {
                    if let Ok(Some(old_pkg)) = self.db.get_installed_package(&pkg_name) {
                        for old_file_path in old_pkg.installed_files.keys() {
                            if !installed_files.contains_key(old_file_path) {
                                let target_path =
                                    self.dest_dir().join(old_file_path.trim_start_matches('/'));
                                if target_path.exists() && target_path.is_file() {
                                    println!(
                                        "{}",
                                        t!("install_file_removed", path = old_file_path)
                                    );
                                    let _ = fs::remove_file(&target_path);
                                }
                            }
                        }
                    }
                }
            }

            let mut configured = true;
            if !ignore_comar {
                if is_upgrade {
                    if let Some(script) = &package_data.metadata.post_upgrade {
                        if self
                            .run_post_upgrade(
                                &pkg_name,
                                script,
                                self.dest_dir().to_str().unwrap_or("/"),
                            )
                            .is_err()
                        {
                            configured = false;
                        }
                    }
                } else if let Some(script) = &package_data.metadata.post_install {
                    if self
                        .run_post_install(
                            &pkg_name,
                            script,
                            self.dest_dir().to_str().unwrap_or("/"),
                        )
                        .is_err()
                    {
                        configured = false;
                    }
                }
            }

            // COMAR scriptlerini .luppo zip'inden çıkar (install.tar.xz dışında ayrı ZIP entry)
            if let Ok(comar_zip) = std::fs::File::open(&luppo_path) {
                if let Ok(mut archive) = ZipArchive::new(comar_zip) {
                    for i in 0..archive.len() {
                        if let Ok(mut entry) = archive.by_index(i) {
                            let name = entry.name().to_string();
                            if !name.starts_with("comar/") { continue; }
                            let mut script_content = Vec::new();
                            let _ = entry.read_to_end(&mut script_content);
                            let target = self.dest_dir().join("var/lib/luppo/package")
                                .join(&pkg_name).join(&name);
                            if let Some(parent) = target.parent() {
                                let _ = fs::create_dir_all(parent);
                            }
                            let _ = fs::write(&target, &script_content);
                        }
                    }
                }
            }

            if !ignore_comar {
                let comar = ComarManager::new(self.dest_dir());
                let provides_comar = package_data.metadata.provides_block.as_ref()
                    .map(|p| p.comar.clone())
                    .unwrap_or_default();
                let mut has_self_post = false;

                for cp in &provides_comar {
                    let Some(cp_script) = &cp.script else {
                        continue;
                    };
                    let script_path = self.dest_dir().join(format!(
                        "var/lib/luppo/package/{}/comar/{}",
                        pkg_name, cp_script
                    ));
                    let app_name = cp.name.clone().unwrap_or_else(|| pkg_name.clone());
                    let script_str = script_path.to_str().unwrap_or("");

                    if let Err(e) = comar.register_package(&app_name, &cp.provide, script_str) {
                        eprintln!("{}", t!("installer_comar_register_err", app = app_name, error = format!("{:?}", e)));
                    }

                    if cp.provide == "System.Service" {
                        let _ = comar.register_service_state(&app_name);
                    }

                    if cp.provide == "System.Package" {
                        has_self_post = true;
                    }
                }

                let _ = comar.run_package_script(&pkg_name, ComarAction::Configure);

                // Only call postInstall if package provides System.Package
                if has_self_post {
                    let mut from_version = String::new();
                    let mut from_release = String::new();
                    if is_upgrade {
                        if let Ok(Some(old_pkg)) = self.db.get_installed_package(&pkg_name) {
                            from_version = old_pkg.version;
                            from_release = old_pkg.release.to_string();
                        }
                    }

                    let action = ComarAction::PostInstall {
                        from_version,
                        from_release,
                        to_version: new_version.clone(),
                        to_release: full_pkg_data.release.to_string(),
                    };

                    if let Err(e) = comar.run_package_script(&pkg_name, action) {
                        eprintln!(
                            "{}",
                            t!(
                                "installer_comar_config_err",
                                package = pkg_name,
                                error = format!("{:?}", e)
                            )
                        );
                    }
                }
            }

            let pkg_info = InstalledPackage {
                name: pkg_name.clone(),
                version: new_version.clone(),
                release: full_pkg_data.release,
                distribution_release: full_pkg_data.distribution_release.clone(),
                package_hash: full_pkg_data.package_hash.clone(),
                description: pkg_def.description.clone(),
                install_date: Local::now().to_rfc3339(),
                installed_files,
                total_size: total_extracted_size,
                licenses: full_pkg_data.licenses.clone(),
                provides: full_pkg_data.provides.clone(),
                post_remove: package_data.metadata.post_remove.clone(),
                pre_remove: package_data.metadata.pre_remove.clone(),
                homepage: full_pkg_data.effective_homepage(),
                icon: full_pkg_data.icon.clone(),
                screenshot: full_pkg_data.effective_screenshot(),
                packager: full_pkg_data.effective_packager(),
                install_tar_hash: full_pkg_data.install_tar_hash.clone(),
                package_format: full_pkg_data.package_format.clone(),
                build_host: full_pkg_data.build_host.clone(),
                distribution: full_pkg_data.distribution.clone(),
                configured,
                signature_verified,
            };
            self.db.install_package(&pkg_info)?;

            // Python Luppo uyumluluğu için filesystem'e metadata.xml ve files.xml yaz
            let pkg_dir = self.config.directories.packages_dir
                .join(format!("{}-{}-{}", pkg_name, new_version, full_pkg_data.release));
            let _ = fs::create_dir_all(&pkg_dir);
            if let Ok(pkg_dir) = std::fs::canonicalize(&pkg_dir) {
                let pkg_name = full_pkg_data.name.clone();
                let src_homepage = full_pkg_data.source.as_ref()
                    .and_then(|s| s.homepage.clone()).unwrap_or_default();
                let src_packager = full_pkg_data.source.as_ref()
                    .and_then(|s| s.packager.clone());
                let top_source = SourceInfo {
                    name: pkg_name.clone(),
                    homepage: src_homepage.clone(),
                    packager: src_packager.clone(),
                };
                let mut pkg_data = full_pkg_data.clone();
                pkg_data.source = Some(crate::package::PackageSource {
                    name: Some(pkg_name),
                    homepage: Some(src_homepage),
                    packager: src_packager,
                    screenshot: None,
                });
                let luppo_root = LuppoRoot {
                    source: Some(top_source),
                    package: pkg_data,
                };
                let mut buffer = String::new();
                {
                    let mut serializer = quick_xml::se::Serializer::new(&mut buffer);
                    serializer.indent(' ', 2);
                    use serde::Serialize;
                    let _ = luppo_root.serialize(serializer);
                }
                let _ = fs::write(pkg_dir.join("metadata.xml"), &buffer);

                let file_entries: Vec<FileXmlEntry> = pkg_info.installed_files.iter().map(|(path, meta)| {
                    let file_type = if meta.mode & 0o111 != 0 { "executable" }
                        else if path.starts_with("etc/") || path.starts_with("/etc/") { "config" }
                        else if path.starts_with("usr/share/doc/") || path.starts_with("/usr/share/doc/") { "doc" }
                        else if path.starts_with("usr/share/man/") || path.starts_with("/usr/share/man/") { "man" }
                        else if path.ends_with(".so") || path.ends_with(".a") { "library" }
                        else if path.ends_with(".h") || path.contains("/include/") { "header" }
                        else { "data" };
                    FileXmlEntry {
                        path: path.trim_start_matches('/').to_string(),
                        file_type: file_type.to_string(),
                        size: meta.size,
                        uid: meta.uid,
                        gid: meta.gid,
                        mode: format!("{:04o}", meta.mode),
                        hash: None,
                    }
                }).collect();
                let files_root = FilesXmlRoot { files: file_entries };
                let mut buffer = String::new();
                {
                    let mut serializer = quick_xml::se::Serializer::new(&mut buffer);
                    serializer.indent(' ', 2);
                    use serde::Serialize;
                    let _ = files_root.serialize(serializer);
                }
                let _ = fs::write(pkg_dir.join("files.xml"), &buffer);
            }

            let old_version = if is_upgrade {
                self.db.get_installed_package(&pkg_name).ok().flatten().map(|p| p.version)
            } else {
                None
            };

            let (op_type, action_details) = if let Some(old_ver) = old_version {
                (
                    "update".to_string(),
                    format!("{} (v{} -> v{})", pkg_name, old_ver, new_version),
                )
            } else {
                (
                    "install".to_string(),
                    format!("{} v{}", pkg_name, new_version),
                )
            };

            self.db.record_action(HistoryAction {
                trace_id,
                operation: op_type,
                timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                details: action_details,
            })?;

            if !ignore_comar {
                if let Ok(Some(installed_pkg)) = self.db.get_installed_package(&pkg_name) {
                    all_affected_files.extend(installed_pkg.installed_files.keys().cloned());
                }
            }

            println!(
                "{}",
                t!("install_success", package = pkg_name, version = new_version)
            );
        }

        if !ignore_comar && !all_affected_files.is_empty() {
            let comar = ComarManager::new(self.dest_dir());
            if let Err(e) = comar.run_system_triggers(&all_affected_files) {
                eprintln!(
                    "{}",
                    t!("installer_system_trigger_err", error = format!("{:?}", e))
                );
            }
        }

        println!("{}", t!("success_all_completed"));
        Ok(())
    }

    pub fn perform_remove(
        &self,
        package_name: String,
        trace_id: u64,
        yes_all: bool,
        ignore_comar: bool,
        ignore_dependency: bool,
        ignore_safety: bool,
        _reporter: Option<&dyn crate::progress::ProgressReporter>,
    ) -> LuppoResult<()> {
        // 1. Kurulu mu kontrol et
        if !self.db.is_package_installed(&package_name)? {
            println!(
                "{}",
                t!("error_package_not_installed", package = package_name)
            );
            return Ok(());
        }

        // Güvenlik kilidi (safety switch) kontrolü: system.base paketlerinin kaldırılmasını engelle
        if !ignore_safety {
            if let Ok(Some(pkg)) = self.db.get_package(&package_name) {
                if pkg.partof == "system.base" || pkg.partof.starts_with("system.base") {
                    return Err(LuppoError::RuntimeError(
                        t!("error_safety_switch", package = package_name).into(),
                    ));
                }
            }
        }

        // 2. Silme zincirini hesapla (Ters bağımlılıklar)
        let remove_chain = if ignore_dependency {
            vec![package_name.clone()]
        } else {
            self.calculate_remove_chain(&package_name)?
        };

        // 3. Kullanıcıya Planı Göster
        println!("{}", t!("remove_plan_title"));
        println!("{}", "-".repeat(45));
        for pkg in &remove_chain {
            if pkg == &package_name {
                println!("{}", t!("install_remove_chain_main", package = pkg));
            } else {
                println!("{}", t!("install_remove_chain_dep", package = pkg));
            }
        }
        println!("{}", "-".repeat(45));
        println!("{}", t!("remove_total_count", count = remove_chain.len()));

        // 4. Onay Al
        if !yes_all {
            print!("{}", t!("remove_continue_q"));
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            let input = input.trim().to_lowercase();
            if input != "e" && input != "y" {
                println!("{}", t!("install_user_cancelled"));
                return Ok(());
            }
        } else {
            println!("{}", t!("installer_yes_all"));
        }

        // 5. Uygulama
        self.remove_packages(&remove_chain, trace_id, ignore_comar)?;

        println!("{}", t!("success_all_completed"));
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn perform_upgrade(
        &self,
        package_names: Vec<String>,
        trace_id: u64,
        yes_all: bool,
        check_only: bool,
        integrity_only: bool,
        component: Option<String>,
        no_integrity: bool,
        limit_kb: Option<usize>,
        auth: Option<(String, String)>,
        download_only: bool,
        ignore_check: bool,
        ignore_comar: bool,
        ignore_file_conflict: bool,
        ignore_package_conflict: bool,
        ignore_dependency: bool,
        reporter: Option<&dyn crate::progress::ProgressReporter>,
    ) -> LuppoResult<()> {
        let title_suffix = if integrity_only {
            t!("upgrade_title_integrity_only").to_string()
        } else if no_integrity {
            t!("upgrade_title_no_integrity").to_string()
        } else if check_only {
            t!("upgrade_title_check_only").to_string()
        } else {
            "".to_string()
        };
        println!("{}", t!("upgrade_title_prefix", suffix = title_suffix));

        let installed_packages = if package_names.is_empty() {
            self.db.list_installed_packages()?
        } else {
            let mut pkgs = Vec::new();
            for name in &package_names {
                if let Ok(Some(pkg)) = self.db.get_installed_package(name) {
                    pkgs.push(pkg);
                } else {
                    println!("{}", t!("error_package_not_installed", package = name));
                }
            }
            pkgs
        };

        if installed_packages.is_empty() && !package_names.is_empty() {
            return Ok(());
        }

        let mut packages_to_process = Vec::new();

        let all_available = self.db.list_available_packages()?;
        let available_map: HashMap<String, Package> = all_available
            .into_iter()
            .map(|p| (p.name.clone(), p))
            .collect();

        let upgrade_results: Vec<(String, String)> = installed_packages
            .into_par_iter()
            .filter_map(|installed| {
                let repo_pkg = available_map.get(&installed.name);
                let mut needs_action = false;
                let mut reason = String::new();

                if let Some(comp_filter) = &component {
                    if let Some(repo_pkg) = repo_pkg {
                        if &repo_pkg.partof != comp_filter {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }

                if !integrity_only {
                    if let Some(repo_pkg) = repo_pkg {
                        let version_changed = repo_pkg.latest_version() != installed.version;
                        let release_changed = repo_pkg.release > installed.release;
                        let hash_changed = repo_pkg.package_hash != installed.package_hash;

                        if version_changed || release_changed || (!ignore_check && hash_changed) {
                            needs_action = true;
                            let update_note =
                                if !version_changed && !release_changed && hash_changed {
                                    t!("upgrade_reason_hash").to_string()
                                } else {
                                    "".to_string()
                                };
                            reason = t!(
                                "upgrade_reason_available",
                                old = installed.version,
                                new = repo_pkg.latest_version(),
                                note = update_note
                            )
                            .to_string();
                        }
                    }
                }

                if !needs_action && !no_integrity {
                    let is_corrupted =
                        installed
                            .installed_files
                            .par_iter()
                            .any(|(file_path, _meta)| {
                                let full_path =
                                    self.dest_dir().join(file_path.trim_start_matches('/'));
                                !full_path.exists()
                            });

                    if is_corrupted {
                        needs_action = true;
                        reason = t!("upgrade_reason_integrity").to_string();
                    }
                }

                if needs_action {
                    Some((installed.name, reason))
                } else {
                    None
                }
            })
            .collect();

        for (name, reason) in upgrade_results {
            println!("📦 {:<25} : {}", name, reason);
            packages_to_process.push(name);
        }

        if packages_to_process.is_empty() {
            if package_names.is_empty() {
                println!("{}", t!("upgrade_system_uptodate"));
            } else {
                for name in package_names {
                    println!("{}", t!("upgrade_package_uptodate", package = name));
                }
            }
        } else if check_only {
            println!(
                "{}",
                t!("upgrade_need_detect", count = packages_to_process.len())
            );
            println!("{}", t!("upgrade_check_only_info"));
        } else {
            println!(
                "{}",
                t!("upgrade_start_info", count = packages_to_process.len())
            );
            self.perform_install(
                packages_to_process.clone(),
                trace_id,
                true,
                yes_all,
                limit_kb,
                auth,
                download_only,
                ignore_check,
                ignore_comar,
                ignore_file_conflict,
                ignore_package_conflict,
                true,
                ignore_dependency,
                reporter,
            )?;

            // Güncellenen paketlerin history'sinde Requires/Type kontrolü
            let mut restart_packages: Vec<&str> = Vec::new();
            let mut security_packages: Vec<&str> = Vec::new();
            for name in &packages_to_process {
                if let Some(repo_pkg) = available_map.get(name) {
                    if let Some(update) = repo_pkg.history.updates.first() {
                        if update.requires.as_ref().map_or(false, |r| r.actions.iter().any(|a| a.action == "systemRestart")) {
                            restart_packages.push(name);
                        }
                        if update.type_.as_deref() == Some("security") {
                            security_packages.push(name);
                        }
                    }
                }
            }
            if !restart_packages.is_empty() {
                println!("\n{}", t!("upgrade_notify_restart_title"));
                let names = restart_packages.join("\n");
                println!("{}", t!("upgrade_notify_restart_body", packages = names));
                println!("{}", t!("upgrade_notify_restart_hint"));
            }
            if !security_packages.is_empty() {
                println!("\n{}", t!("upgrade_notify_security_title"));
                let names = security_packages.join("\n");
                println!("{}", t!("upgrade_notify_security_body", packages = names));
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn perform_emerge_up(
        &self,
        package_names: Vec<String>,
        trace_id: u64,
        yes_all: bool,
        limit_kb: Option<usize>,
        auth: Option<(String, String)>,
        download_only: bool,
        ignore_check: bool,
        ignore_comar: bool,
        ignore_file_conflict: bool,
        ignore_package_conflict: bool,
        reporter: Option<&dyn crate::progress::ProgressReporter>,
    ) -> LuppoResult<()> {
        let repo_manager = Repository::new(self.db.clone(), self.config.clone());
        let order = self.calculate_install_order(&package_names)?;
        for pkg_name in order {
            if let Ok(Some(repo_pkg)) = self.db.get_available_package(&pkg_name) {
                let path = repo_manager.fetch_package(&repo_pkg, None, limit_kb, auth.clone(), reporter)?;
                self.perform_install(
                    vec![path.to_string_lossy().to_string()],
                    trace_id,
                    false,
                    yes_all,
                    limit_kb,
                    auth.clone(),
                    download_only,
                    ignore_check,
                    ignore_comar,
                    ignore_file_conflict,
                    ignore_package_conflict,
                    false,
                    false,
                    reporter,
                )?;
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn perform_emerge_up_all(
        &self,
        trace_id: u64,
        yes_all: bool,
        limit_kb: Option<usize>,
        auth: Option<(String, String)>,
        download_only: bool,
        ignore_check: bool,
        ignore_comar: bool,
        ignore_file_conflict: bool,
        ignore_package_conflict: bool,
        reporter: Option<&dyn crate::progress::ProgressReporter>,
    ) -> LuppoResult<()> {
        let repo_manager = Repository::new(self.db.clone(), self.config.clone());
        repo_manager.perform_update(trace_id)?;

        let installed = self.db.list_installed_packages()?;
        let mut emerge_list = Vec::new();

        println!("{}", t!("emerge_searching"));
        for pkg in installed {
            if let Ok(Some(available)) = self.db.get_available_package(&pkg.name) {
                if available.latest_version() != pkg.version || available.release > pkg.release {
                    emerge_list.push(pkg.name);
                }
            }
        }
        if emerge_list.is_empty() {
            println!("{}", t!("emerge_no_packages"));
            return Ok(());
        }
        println!("{}", t!("emerge_count", count = emerge_list.len()));
        self.perform_emerge_up(
            emerge_list,
            trace_id,
            yes_all,
            limit_kb,
            auth,
            download_only,
            ignore_check,
            ignore_comar,
            ignore_file_conflict,
            ignore_package_conflict,
            reporter,
        )
    }

    /// Yapılandırması yarım kalmış (configured: false) paketleri tekrar yapılandırır.
    pub fn perform_configure_pending(&self) -> LuppoResult<()> {
        let installed = self.db.list_installed_packages()?;
        let pending: Vec<_> = installed.into_iter().filter(|p| !p.configured).collect();

        if pending.is_empty() {
            println!("{}", t!("config_pending_none"));
            return Ok(());
        }

        for mut pkg in pending {
            println!(
                "{}",
                t!(
                    "config_pending_starting",
                    package = pkg.name,
                    version = pkg.version
                )
            );
            if let Ok(Some(repo_pkg)) = self.db.get_available_package(&pkg.name) {
                let script_to_run = repo_pkg.post_install.as_ref();
                if let Some(script) = script_to_run {
                    match self.run_post_install(
                        &pkg.name,
                        script,
                        self.dest_dir().to_str().unwrap_or("/"),
                    ) {
                        Ok(_) => {
                            pkg.configured = true;
                            self.db.install_package(&pkg)?;
                            println!("{}", t!("config_pending_success", package = pkg.name));
                        }
                        Err(e) => eprintln!(
                            "{}",
                            t!(
                                "config_pending_error",
                                package = pkg.name,
                                error = format!("{:?}", e)
                            )
                        ),
                    }
                } else {
                    pkg.configured = true;
                    self.db.install_package(&pkg)?;
                    println!("{}", t!("config_pending_no_script", package = pkg.name));
                }
            } else {
                eprintln!("{}", t!("config_pending_not_found", package = pkg.name));
            }
        }
        Ok(())
    }

    /// Sistemdeki sahipsiz (orphaned) paketleri tespit eder ve kaldırır.
    pub fn perform_remove_orphaned(&self, trace_id: u64) -> LuppoResult<()> {
        let orphans = self.find_orphaned_packages()?;
        if !orphans.is_empty() {
            println!(
                "{}",
                t!("orphans_cleaning", packages = format!("{:?}", orphans))
            );
            self.remove_packages(&orphans, trace_id, false)?;
        } else {
            println!("{}", t!("orphans_none"));
        }
        Ok(())
    }

    /// Paketin GPG imzasını doğrular (sequoia-openpgp ile pure Rust).
    /// Paketin GPG imzasını doğrular (gpgv kullanarak).
    /// Not: Bu fonksiyon sistemde 'gpgv' komutunun ve Luppo anahtarlarının yüklü olmasını bekler.
    fn verify_package_signature(&self, path: &std::path::Path) -> LuppoResult<bool> {
        let sig_path = path.with_extension("sig");

        if !sig_path.exists() {
            println!(
                "{}",
                t!(
                    "gpg_no_sig",
                    name = path.file_name().unwrap_or_default().to_string_lossy()
                )
            );
            return Ok(false);
        }

        println!("{}", t!("gpg_verifying"));

        let status = std::process::Command::new("gpgv")
            .arg("--keyring")
            .arg("/etc/luppo/trusted-keys.gpg")
            .arg(&sig_path)
            .arg(path)
            .status()
            .map_err(|e| {
                LuppoError::RuntimeError(format!("GPG doğrulaması başlatılamadı: {}", e))
            })?;

        if status.success() {
            println!("{}", t!("gpg_success"));
            Ok(true)
        } else {
            Err(LuppoError::RuntimeError(
                t!("gpg_error_critical", path = path.display()).to_string(),
            ))
        }
    }

    pub fn perform_rollback(&self, target_trace_id: u64, current_trace_id: u64) -> LuppoResult<()> {
        println!("{}", t!("rollback_title", id = target_trace_id));

        let actions_to_rollback = self.db.get_actions_for_rollback(target_trace_id)?;

        if actions_to_rollback.is_empty() {
            println!("{}", t!("rollback_none"));
            return Ok(());
        }

        for action in actions_to_rollback {
            println!(
                "{}",
                t!(
                    "rollback_action",
                    op = action.operation,
                    id = action.trace_id
                )
            );

            match action.operation.as_str() {
                "install" => {
                    let pkg_name = action.details.split_whitespace().next().unwrap_or("");
                    if !pkg_name.is_empty() {
                        println!("{}", t!("installer_removing_files", package = pkg_name));
                        self.perform_remove(
                            pkg_name.to_string(),
                            current_trace_id,
                            true,
                            false,
                            false,
                            true,
                            None,
                        )?;
                    }
                }
                "remove" => {
                    let pkg_name = action.details.split_whitespace().next().unwrap_or("");
                    if !pkg_name.is_empty() {
                        println!("{}", t!("rollback_reinstalling", package = pkg_name));
                        self.perform_install(
                            vec![pkg_name.to_string()],
                            current_trace_id,
                            false,
                            true,
                            None,
                            None,
                            false,
                            false,
                            false,
                            false,
                            false,
                            false,
                            false,
                            None,
                        )?;
                    }
                }
                _ => {}
            }
        }
        println!("{}", t!("rollback_success"));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::database::LuppoDatabase;
    use crate::package::{InstalledPackage, Package};
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_installer_safety_switch() {
        let dir = tempdir().expect("Failed to create tempdir");
        let db = LuppoDatabase::open(dir.path().to_path_buf()).expect("Failed to open DB");
        let config = Config::load(None);
        let installer = Installer::new(db.clone(), config);

        // Create a mock package part of system.base component
        let pkg_json = r#"{
            "Name": "glibc",
            "Summary": [],
            "Description": [],
            "History": { "Update": [] },
            "Architecture": "x86_64",
            "PackageURI": "",
            "PartOf": "system.base",
            "InstalledSize": 0
        }"#;
        let pkg: Package = serde_json::from_str(pkg_json).unwrap();
        db.save_package(&pkg).unwrap();

        // Mark it as installed
        let inst_pkg = InstalledPackage {
            name: "glibc".to_string(),
            version: "2.33".to_string(),
            description: "GNU C Library".to_string(),
            install_date: "2026-05-17".to_string(),
            installed_files: HashMap::new(),
            total_size: 0,
            package_hash: "mockhash".to_string(),
            release: 1,
            distribution_release: "1.0".to_string(),
            licenses: vec![],
            provides: vec![],
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
            signature_verified: true,
        };
        db.install_package(&inst_pkg).unwrap();

        // Trying to perform_remove on glibc without ignore_safety should fail
        let result = installer.perform_remove("glibc".to_string(), 1, true, true, false, false, None);
        assert!(
            result.is_err(),
            "Safety switch should prevent removing system.base packages"
        );

        if let Err(LuppoError::RuntimeError(err_msg)) = result {
            assert!(err_msg.contains("glibc"));
        } else {
            panic!("Expected RuntimeError!");
        }

        // Bypassing safety check with ignore_safety = true should allow it
        let result_bypass =
            installer.perform_remove("glibc".to_string(), 1, true, true, false, true, None);
        assert!(
            result_bypass.is_ok(),
            "Safety switch should be bypassed with ignore_safety = true"
        );
        assert!(
            !db.is_package_installed("glibc").unwrap(),
            "Package should be successfully removed"
        );
    }
}
