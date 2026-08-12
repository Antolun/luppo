use crate::config::Config;
use crate::package::{Component, FileData, InstalledPackage, Package};
use crate::repo::RepositoryEntry;
use crate::PisiError;
use chrono::prelude::*;
use pisi_spec::models::{HistoryAction, PackageDefinition};
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use sled::{Db, Tree};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fs;
use std::path::{Path, PathBuf};
rust_i18n::i18n!("../locales", fallback = "tr");

type PisiResult<T> = Result<T, PisiError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEntry {
    pub timestamp: DateTime<Utc>,
    pub action_type: String,
    pub details: String,
    pub trace_id: u64,
}

#[derive(Debug, Clone)]
pub struct PisiDatabase {
    db: Db,
    path: PathBuf,
    packages: Tree,
    local_packages: Tree,
    installed: Tree,
    files: Tree,
    history: Tree,
    repositories: Tree,
    config: Tree,
    components: Tree,
    sources: Tree,
    deps_cache: Tree,
}

impl Default for PisiDatabase {
    fn default() -> Self {
        let config = Config::load(None);
        PisiDatabase::open(config.directories.lib_dir.join("db"))
            .unwrap_or_else(|_| panic!("{}", t!("db_err_init").to_string()))
    }
}

impl PisiDatabase {
    const TRACE_ID_KEY: &[u8] = b"next_trace_id";

    pub fn open(path: PathBuf) -> PisiResult<Self> {
        let db = sled::open(&path)?;
        let packages = db.open_tree("repo_packages")?;
        let local_packages = db.open_tree("local_packages")?;
        let installed = db.open_tree("installed_packages")?;
        let files = db.open_tree("files")?;
        let history = db.open_tree("history")?;
        let repositories = db.open_tree("repositories")?;
        let config = db.open_tree("config")?;
        let components = db.open_tree("components")?;
        let sources = db.open_tree("source_packages")?;
        let deps_cache = db.open_tree("deps_cache")?;

        Ok(PisiDatabase {
            db,
            path,
            packages,
            local_packages,
            installed,
            files,
            history,
            repositories,
            config,
            components,
            sources,
            deps_cache,
        })
    }

    // --- REPO SOURCES (KAYNAK DEPOLARI) EŞLEŞTİRME ---

    pub fn insert_source(&self, package_name: &str, pspec_url: &str) -> PisiResult<()> {
        self.sources
            .insert(package_name.trim().as_bytes(), pspec_url.trim().as_bytes())?;
        Ok(())
    }

    pub fn get_source(&self, package_name: &str) -> PisiResult<Option<String>> {
        match self.sources.get(package_name.trim().as_bytes())? {
            Some(ivec) => Ok(Some(String::from_utf8_lossy(&ivec).into_owned())),
            None => Ok(None),
        }
    }

    // --- REPO (DEPO) YÖNETİMİ ---

    pub fn insert_repo(&self, name: &str, entry: RepositoryEntry) -> PisiResult<()> {
        let value = bincode::serialize(&entry).map_err(PisiError::BincodeError)?;
        self.repositories.insert(name.as_bytes(), value)?;
        self.repositories.flush()?;
        Ok(())
    }

    pub fn remove_repo(&self, name: &str) -> PisiResult<bool> {
        let removed = self.repositories.remove(name.as_bytes())?;
        self.repositories.flush()?;
        Ok(removed.is_some())
    }

    pub fn list_repos(&self) -> PisiResult<Vec<RepositoryEntry>> {
        let mut repos = Vec::new();
        for item in self.repositories.iter() {
            let (_, value) = item?;
            repos.push(bincode::deserialize(&value).map_err(PisiError::BincodeError)?);
        }
        Ok(repos)
    }

    pub fn get_repo_name_by_package_name(&self, package_name: &str) -> PisiResult<String> {
        if let Some(ivec) = self.packages.get(package_name)? {
            let pkg: Package = bincode::deserialize(&ivec).map_err(PisiError::BincodeError)?;

            Ok(pkg.repo_name.clone())
        } else {
            Err(PisiError::RuntimeError(
                t!("db_err_not_found", name = package_name).to_string(),
            ))
        }
    }

    // --- KURULU PAKET İŞLEMLERİ ---

    pub fn install_package(&self, pkg: &InstalledPackage) -> PisiResult<()> {
        let mut pkg_to_save = pkg.clone();
        pkg_to_save.prune();
        let compressed = Self::compress(&pkg_to_save)?;
        self.installed.insert(pkg.name.as_bytes(), compressed)?;
        self.installed.flush()?;
        Ok(())
    }

    pub fn remove_package(&self, name: &str) -> PisiResult<()> {
        if self.installed.remove(name.as_bytes())?.is_some() {
            self.installed.flush()?;
            Ok(())
        } else {
            Err(PisiError::RuntimeError(
                t!("db_err_not_installed", name = name).to_string(),
            ))
        }
    }

    pub fn get_installed_package(&self, name: &str) -> PisiResult<Option<InstalledPackage>> {
        match self.installed.get(name.as_bytes())? {
            Some(ivec) => Ok(Some(Self::decompress(&ivec)?)),
            None => Ok(None),
        }
    }

    pub fn list_installed_packages(&self) -> PisiResult<Vec<InstalledPackage>> {
        let mut packages = Vec::new();
        for item in self.installed.iter() {
            let (_, val) = item?;
            if let Ok(pkg) = Self::decompress::<InstalledPackage>(&val) {
                packages.push(pkg);
            }
        }
        Ok(packages)
    }

    /// Sadece paket isimlerini döndürür (decompression yapmaz) — çok daha hızlı.
    pub fn list_installed_package_names(&self) -> PisiResult<HashSet<String>> {
        let mut names = HashSet::new();
        for item in self.installed.iter() {
            let (key, _) = item?;
            if let Ok(name) = String::from_utf8(key.to_vec()) {
                names.insert(name);
            }
        }
        Ok(names)
    }

    /// Sadece mevcut (repo) paket isimlerini döndürür (decompression yapmaz).
    pub fn list_available_package_names(&self) -> PisiResult<HashSet<String>> {
        let mut names = HashSet::new();
        for item in self.packages.iter() {
            let (key, _) = item?;
            if let Ok(name) = String::from_utf8(key.to_vec()) {
                names.insert(name);
            }
        }
        Ok(names)
    }

    /// Sadece belirtilen isimlerdeki paketleri yükler (tek iterasyonda, sadece gerekli olanları deserialize eder).
    pub fn get_packages_by_names(&self, names: &HashSet<String>) -> PisiResult<Vec<crate::package::Package>> {
        let mut results = Vec::new();
        for item in self.packages.iter() {
            let (key, value) = item?;
            if let Ok(name) = String::from_utf8(key.to_vec()) {
                if names.contains(&name) {
                    if let Ok(pkg) = Self::decompress::<crate::package::Package>(&value) {
                        results.push(pkg);
                    }
                }
            }
        }
        Ok(results)
    }

    /// Tek seferlik DB iterasyonu ile TÜM paketlerin runtime bağımlılıklarını yükler.
    /// Sadece (name -> runtime_deps) map'i döndürür, full metadata yüklemez.
    pub fn load_all_runtime_deps(&self) -> PisiResult<HashMap<String, Vec<String>>> {
        let mut deps_map = HashMap::new();
        for item in self.packages.iter() {
            let (key, value) = item?;
            if let Ok(name) = String::from_utf8(key.to_vec()) {
                if let Ok(pkg) = Self::decompress::<crate::package::Package>(&value) {
                    let runtime_deps: Vec<String> = pkg
                        .runtime_dependencies
                        .as_ref()
                        .map(|r| r.dependencies.clone())
                        .unwrap_or_default();
                    deps_map.insert(name, runtime_deps);
                }
            }
        }
        Ok(deps_map)
    }

    /// Cache'lenmiş runtime deps'leri getirir veya ilk kez oluşturur.
    /// Cache key: "all_runtime_deps", value: bincode-serialized HashMap<String, Vec<String>>.
    pub fn get_or_build_runtime_deps_cache(&self) -> PisiResult<HashMap<String, Vec<String>>> {
        const CACHE_KEY: &[u8] = b"all_runtime_deps";
        const CACHE_VERSION: &[u8] = b"cache_version";
        const CURRENT_VERSION: u32 = 1;

        // Cache versiyonu kontrol et
        if let Ok(Some(version_bytes)) = self.deps_cache.get(CACHE_VERSION) {
            if let Ok(version) = Self::decompress::<u32>(&version_bytes) {
                if version == CURRENT_VERSION {
                    // Cache geçerli, deps_map'i oku
                    if let Ok(Some(deps_bytes)) = self.deps_cache.get(CACHE_KEY) {
                        if let Ok(deps_map) = Self::decompress::<HashMap<String, Vec<String>>>(&deps_bytes) {
                            return Ok(deps_map);
                        }
                    }
                }
            }
        }

        // Cache yok veya eski, yeniden oluştur
        let deps_map = self.load_all_runtime_deps()?;

        // Cache'e yaz
        let compressed = Self::compress(&deps_map)?;
        self.deps_cache.insert(CACHE_KEY, compressed)?;
        self.deps_cache.insert(CACHE_VERSION, Self::compress(&CURRENT_VERSION)?)?;
        self.deps_cache.flush()?;

        Ok(deps_map)
    }

    /// Repo değiştiğinde cache'i geçersiz kılar.
    pub fn invalidate_runtime_deps_cache(&self) -> PisiResult<()> {
        self.deps_cache.remove(b"all_runtime_deps")?;
        self.deps_cache.remove(b"cache_version")?;
        self.deps_cache.flush()?;
        Ok(())
    }

    pub fn is_package_installed(&self, name: &str) -> PisiResult<bool> {
        Ok(self.installed.get(name.as_bytes())?.is_some())
    }

    pub fn clear_installed_packages(&self) -> PisiResult<()> {
        self.installed.clear().map_err(PisiError::DatabaseError)?;
        self.files.clear().map_err(PisiError::DatabaseError)?;
        self.installed.flush()?;
        self.files.flush()?;
        Ok(())
    }

    pub fn clear_repo_packages(&self) -> PisiResult<()> {
        self.packages.clear().map_err(PisiError::DatabaseError)?;
        self.packages.flush().map_err(PisiError::DatabaseError)?;
        Ok(())
    }

    // --- MEVCUT (DEPO) PAKETLERİ ---

    pub fn save_package(&self, pkg: &Package) -> PisiResult<()> {
        let mut pkg_to_save = pkg.clone();
        pkg_to_save.prune();
        let compressed = Self::compress(&pkg_to_save)?;
        self.packages
            .insert(pkg.name.trim().as_bytes(), compressed)?;
        Ok(())
    }

    pub fn get_available_package(&self, name: &str) -> PisiResult<Option<Package>> {
        match self.packages.get(name.trim().as_bytes())? {
            Some(ivec) => Ok(Some(Self::decompress(&ivec)?)),
            None => Ok(None),
        }
    }

    // query.rs ve resolver.rs için kritik:
    pub fn get_package(&self, name: &str) -> PisiResult<Option<Package>> {
        self.get_available_package(name)
    }

    pub fn list_available_packages(&self) -> PisiResult<Vec<Package>> {
        let mut results = Vec::new();
        for item in self.packages.iter() {
            let (_, value) = item?;
            if let Ok(pkg) = Self::decompress::<Package>(&value) {
                results.push(pkg);
            }
        }
        Ok(results)
    }

    pub fn insert_package(&self, pkg: &PackageDefinition) -> PisiResult<()> {
        let compressed = Self::compress(pkg)?;
        self.local_packages.insert(pkg.name.as_bytes(), compressed)?;
        Ok(())
    }

    pub fn search_package(&self, query: &str) -> PisiResult<Vec<Package>> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        for item in self.packages.iter() {
            let (k, value) = item?;
            match Self::decompress::<Package>(&value) {
                Ok(pkg) => {
                    if pkg.name.to_lowercase().contains(&query_lower) {
                        results.push(pkg);
                    }
                }
                Err(e) => {
                    println!(
                        "Warning: Failed to decompress a package ({}) from the database. Error: {:?}",
                        String::from_utf8_lossy(&k),
                        e
                    );
                }
            }
        }
        Ok(results)
    }

    pub fn get_all_available_packages(&self) -> PisiResult<Vec<Package>> {
        let mut packages = Vec::new();
        for item in self.packages.iter() {
            let (_key, value) = item.map_err(PisiError::DatabaseError)?;
            if let Ok(pkg) = Self::decompress::<Package>(&value) {
                packages.push(pkg);
            }
        }
        Ok(packages)
    }

    // --- BİLEŞEN (COMPONENT) YÖNETİMİ ---

    pub fn save_component(&self, comp: &Component) -> PisiResult<()> {
        let compressed = Self::compress(comp)?;
        self.components
            .insert(comp.name.trim().as_bytes(), compressed)?;
        Ok(())
    }

    pub fn get_component(&self, name: &str) -> PisiResult<Option<Component>> {
        match self.components.get(name.trim().as_bytes())? {
            Some(value) => Ok(Some(Self::decompress(&value)?)),
            None => Ok(None),
        }
    }

    pub fn list_components(&self) -> PisiResult<Vec<Component>> {
        let mut results = Vec::new();
        for item in self.components.iter() {
            let (_, value) = item?;
            if let Ok(comp) = Self::decompress::<Component>(&value) {
                results.push(comp);
            }
        }
        Ok(results)
    }

    // --- DOSYA SİSTEMİ EŞLEŞTİRME ---

    pub fn register_file(&self, file_path: &str, pkg_name: &str) -> PisiResult<()> {
        let normalized_path = if file_path.starts_with('/') {
            file_path.to_string()
        } else {
            format!("/{}", file_path)
        };
        self.files
            .insert(normalized_path.as_bytes(), pkg_name.as_bytes())?;
        Ok(())
    }

    pub fn find_package_by_file(&self, file_path: &str) -> PisiResult<Option<String>> {
        match self.files.get(file_path.as_bytes())? {
            Some(ivec) => Ok(Some(String::from_utf8_lossy(&ivec).into_owned())),
            None => Ok(None),
        }
    }

    pub fn remove_file_entry(&self, file_path: &str) -> PisiResult<()> {
        self.files.remove(file_path.as_bytes())?;
        Ok(())
    }

    pub fn search_file(&self, query: &str) -> PisiResult<Vec<(String, String)>> {
        let mut results = Vec::new();

        // Kurulu paketleri tara
        for item in self.installed.iter() {
            let (_key, value) = item.map_err(PisiError::DatabaseError)?;
            let pkg: InstalledPackage = Self::decompress(&value)?;

            // Pakete ait kurulu dosyaları tara (HashSet<String>)
            for path in pkg.installed_files.keys() {
                if path.contains(query) {
                    results.push((pkg.name.clone(), path.clone()));
                }
            }
        }
        Ok(results)
    }

    pub fn get_files_for_package(&self, name: &str) -> PisiResult<Option<Vec<FileData>>> {
        if let Some(data) = self.files.get(name).map_err(PisiError::DatabaseError)? {
            let files: Vec<FileData> = bincode::deserialize(&data)
                .map_err(|e| PisiError::DatabaseError(sled::Error::Unsupported(e.to_string())))?;
            Ok(Some(files))
        } else {
            Ok(None)
        }
    }

    // --- GEÇMİŞ (HISTORY) ---

    pub fn record_action(&self, action: HistoryAction) -> PisiResult<()> {
        let key = format!("{}-{}", action.trace_id, action.timestamp);
        let compressed = Self::compress(&action)?;
        self.history.insert(key.as_bytes(), compressed)?;
        self.history.flush()?;
        Ok(())
    }

    pub fn list_history(&self, filter_trace_id: Option<u64>) -> PisiResult<Vec<HistoryAction>> {
        let mut entries = Vec::new();
        let requested_id = filter_trace_id.unwrap_or(0);
        for item in self.history.iter().rev() {
            let (_, val_bytes) = item?;
            // Not: Kaydederken HistoryAction kullandığın için okurken de onu kullanmalısın
            if let Ok(entry) = Self::decompress::<HistoryAction>(&val_bytes) {
                if requested_id == 0 || entry.trace_id == requested_id {
                    entries.push(entry);
                }
            }
        }
        Ok(entries)
    }

    pub fn get_history(&self) -> PisiResult<Vec<HistoryAction>> {
        let mut actions = Vec::new();
        for item in self.history.iter() {
            let (_, value) = item?;
            if let Ok(action) = Self::decompress::<HistoryAction>(&value) {
                actions.push(action);
            }
        }
        actions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(actions)
    }

    pub fn delete_history_entry(&self, trace_id: u64) -> PisiResult<()> {
        let mut keys_to_delete = Vec::new();
        for item in self.history.iter() {
            let (key, val) = item?;
            if let Ok(entry) = Self::decompress::<HistoryAction>(&val) {
                if entry.trace_id == trace_id {
                    keys_to_delete.push(key);
                }
            }
        }
        for key in keys_to_delete {
            self.history.remove(key)?;
        }
        self.history.flush()?;
        Ok(())
    }

    pub fn get_actions_for_rollback(&self, start_trace_id: u64) -> PisiResult<Vec<HistoryAction>> {
        let entries = self.list_history(None)?; // list_history artık Vec<HistoryAction> dönüyor
        Ok(entries
            .into_iter()
            .filter(|e| e.trace_id > start_trace_id)
            .collect()) // Artık tipler uyuşuyor (HistoryAction -> HistoryAction)
    }

    pub fn delete_package_only(&self, name: &str) -> PisiResult<()> {
        self.installed.remove(name.as_bytes())?;
        self.installed.flush()?;
        Ok(())
    }

    // --- CONFIG VE TRACE ---

    pub fn get_next_trace_id(&self) -> PisiResult<u64> {
        match self.config.get(Self::TRACE_ID_KEY)? {
            Some(ivec) => Ok(u64::from_be_bytes(
                ivec.as_ref().try_into().unwrap_or([0; 8]),
            )),
            None => Ok(1),
        }
    }

    pub fn increment_trace_id(&self, current_id: u64) -> PisiResult<()> {
        let next_id = current_id + 1;
        self.config
            .insert(Self::TRACE_ID_KEY, &next_id.to_be_bytes())?;
        self.config.flush()?;
        Ok(())
    }

    pub fn flush(&self) -> PisiResult<usize> {
        self.db.flush().map_err(PisiError::DatabaseError)
    }

    fn compress<T: Serialize>(data: &T) -> PisiResult<Vec<u8>> {
        let encoded = bincode::serialize(data).map_err(PisiError::BincodeError)?;
        zstd::encode_all(&encoded[..], 3).map_err(|e| PisiError::RuntimeError(e.to_string()))
    }

    fn decompress<T: for<'a> Deserialize<'a>>(data: &[u8]) -> PisiResult<T> {
        let decompressed =
            zstd::decode_all(data).map_err(|e| PisiError::RuntimeError(e.to_string()))?;
        bincode::deserialize(&decompressed).map_err(PisiError::BincodeError)
    }

    // --- BACKUP / RESTORE ---

    /// Veritabanını belirtilen dizine yedekler (dosya sistemi seviyesinde kopyalama)
    pub fn backup_to_dir(&self, backup_dir: &PathBuf) -> PisiResult<()> {
        if !backup_dir.exists() {
            fs::create_dir_all(backup_dir)?;
        }

        // Veritabanını flush et
        self.db.flush()?;

        // Tüm veritabanı dizinini kopyala (sled dosya tabanlıdır)
        Self::copy_dir_all(&self.path, backup_dir)?;

        // Trace ID'yi de yedekle
        let trace_id = self.get_next_trace_id()?;
        let trace_path = backup_dir.join("trace_id.json");
        let trace_json = serde_json::to_string(&trace_id)
            .map_err(|e| PisiError::RuntimeError(e.to_string()))?;
        fs::write(&trace_path, trace_json)?;

        println!("{}", t!("db_backup_success", path = backup_dir.display()));
        Ok(())
    }

    /// Veritabanını belirtilen dizinden geri yükler (dosya sistemi seviyesinde kopyalama)
    pub fn restore_from_dir(&self, backup_dir: &PathBuf) -> PisiResult<()> {
        if !backup_dir.exists() {
            return Err(PisiError::RuntimeError(
                t!("db_restore_not_found", path = backup_dir.display()).to_string(),
            ));
        }

        // Mevcut veritabanını kapat ve temizle
        // Not: Gerçek restorasyon için işlem durdurulmalı veya yeni DB açılmalı
        // Bu implementasyon mevcut DB'yi kullanarak kopyalar

        let db_path = &self.path;

        // Hedef dizini temizle
        if db_path.exists() {
            fs::remove_dir_all(db_path)?;
        }

        // Yedekten geri yükle
        Self::copy_dir_all(backup_dir, db_path)?;

        // Trace ID'yi geri yükle
        let trace_path = backup_dir.join("trace_id.json");
        if trace_path.exists() {
            let content = fs::read_to_string(&trace_path)?;
            let trace_id: u64 = serde_json::from_str(&content)
                .map_err(|e| PisiError::RuntimeError(e.to_string()))?;
            self.db.insert(PisiDatabase::TRACE_ID_KEY, &trace_id.to_be_bytes())?;
        }

        println!("{}", t!("db_restore_success", path = backup_dir.display()));
        Ok(())
    }

    /// Dizin kopyalama yardımcı fonksiyonu
    fn copy_dir_all(src: &Path, dst: &Path) -> PisiResult<()> {
        if !dst.exists() {
            fs::create_dir_all(dst)?;
        }

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                Self::copy_dir_all(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    /// Crash recovery: sled otomatik WAL recovery yapar, ama manuel kontrol
    pub fn verify_integrity(&self) -> PisiResult<()> {
        let counts: Vec<(&str, usize)> = vec![
            ("repo_packages", self.packages.len()),
            ("installed_packages", self.installed.len()),
            ("files", self.files.len()),
            ("history", self.history.len()),
            ("repositories", self.repositories.len()),
            ("config", self.config.len()),
            ("components", self.components.len()),
            ("source_packages", self.sources.len()),
        ];

        for (name, count) in &counts {
            println!("{}", t!("db_integrity_check", name = name, count = count));
        }
        Ok(())
    }
}

impl Drop for PisiDatabase {
    fn drop(&mut self) {
        let _ = self.db.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::FileMetadata;
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn setup_test_db() -> (PisiDatabase, tempfile::TempDir) {
        let dir = tempdir().expect("Geçici dizin oluşturulamadı");
        let db = PisiDatabase::open(dir.path().to_path_buf()).expect("Test veritabanı açılamadı");
        (db, dir)
    }

    #[test]
    fn test_repo_management() {
        let (db, _dir) = setup_test_db();
        let repo = RepositoryEntry {
            name: "test-repo".to_string(),
            url: "https://repo.pisilinux.org".to_string(),
            mirrors: Vec::new(),
            enabled: true,
        };

        db.insert_repo("test-repo", repo.clone()).unwrap();
        let repos = db.list_repos().unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "test-repo");

        let removed = db.remove_repo("test-repo").unwrap();
        assert!(removed);
        assert_eq!(db.list_repos().unwrap().len(), 0);
    }

    #[test]
    fn test_package_install_and_get() {
        let (db, _dir) = setup_test_db();
        let mut files = HashMap::new();
        files.insert(
            "/usr/bin/hello".to_string(),
            FileMetadata {
                mode: 0o755,
                uid: 0,
                gid: 0,
                size: 1024,
            },
        );

        let pkg = InstalledPackage {
            name: "hello".to_string(),
            version: "1.0".to_string(),
            description: "A test package".to_string(),
            install_date: "2023-10-27".to_string(),
            installed_files: files,
            total_size: 1024,
            package_hash: "abc123hash".to_string(),
            release: 1,
            distribution_release: "2.0".to_string(),
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
            signature_verified: true,
        };

        db.install_package(&pkg).unwrap();
        assert!(db.is_package_installed("hello").unwrap());

        let retrieved = db.get_installed_package("hello").unwrap().unwrap();
        assert_eq!(retrieved.version, "1.0");
        assert!(retrieved.signature_verified);
    }

    #[test]
    fn test_file_ownership_registry() {
        let (db, _dir) = setup_test_db();
        db.register_file("/etc/pisi.conf", "pisi").unwrap();

        let owner = db.find_package_by_file("/etc/pisi.conf").unwrap();
        assert_eq!(owner, Some("pisi".to_string()));

        db.remove_file_entry("/etc/pisi.conf").unwrap();
        assert!(db.find_package_by_file("/etc/pisi.conf").unwrap().is_none());
    }

    #[test]
    fn test_trace_id_flow() {
        let (db, _dir) = setup_test_db();
        let first_id = db.get_next_trace_id().unwrap();
        assert_eq!(first_id, 1);

        db.increment_trace_id(first_id).unwrap();
        let second_id = db.get_next_trace_id().unwrap();
        assert_eq!(second_id, 2);
    }
}
