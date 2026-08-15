use crate::database::LuppoDatabase;
use crate::package::{
    Component, ComponentsRoot, FileMetadata, FilesXmlRoot, InstalledPackage, Package, LuppoIndex,
    LuppoRoot,
};
use crate::resolver::{PackageResolver, LuppoRepo};
use crate::LuppoError;
use chrono::Local;
use indicatif::{ProgressBar, ProgressStyle};
use luppo_spec::models::HistoryAction;
use rust_i18n::t;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use xz2::read::XzDecoder; // ✅ XZ desteği
rust_i18n::i18n!("../locales", fallback = "tr");
// Luppo paket depo tanımı
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryEntry {
    pub name: String,
    pub url: String,
    pub mirrors: Vec<String>, // Alternatif yansıma URL'leri
    pub enabled: bool,
}

type LuppoResult<T> = Result<T, LuppoError>;

pub struct Repository {
    db: LuppoDatabase,
    config: crate::config::Config,
}

impl Repository {
    pub fn new(db: LuppoDatabase, config: crate::config::Config) -> Self {
        Repository { db, config }
    }

    fn get_client(&self) -> LuppoResult<reqwest::blocking::Client> {
        let mut builder = reqwest::blocking::Client::builder().danger_accept_invalid_certs(true);

        if let Some(ref proxy) = self.config.general.http_proxy {
            builder = builder.proxy(
                reqwest::Proxy::http(proxy).map_err(|e| LuppoError::RuntimeError(e.to_string()))?,
            );
        }
        if let Some(ref proxy) = self.config.general.https_proxy {
            builder = builder.proxy(
                reqwest::Proxy::https(proxy).map_err(|e| LuppoError::RuntimeError(e.to_string()))?,
            );
        }

        builder
            .build()
            .map_err(|e| LuppoError::RuntimeError(e.to_string()))
    }

    // --- TEMEL DEPO YÖNETİMİ ---

    pub fn add_repo(&self, name: &str, url: &str) -> LuppoResult<()> {
        let entry = RepositoryEntry {
            name: name.to_string(),
            url: url.to_string(),
            mirrors: Vec::new(),
            enabled: true,
        };
        self.db.insert_repo(name, entry)?;
        // println!("✅ Depo '{}' eklendi: {}", name, url);
        Ok(())
    }

    pub fn remove_repo(&self, name: &str) -> LuppoResult<()> {
        if !self.db.remove_repo(name)? {
            return Err(LuppoError::RuntimeError(format!(
                "{}: {}",
                t!("error_command_not_found", command = ""),
                name
            )));
        }
        Ok(())
    }

    pub fn list_repos(&self) -> LuppoResult<Vec<RepositoryEntry>> {
        self.db.list_repos()
    }

    /// Deponun durumunu (aktif/pasif) günceller.
    pub fn set_repo_status(&self, name: &str, enabled: bool) -> LuppoResult<()> {
        let repos = self.db.list_repos()?;
        if let Some(repo) = repos.iter().find(|r| r.name == name) {
            let mut updated_repo = repo.clone();
            updated_repo.enabled = enabled;
            self.db.insert_repo(name, updated_repo)?;
            let status_str = if enabled {
                t!("repo_set_enabled")
            } else {
                t!("repo_set_disabled")
            };
            println!(
                "{}",
                t!("repo_set_status", name = name, status = status_str)
            );
            Ok(())
        } else {
            Err(LuppoError::RuntimeError(
                t!("repo_error_not_found", name = name).into(),
            ))
        }
    }

    // --- GERÇEK DEPO GÜNCELLEME ---

    /// Tüm etkin depoların paket listelerini indirir ve yerel DB'ye kaydeder.
    fn update_repositories_internal(&self, _trace_id: u64) -> LuppoResult<()> {
        let enabled_repos = self.list_repos()?;
        let active_repos: Vec<_> = enabled_repos.into_iter().filter(|r| r.enabled).collect();
        if active_repos.is_empty() {
            return Err(LuppoError::RuntimeError(
                "Tanımlı veya aktif depo bulunamadı.".to_string(),
            ));
        }

        // Güncelleme öncesinde tüm eski paket verilerini temizle.
        // Bu, veri yapılarındaki değişikliklerde bozuk/eski kayıtların
        // arama sonuçlarını engellemesini önler.
        self.db.clear_repo_packages()?;

        for repo in active_repos {
            println!("{}", t!("repo_syncing", name = repo.name, url = repo.url));

            let mut fetched = None;

            // SIKIŞTIRILMIŞ JSON Denemesi (.json.xz)
            match self.try_fetch_json_xz(&repo.url) {
                Ok(luppo_index) => {
                    println!("{}", t!("repo_json_success"));
                    fetched = Some(luppo_index);
                }
                Err(e) => {
                    eprintln!("  JSON: {}", e);
                }
            }

            // SIKIŞTIRILMIŞ XML Denemesi (.xml.xz) — geriye dönük uyumluluk
            if fetched.is_none() {
                match self.try_fetch_legacy_xml_xz(&repo.url) {
                    Ok(luppo_index) => {
                        println!("{}", t!("repo_xml_success"));
                        fetched = Some(luppo_index);
                    }
                    Err(e) => {
                        eprintln!("  XML: {}", e);
                    }
                }
            }

            // Yerel önbellek (/var/lib/luppo/index/<repo_adı>/) — ağ yoksa veya download başarısızsa
            if fetched.is_none() {
                match self.try_load_local_index(&repo.name) {
                    Ok(luppo_index) => {
                        fetched = Some(luppo_index);
                    }
                    Err(e) => {
                        eprintln!("  Local: {}", e);
                    }
                }
            }

            if let Some(luppo_index) = fetched {
                let total_count = luppo_index.packages.len() + luppo_index.spec_files.len();
                println!("{}", t!("repo_xml_processing", count = total_count));
                self.process_luppo_index(luppo_index, &repo.url, &repo.name)?;

                // Bileşenleri çek (sadece network başarılıysa dene)
                if let Ok(components) = self.try_fetch_components_xml_xz(&repo.url) {
                    println!(
                        "{}",
                        t!("repo_components_processing", count = components.len())
                    );
                    for comp in components {
                        let _ = self.db.save_component(&comp);
                    }
                }
            } else {
                eprintln!(
                    "{}",
                    t!("repo_sync_failed", name = repo.name, url = repo.url)
                );
            }
        }
        // Depo güncelleme bitti, runtime deps cache'ini arka planda oluştur
        let _ = self.db.get_or_build_runtime_deps_cache();
        Ok(())
    }

    /// Paketi önbelleğini (/var/cache/luppo/packages) temizler.
    pub fn perform_delete_cache(&self) -> LuppoResult<()> {
        let cache_dir = &self.config.directories.cached_packages_dir;
        println!("{}", t!("repo_cache_cleaning", path = cache_dir.display()));
        if cache_dir.exists() {
            let entries = fs::read_dir(cache_dir).map_err(LuppoError::IoError)?;
            let mut count = 0;

            for entry in entries {
                let entry = entry.map_err(LuppoError::IoError)?;
                if entry.path().is_file() {
                    fs::remove_file(entry.path()).map_err(LuppoError::IoError)?;
                    count += 1;
                }
            }
            println!("{}", t!("repo_cache_cleaned", count = count));
        } else {
            // println!("ℹ️ Önbellek dizini zaten boş veya mevcut değil.");
        }
        Ok(())
    }

    /// En hızlı yansıma (mirror) seçimi - HEAD isteği ile latency ölçümü
    fn select_best_mirror(&self, mirrors: &[String], auth: &Option<(String, String)>) -> Option<String> {
        if mirrors.is_empty() {
            return None;
        }

        let client = self.get_client().ok()?;
        let mut best_mirror = None;
        let mut best_latency = Duration::from_secs(u64::MAX);

        for mirror in mirrors {
            let test_url = mirror.trim_end_matches('/').to_string() + "/luppo-index.json.xz";
            let mut req = client.head(&test_url);
            if let Some((u, p)) = auth {
                req = req.basic_auth(u, Some(p));
            }
            if let Ok(req) = req.build() {
                let start = Instant::now();
                if let Ok(res) = client.execute(req) {
                    let latency = start.elapsed();
                    if res.status().is_success() && latency < best_latency {
                        best_latency = latency;
                        best_mirror = Some(mirror.clone());
                    }
                }
            }
        }

        if let Some(ref m) = best_mirror {
            println!("{}", t!("repo_mirror_selected", url = m, latency = best_latency.as_millis()));
        }
        best_mirror
    }

    /// Paketi belirtilen dizine indirir. Eğer dest_dir None ise varsayılan cache dizini kullanılır.
    pub fn fetch_package(
        &self,
        pkg: &Package,
        dest_dir: Option<PathBuf>,
        limit_kb: Option<usize>,
        auth: Option<(String, String)>,
        reporter: Option<&dyn crate::progress::ProgressReporter>,
    ) -> LuppoResult<PathBuf> {
        let target_dir =
            dest_dir.unwrap_or_else(|| self.config.directories.cached_packages_dir.clone());

        if !target_dir.exists() {
            fs::create_dir_all(&target_dir).map_err(LuppoError::IoError)?;
        }

        // --- 1. SÜRÜM VE RELEASE BELİRLEME ---
        let version = pkg.latest_version();

        // Önce ana struct'taki release'e bak, 0 ise history'den dene, o da yoksa 1 yap.
        let release_no = if pkg.release != 0 {
            pkg.release.to_string()
        } else {
            pkg.history
                .updates
                .first()
                .map(|u| u.release.to_string())
                .unwrap_or_else(|| "1".to_string())
        };

        // Arşiv adını oluştur (ncurses-6.5-1-p2-x86_64.luppo gibi)
        let archive_name = format!("{}-{}-{}-p2-x86_64.luppo", pkg.name, version, release_no);

        let base_url = if !pkg.repo_url.is_empty() {
            pkg.repo_url.trim_end_matches('/').to_string()
        } else {
            "https://stable2.antolun.com".to_string()
        };

        // --- YANSİMA (MIRROR) SEÇİMİ ---
        // Eğer paket miro'ları varsa en hızlısını seç
        let selected_base_url = if !pkg.mirrors.is_empty() {
            self.select_best_mirror(&pkg.mirrors, &auth)
                .unwrap_or(base_url)
        } else {
            base_url
        };

        // --- 3. URL OLUŞTURMA (Öncelik PackageURI) ---
        // Eğer indekste PackageURI varsa doğrudan onu kullan, yoksa tahmin et (fallback)
        let download_url = if !pkg.package_uri.is_empty() {
            format!("{}/{}", selected_base_url, pkg.package_uri)
        } else {
            let pkg_name_lower = pkg.name.to_lowercase();
            let folder_prefix = if pkg_name_lower.starts_with("lib") && pkg_name_lower.len() > 3 {
                format!("lib{}", pkg_name_lower.chars().nth(3).unwrap())
            } else {
                pkg_name_lower.chars().next().unwrap().to_string()
            };
            format!(
                "{}/{}/{}/{}",
                selected_base_url, folder_prefix, pkg_name_lower, archive_name
            )
        };

        let mut final_url = download_url.clone();
        let mut final_name = archive_name.clone();
        let mut is_delta = false;

        // --- 3.1 DELTA PAKET KONTROLÜ ---
        if let Ok(Some(installed)) = self.db.get_installed_package(&pkg.name) {
            if installed.release > 0 && installed.release < pkg.release {
                let delta_name = format!(
                    "{}-{}-{}.delta.luppo",
                    pkg.name, installed.release, pkg.release
                );
                let delta_url = download_url
                    .replace(&pkg.package_uri, &delta_name)
                    .replace(&archive_name, &delta_name);

                let client = self.get_client()?;

                let mut head_req = client.head(&delta_url);
                if let Some((u, p)) = &auth {
                    head_req = head_req.basic_auth(u, Some(p));
                }

                if let Ok(res) = head_req.send() {
                    if res.status().is_success() {
                        println!("{}", t!("repo_delta_found", name = delta_name));
                        final_url = delta_url;
                        final_name = delta_name;
                        is_delta = true;
                    }
                }
            }
        }

        let dest_path = target_dir.join(&final_name);

        // --- 2. ÖNBELLEK KONTROLÜ ---
        if dest_path.exists() {
            let file = fs::File::open(&dest_path).map_err(LuppoError::IoError)?;
            if zip::ZipArchive::new(file).is_err() {
                println!("{}", t!("repo_cache_corrupted"));
                fs::remove_file(&dest_path).map_err(LuppoError::IoError)?;
            } else {
                println!("{}", t!("repo_cache_hit", name = final_name));
                return Ok(dest_path);
            }
        }

        println!("{}", t!("repo_downloading", url = final_url));

        let client = self.get_client()?;

        // --- 4. HTTP İSTEĞİ VE İLERLEME ÇUBUĞU ---
        let mut req_builder = client.get(&final_url);
        if let Some((u, p)) = &auth {
            req_builder = req_builder.basic_auth(u, Some(p));
        }

        let mut response = req_builder
            .send()
            .map_err(|e| LuppoError::RuntimeError(t!("repo_error_connection", error = e).into()))?;

        if !response.status().is_success() {
            return Err(LuppoError::RuntimeError(format!(
                "{} (URL: {})",
                response.status(),
                final_url
            )));
        }

        let total_size = response.content_length().unwrap_or(0);
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {binary_bytes}/{binary_total_bytes} ({binary_bytes_per_sec}, {eta})")
        .unwrap()
        .progress_chars("#>-"));

        let mut file = fs::File::create(&dest_path).map_err(LuppoError::IoError)?;
        let mut hasher = Sha1::new();
        let mut buffer = [0; 8192];
        let mut downloaded: u64 = 0;
        let start_time = std::time::Instant::now();

        if let Some(r) = reporter {
            r.on_message(&t!("repo_downloading", url = &final_url));
        }

        // --- 5. İNDİRME VE HASH HESAPLAMA ---
        while let Ok(n) = response.read(&mut buffer) {
            if n == 0 {
                break;
            }
            let chunk = &buffer[..n];
            file.write_all(chunk).map_err(LuppoError::IoError)?;

            if let Some(limit) = limit_kb {
                if limit > 0 {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let expected_time = (downloaded + n as u64) as f64 / (limit as f64 * 1024.0);
                    if elapsed < expected_time {
                        std::thread::sleep(std::time::Duration::from_secs_f64(
                            expected_time - elapsed,
                        ));
                    }
                }
            }

            hasher.update(chunk);
            downloaded += n as u64;
            pb.set_position(downloaded);

            if let Some(r) = reporter {
                if total_size > 0 {
                    let pct = downloaded as f32 / total_size as f32;
                    r.on_progress(pct, None, None);
                }
            }
        }

        pb.finish_with_message(t!("repo_download_complete"));
        if let Some(r) = reporter {
            r.on_finish(&t!("repo_download_complete"));
        }

        // --- 6. HASH DOĞRULAMA ---
        let result_hash = format!("{:x}", hasher.finalize());

        // İndeksteki hash boş değilse ve uyuşmuyorsa
        if !is_delta && !pkg.package_hash.is_empty() && result_hash != pkg.package_hash {
            fs::remove_file(&dest_path).ok();
            return Err(LuppoError::RuntimeError(format!(
                "{}",
                t!(
                    "error_hash_mismatch_detail",
                    expected = pkg.package_hash,
                    calculated = result_hash
                )
            )));
        }

        println!("{}", t!("repo_hash_verified", hash = result_hash));

        // --- 7. İMZA DOSYASINI İNDİR (.sig) ---
        let sig_url = final_url.replace(".luppo", ".sig");
        let sig_dest_path = dest_path.with_extension("sig");

        let mut sig_req = client.get(&sig_url);
        if let Some((u, p)) = &auth {
            sig_req = sig_req.basic_auth(u, Some(p));
        }

        if let Ok(mut sig_response) = sig_req.send() {
            if sig_response.status().is_success() {
                if let Ok(mut sig_file) = fs::File::create(&sig_dest_path) {
                    if std::io::copy(&mut sig_response, &mut sig_file).is_ok() {
                        println!("{}", t!("repo_gpg_success"));
                    }
                }
            }
        }

        Ok(dest_path)
    }

    /// JSON.xz indeksi çek ve çöz
    /// Verilen base_url zaten dosya adını içeriyorsa olduğu gibi kullan,
    /// yoksa dosya adını base_url'in sonuna ekleyerek URL oluşturur.
    fn resolve_index_url(base_url: &str, filename: &str) -> String {
        let base = base_url.trim_end_matches('/');
        if base.ends_with(filename) {
            return base.to_string();
        }
        // URL başka bir index dosyasına işaret ediyorsa (örn. luppo-index.json.xz),
        // onu temizleyip yeni dosya adını ekle.
        if let Some(base_pos) = base.rfind("/luppo-index.") {
            format!("{}/{}", &base[..base_pos], filename)
        } else if let Some(base_pos) = base.rfind("/components.") {
            format!("{}/{}", &base[..base_pos], filename)
        } else {
            format!("{}/{}", base, filename)
        }
    }

    fn try_fetch_json_xz(&self, base_url: &str) -> LuppoResult<LuppoIndex> {
        let url = Self::resolve_index_url(base_url, "luppo-index.json.xz");

        println!("{}", t!("repo_url_trying", url = url));

        let client = self.get_client()?;

        let response = client.get(&url).send().map_err(|e| {
            LuppoError::RuntimeError(t!("repo_error_download", error = e).to_string())
        })?;

        if !response.status().is_success() {
            return Err(LuppoError::RuntimeError(
                t!("error_json_index_not_found").to_string(),
            ));
        }

        let total_size = response.content_length().unwrap_or(0);
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {binary_bytes}/{binary_total_bytes} ({binary_bytes_per_sec}, {eta})")
            .unwrap()
            .progress_chars("#>-"));

        let mut xz_data = Vec::new();
        let mut reader = pb.wrap_read(response);
        reader
            .read_to_end(&mut xz_data)
            .map_err(LuppoError::IoError)?;
        pb.finish_and_clear();

        let mut decoder = XzDecoder::new(&xz_data[..]);
        let mut json_content = String::new();
        decoder
            .read_to_string(&mut json_content)
            .map_err(|e| LuppoError::RuntimeError(t!("error_xz_extract", error = e).to_string()))?;

        let luppo_root: LuppoIndex = serde_json::from_str(&json_content)
            .map_err(|e| LuppoError::RuntimeError(t!("repo_error_json", error = e).to_string()))?;

        Ok(luppo_root)
    }

    /// XML.xz indeksi çek ve çöz
    fn try_fetch_legacy_xml_xz(&self, base_url: &str) -> LuppoResult<LuppoIndex> {
        let url = Self::resolve_index_url(base_url, "luppo-index.xml.xz");

        println!("{}", t!("repo_url_trying", url = url));

        let client = self.get_client()?;

        let response = client.get(&url).send().map_err(|e| {
            LuppoError::RuntimeError(t!("repo_error_download", error = e).to_string())
        })?;

        if !response.status().is_success() {
            return Err(LuppoError::RuntimeError(
                t!("error_xml_index_not_found").to_string(),
            ));
        }

        let total_size = response.content_length().unwrap_or(0);
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {binary_bytes}/{binary_total_bytes} ({binary_bytes_per_sec}, {eta})")
            .unwrap()
            .progress_chars("#>-"));

        // 1. XZ Sıkıştırmasını aç
        let mut xz_data = Vec::new();
        let mut reader = pb.wrap_read(response);
        reader
            .read_to_end(&mut xz_data)
            .map_err(LuppoError::IoError)?;
        pb.finish_and_clear();

        let mut decoder = XzDecoder::new(&xz_data[..]);
        let mut xml_content = String::new();
        decoder
            .read_to_string(&mut xml_content)
            .map_err(|e| LuppoError::RuntimeError(t!("error_xz_extract", error = e).to_string()))?;

        // 2. XML'i LuppoIndex yapısına parse et
        // <LUPPO> etiketini karşılamak için LuppoIndex kullanıyoruz
        let luppo_root: LuppoIndex = serde_xml_rs::from_str(&xml_content)
            .map_err(|e| LuppoError::RuntimeError(t!("error_xml_parse", error = e).to_string()))?;

        Ok(luppo_root)
    }

    /// Klasik components.xml.xz Çekme ve Çözme
    fn try_fetch_components_xml_xz(&self, base_url: &str) -> LuppoResult<Vec<Component>> {
        let url = Self::resolve_index_url(base_url, "components.xml.xz");

        let client = self.get_client()?;

        let mut response = client
            .get(&url)
            .send()
            .map_err(|e| LuppoError::RuntimeError(t!("repo_error_download", error = e).into()))?;

        if !response.status().is_success() {
            return Err(LuppoError::RuntimeError(
                t!("error_components_not_found").to_string(),
            ));
        }

        let mut xz_data = Vec::new();
        response
            .read_to_end(&mut xz_data)
            .map_err(LuppoError::IoError)?;

        let mut decoder = XzDecoder::new(&xz_data[..]);
        let mut xml_content = String::new();
        decoder
            .read_to_string(&mut xml_content)
            .map_err(|e| LuppoError::RuntimeError(t!("error_xz_extract", error = e).to_string()))?;

        let luppo_root: ComponentsRoot = serde_xml_rs::from_str(&xml_content)
            .map_err(|e| LuppoError::RuntimeError(t!("error_xml_parse", error = e).to_string()))?;

        Ok(luppo_root.components.items)
    }

    /// Yerel indeks önbelleğinden (.xml.xz) okumayı dener.
    /// Eski luppo, indeks dosyalarını /var/lib/luppo/index/<repo_adı>/luppo-index.xml.xz yolunda önbelleğe alır.
    fn try_load_local_index(&self, repo_name: &str) -> LuppoResult<LuppoIndex> {
        let cache_path = self.config.directories.index_dir.join(repo_name).join("luppo-index.xml.xz");
        if !cache_path.exists() {
            return Err(LuppoError::RuntimeError(format!(
                "Yerel indeks dosyası bulunamadı: {}",
                cache_path.display()
            )));
        }
        let xz_data = fs::read(&cache_path).map_err(LuppoError::IoError)?;
        let mut decoder = XzDecoder::new(&xz_data[..]);
        let mut xml_content = String::new();
        decoder.read_to_string(&mut xml_content)
            .map_err(|e| LuppoError::RuntimeError(t!("error_xz_extract", error = e).to_string()))?;
        let luppo_root: LuppoIndex = serde_xml_rs::from_str(&xml_content)
            .map_err(|e| LuppoError::RuntimeError(t!("error_xml_parse", error = e).to_string()))?;
        println!("{}", t!("repo_xml_local", path = cache_path.display()));
        Ok(luppo_root)
    }

    fn process_index(
        db: &LuppoDatabase,
        packages: Vec<Package>,
        repo_url: &str,
        repo_name: &str,
    ) -> LuppoResult<()> {
        for mut pkg in packages {
            if let Some(latest_update) = pkg.history.updates.first() {
                pkg.version = latest_update.version.clone();
                pkg.release = latest_update.release;
            }

            pkg.repo_name = repo_name.to_string();
            pkg.repo_url = repo_url.to_string();

            db.save_package(&pkg)?;
        }
        // Repo güncellendiğinde deps cache'i geçersiz kıl
        db.invalidate_runtime_deps_cache()?;
        Ok(())
    }

    fn process_luppo_index(
        &self,
        index: LuppoIndex,
        repo_url: &str,
        repo_name: &str,
    ) -> LuppoResult<()> {
        // 1. İkili paketleri kaydet
        Self::process_index(&self.db, index.packages, repo_url, repo_name)?;

        // 2. Kaynak (recipe) paketlerini kaydet
        for spec in index.spec_files {
            if let Some(ref source_uri) = spec.source.source_uri {
                // Uzak lopec.xml URL'si: <repo_url>/<source_uri>
                let full_url = format!("{}/{}", repo_url.trim_end_matches('/'), source_uri);

                // Bu spec altındaki tüm paket isimlerini bu uzak URL ile eşleştir
                let _ = self.db.insert_source(&spec.source.name, &full_url);
                for pkg in spec.packages {
                    let _ = self.db.insert_source(&pkg.name, &full_url);
                }
            }
        }

        Ok(())
    }

    pub fn perform_add_repo(&self, name: &str, url: &str, trace_id: u64) -> LuppoResult<()> {
        println!("{}", t!("repo_syncing", name = name, url = url));

        let mut luppo_index = None;
        let mut components = None;

        // SIKIŞTIRILMIŞ JSON Denemesi (.json.xz)
        match self.try_fetch_json_xz(url) {
            Ok(fetched_luppo_index) => {
                println!("{}", t!("repo_json_success"));
                let total_count = fetched_luppo_index.packages.len() + fetched_luppo_index.spec_files.len();
                println!("{}", t!("repo_xml_processing", count = total_count));
                luppo_index = Some(fetched_luppo_index);
                if let Ok(comps) = self.try_fetch_components_xml_xz(url) {
                    components = Some(comps);
                }
            }
            Err(e) => {
                eprintln!("  JSON: {}", e);
            }
        }

        // SIKIŞTIRILMIŞ XML Denemesi (.xml.xz) — geriye dönük uyumluluk
        if luppo_index.is_none() {
            match self.try_fetch_legacy_xml_xz(url) {
                Ok(fetched_luppo_index) => {
                    println!("{}", t!("repo_xml_success"));
                    let total_count = fetched_luppo_index.packages.len() + fetched_luppo_index.spec_files.len();
                    println!("{}", t!("repo_xml_processing", count = total_count));
                    luppo_index = Some(fetched_luppo_index);
                    if let Ok(comps) = self.try_fetch_components_xml_xz(url) {
                        components = Some(comps);
                    }
                }
                Err(e) => {
                    eprintln!("  XML: {}", e);
                }
            }
        }

        // Yerel önbellek (/var/lib/luppo/index/<repo_adı>/) — ağ yoksa veya download başarısızsa
        if luppo_index.is_none() {
            match self.try_load_local_index(name) {
                Ok(idx) => {
                    luppo_index = Some(idx);
                }
                Err(e) => {
                    eprintln!("  Local: {}", e);
                }
            }
        }

        if luppo_index.is_none() {
            return Err(LuppoError::RuntimeError(format!(
                "Depo index dosyaları indirilemedi veya geçersiz: {}",
                url
            )));
        }

        // Add the repo only after successful index fetch/validation
        self.add_repo(name, url)?;

        // Save index to DB
        if let Some(idx) = luppo_index {
            self.process_luppo_index(idx, url, name)?;
            if let Some(comps) = components {
                println!(
                    "{}",
                    t!("repo_components_processing", count = comps.len())
                );
                for comp in comps {
                    let _ = self.db.save_component(&comp);
                }
            }
        }

        self.db.record_action(HistoryAction {
            trace_id,
            operation: "repo_add".to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            details: t!("history_repo_added", name = name, url = url).to_string(),
        })?;
        Ok(())
    }

    pub fn perform_remove_repo(&self, name: &str, trace_id: u64) -> LuppoResult<()> {
        self.remove_repo(name)?;
        self.db.record_action(HistoryAction {
            trace_id,
            operation: "repo_remove".to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            details: t!("history_repo_removed", name = name).to_string(),
        })?;
        Ok(())
    }

    pub fn perform_set_repo_status(
        &self,
        name: &str,
        enabled: bool,
        trace_id: u64,
    ) -> LuppoResult<()> {
        self.set_repo_status(name, enabled)?;
        let op = if enabled {
            "repo_enable"
        } else {
            "repo_disable"
        };
        let detail = if enabled {
            "Etkinleştirildi"
        } else {
            "Devre dışı bırakıldı"
        };
        self.db.record_action(HistoryAction {
            trace_id,
            operation: op.to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            details: format!("{}: {}", detail, name),
        })?;
        Ok(())
    }

    pub fn perform_list_repos(&self) -> LuppoResult<()> {
        let repos = self.list_repos()?;
        for repo in repos {
            let (name_str, status_str) = if repo.enabled {
                (
                    crate::colorize(&repo.name, "brightgreen"),
                    crate::colorize("[active]", "green"),
                )
            } else {
                (
                    crate::colorize(&repo.name, "brightred"),
                    crate::colorize("[inactive]", "red"),
                )
            };
            println!("🏠 {} {}", name_str, status_str);
            let index_url = if repo.url.ends_with(".xml")
                || repo.url.ends_with(".json")
                || repo.url.ends_with(".xz")
            {
                repo.url.clone()
            } else {
                format!("{}/luppo-index.json.xz", repo.url.trim_end_matches('/'))
            };
            println!("   {}", index_url);
        }
        Ok(())
    }

    pub fn perform_fetch(
        &self,
        package_names: Vec<String>,
        output_dir: PathBuf,
        include_deps: bool,
        limit_kb: Option<usize>,
        auth: Option<(String, String)>,
    ) -> LuppoResult<()> {
        let mut targets = package_names.clone();

        if include_deps {
            println!("{}", t!("repo_fetch_deps"));
            let repo = LuppoRepo::new(self.db.clone());
            let mut resolver = PackageResolver::new(self.db.clone(), repo);
            match resolver.resolve_deps(&package_names) {
                Ok(plan) => {
                    targets = plan.into_iter().map(|p| p.name).collect();
                }
                Err(_) => {
                    println!("{}", t!("repo_fetch_warn_resolver"));
                }
            }
        }

        let all_available = self.db.list_available_packages()?;
        let mut found_any = false;

        for name in targets {
            if let Some(pkg_data) = all_available.iter().find(|p| p.name == name) {
                println!("{}", t!("repo_fetch_downloading", name = name));
                match self.fetch_package(pkg_data, Some(output_dir.clone()), limit_kb, auth.clone(), None)
                {
                    Ok(path) => {
                        println!("{}", t!("repo_fetch_saved", path = path.display()));
                        found_any = true;
                    }
                    Err(e) => println!(
                        "{}",
                        t!("repo_fetch_error", name = name, error = format!("{:?}", e))
                    ),
                }
            } else {
                println!("{}", t!("repo_fetch_error_not_found", name = name));
            }
        }

        if !found_any {
            println!("{}", t!("repo_fetch_no_packages"));
        }
        Ok(())
    }

    pub fn perform_rebuild_db(&self, trace_id: u64) -> LuppoResult<()> {
        println!("{}", t!("repo_rebuild_starting"));
        println!("{}", t!("repo_rebuild_updating"));
        self.update_repositories_internal(trace_id)?;

        println!("{}", t!("repo_rebuild_checking"));

        // Daima /var/lib/luppo/package dizininden kurulu paketleri senkronize et
        self.rebuild_installed_db()?;

        let installed = self.db.list_installed_packages()?;
        println!("{}", t!("repo_rebuild_verified", count = installed.len()));
        self.db.record_action(HistoryAction {
            trace_id,
            operation: "rebuild-db".to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            details: t!("repo_rebuild_detail").to_string(),
        })?;
        println!("{}", t!("repo_rebuild_success"));
        Ok(())
    }

    /// /var/lib/luppo/package dizinini tarayarak kurulu paketleri DB'ye geri yükler.
    pub fn rebuild_installed_db(&self) -> LuppoResult<()> {
        let pkg_dir = &self.config.directories.packages_dir;
        if !pkg_dir.exists() {
            return Ok(());
        }

        println!("{}", t!("repo_rebuild_checking_fs"));

        let mut recovered_count = 0;

        // Önce mevcut Sled kayıtlarını temizleyelim
        self.db.clear_installed_packages()?;

        if let Ok(entries) = fs::read_dir(pkg_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let metadata_path = path.join("metadata.xml");
                    let files_path = path.join("files.xml");

                    if metadata_path.exists() && files_path.exists() {
                        if let Ok(pkg_info) =
                            self.parse_installed_metadata(&metadata_path, &files_path)
                        {
                            if self.db.install_package(&pkg_info).is_ok() {
                                // Dosya sahipliği kaydı
                                for file_path in pkg_info.installed_files.keys() {
                                    let _ = self.db.register_file(file_path, &pkg_info.name);
                                }
                                recovered_count += 1;
                            }
                        }
                    }
                }
            }
        }

        if recovered_count > 0 {
            println!("{}", t!("repo_rebuild_recovered", count = recovered_count));
        }

        Ok(())
    }

    fn parse_installed_metadata(
        &self,
        metadata_path: &std::path::Path,
        files_path: &std::path::Path,
    ) -> LuppoResult<InstalledPackage> {
        let metadata_content = fs::read_to_string(metadata_path).map_err(LuppoError::IoError)?;
        let luppo_root: LuppoRoot = serde_xml_rs::from_str(&metadata_content).map_err(|e| {
            LuppoError::RuntimeError(t!("error_metadata_parse", error = e).to_string())
        })?;

        let mut pkg = luppo_root.package;
        if let Some(latest) = pkg.history.updates.first() {
            pkg.version = latest.version.clone();
            pkg.release = latest.release;
        }

        let files_content = fs::read_to_string(files_path).map_err(LuppoError::IoError)?;
        let files_root: FilesXmlRoot = serde_xml_rs::from_str(&files_content)
            .map_err(|e| LuppoError::RuntimeError(t!("error_files_parse", error = e).to_string()))?;

        let mut installed_files = std::collections::HashMap::new();
        let mut total_size = 0;
        for file in files_root.files {
            let mode = u32::from_str_radix(&file.mode, 8).unwrap_or(0o644);
            installed_files.insert(
                file.path.clone(),
                FileMetadata {
                    mode,
                    uid: file.uid,
                    gid: file.gid,
                    size: file.size,
                },
            );
            total_size += file.size;
        }

        let description = pkg
            .descriptions
            .first()
            .map(|d| d.text.clone())
            .unwrap_or_default();
        let install_date = if let Ok(metadata) = fs::metadata(metadata_path) {
            if let Ok(created) = metadata.created() {
                let datetime: chrono::DateTime<chrono::Local> = created.into();
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            } else {
                Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
            }
        } else {
            Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
        };

        let homepage = pkg.effective_homepage();
        let screenshot = pkg.effective_screenshot();
        let packager = pkg.effective_packager();

        Ok(InstalledPackage {
            name: pkg.name,
            version: pkg.version,
            description,
            install_date,
            installed_files,
            total_size,
            package_hash: pkg.package_hash,
            release: pkg.release,
            distribution_release: pkg.distribution_release,
            licenses: pkg.licenses,
            provides: pkg.provides,
            post_remove: pkg.post_remove,
            pre_remove: pkg.pre_remove,
            homepage,
            icon: pkg.icon,
            screenshot,
            packager,
            install_tar_hash: pkg.install_tar_hash,
            package_format: pkg.package_format,
            build_host: pkg.build_host,
            distribution: pkg.distribution,
            configured: true,
            signature_verified: false,
        })
    }

    pub fn perform_clean(&self) -> LuppoResult<()> {
        println!("{}", t!("repo_clean_starting"));
        let mut found = false;
        if let Ok(entries) = fs::read_dir("./") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && path.extension().is_some_and(|ext| ext == "lock")
                    && fs::remove_file(&path).is_ok()
                {
                    println!("{}", t!("repo_clean_cleaned", path = path.display()));
                    found = true;
                }
            }
        }
        if !found {
            println!("{}", t!("repo_clean_not_found"));
        } else {
            println!("{}", t!("repo_clean_success"));
        }
        Ok(())
    }

    pub fn perform_update(&self, trace_id: u64) -> LuppoResult<()> {
        println!("{}", t!("repo_update_starting"));
        self.update_repositories_internal(trace_id)?;
        self.db.record_action(HistoryAction {
            trace_id,
            operation: "update".to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            details: t!("repo_update_detail").to_string(),
        })?;
        println!("{}", t!("repo_update_success"));
        Ok(())
    }

    // --- GÜNCELLEME SORGULAMA ---

    pub fn find_updates(
        &self,
        db: &LuppoDatabase,
        compare_hashes: bool,
    ) -> LuppoResult<Vec<(String, String, String)>> {
        let mut updates = Vec::new();
        let installed_packages = db.list_installed_packages()?;
        let remote_packages = db.list_available_packages()?;

        for installed in installed_packages {
            if let Some(remote) = remote_packages.iter().find(|p| p.name == installed.name) {
                let version_changed = remote.latest_version() != installed.version;
                let release_changed = remote.release != installed.release;

                // SENİN ÖNERDİĞİN KRİTİK KONTROL:
                // Sürüm ve release aynı olsa bile hash farklıysa paket yeniden derlenmiştir.
                let hash_changed = remote.package_hash != installed.package_hash;

                if version_changed || release_changed || (compare_hashes && hash_changed) {
                    let reason = if hash_changed && !release_changed {
                        t!(
                            "repo_upgrade_reason_rebuilt",
                            version = remote.latest_version()
                        )
                        .to_string()
                    } else {
                        t!("repo_upgrade_reason_new", version = remote.latest_version()).to_string()
                    };

                    updates.push((installed.name.clone(), installed.version.clone(), reason));
                }
            }
        }
        Ok(updates)
    }

    /// Uzak bir URL'den lopec dosyasını ve onunla ilişkili companion (actions.py, translations.xml vb.),
    /// ek dosyaları (additional_files) ve yamaları (patches) indirip yerel bir geçici dizine kaydeder.
    pub fn download_remote_spec(&self, url: &str) -> LuppoResult<PathBuf> {
        let base_url = if url.ends_with("/lopec.xml") {
            url.trim_end_matches("lopec.xml")
        } else if url.ends_with("/lopec.kdl") {
            url.trim_end_matches("lopec.kdl")
        } else {
            return Err(LuppoError::RuntimeError(
                t!("repo_err_invalid_lopec_url").to_string(),
            ));
        };

        // Benzersiz geçici dizin oluştur
        let unique_id = chrono::Utc::now().timestamp_millis();
        let temp_dir = PathBuf::from(format!("/var/tmp/luppo/remote-build/{}", unique_id));
        fs::create_dir_all(&temp_dir).map_err(LuppoError::IoError)?;

        let filename = url.split('/').next_back().unwrap_or("lopec.xml");
        let lopec_file_path = temp_dir.join(filename);

        println!("{}", t!("repo_downloading", url = url));

        let client = self.get_client()?;
        let res = client
            .get(url)
            .send()
            .map_err(|e| LuppoError::RuntimeError(e.to_string()))?;
        if !res.status().is_success() {
            return Err(LuppoError::RuntimeError(t!(
                "repo_err_lopec_download_failed",
                status = res.status().as_u16()
            ).to_string()));
        }
        let content = res
            .text()
            .map_err(|e| LuppoError::RuntimeError(e.to_string()))?;
        fs::write(&lopec_file_path, &content).map_err(LuppoError::IoError)?;

        // lopec'i parse et ki bağımlılıklarını/yamalarını/ek dosyalarını bilelim
        let spec = luppo_spec::models::LuppoSpec::from_path(&lopec_file_path).map_err(|e| {
            LuppoError::RuntimeError(t!("repo_err_lopec_parse_failed", error = e).to_string())
        })?;

        // companion dosyaları indir
        for companion in &["actions.py", "translations.xml"] {
            let comp_url = format!("{}{}", base_url, companion);
            let comp_dest = temp_dir.join(companion);
            if let Ok(res) = client.get(&comp_url).send() {
                if res.status().is_success() {
                    if let Ok(bytes) = res.bytes() {
                        println!("{}", t!("repo_downloading", url = &comp_url));
                        let _ = fs::write(&comp_dest, bytes);
                    }
                }
            }
        }

        // Ek dosyaları ve yamaları indir (files/ dizini altına)
        let files_dir = temp_dir.join("files");
        let mut files_dir_created = false;

        let ensure_files_dir = |created: &mut bool, dir: &PathBuf| -> Result<(), LuppoError> {
            if !*created {
                fs::create_dir_all(dir).map_err(LuppoError::IoError)?;
                *created = true;
            }
            Ok(())
        };

        // 1. Ek dosyaları indir
        for pkg in &spec.packages {
            if let Some(wrapper) = &pkg.additional_files {
                for file in &wrapper.files {
                    ensure_files_dir(&mut files_dir_created, &files_dir)?;
                    let file_url = format!("{}files/{}", base_url, file.filename);
                    let file_dest = files_dir.join(&file.filename);
                    println!("{}", t!("repo_downloading", url = &file_url));
                    if let Ok(res) = client.get(&file_url).send() {
                        if res.status().is_success() {
                            if let Ok(bytes) = res.bytes() {
                                let _ = fs::write(&file_dest, bytes);
                            }
                        }
                    }
                }
            }
        }

        // 2. Yamaları indir
        if let Some(wrapper) = &spec.source.patches {
            for patch in &wrapper.patches {
                ensure_files_dir(&mut files_dir_created, &files_dir)?;
                let patch_url = format!("{}files/{}", base_url, patch.file);
                let patch_dest = files_dir.join(&patch.file);
                println!("{}", t!("repo_downloading", url = &patch_url));
                if let Ok(res) = client.get(&patch_url).send() {
                    if res.status().is_success() {
                        if let Ok(bytes) = res.bytes() {
                            let _ = fs::write(&patch_dest, bytes);
                        }
                    }
                }
            }
        }

        Ok(lopec_file_path)
    }

    pub fn perform_check_repo(&self, check_circular: bool) -> LuppoResult<()> {
        println!("{}", t!("repo_check_started"));
        let mut packages = std::collections::HashMap::new();

        if let Ok(all_packages) = self.db.list_available_packages() {
            for pkg in all_packages {
                packages.insert(pkg.name.clone(), pkg);
            }
        }

        if check_circular {
            println!("{}", t!("repo_check_circular"));
            use petgraph::algo::toposort;
            use petgraph::graph::{DiGraph, NodeIndex};

            let mut graph = DiGraph::<String, ()>::new();
            let mut nodes = std::collections::HashMap::<String, NodeIndex>::new();

            for name in packages.keys() {
                let node = graph.add_node(name.clone());
                nodes.insert(name.clone(), node);
            }

            for (name, pkg) in &packages {
                let current_node = nodes[name];
                if let Some(deps) = &pkg.runtime_dependencies {
                    for dep in &deps.dependencies {
                        if let Some(dep_node) = nodes.get(dep) {
                            graph.add_edge(current_node, *dep_node, ());
                        }
                    }
                }
            }

            match toposort(&graph, None) {
                Ok(_) => {
                    println!("{}", t!("repo_check_circular_ok"));
                }
                Err(cycle) => {
                    let node_name = &graph[cycle.node_id()];
                    println!("{} {}", t!("repo_check_circular_fail"), node_name);
                }
            }
        }

        Ok(())
    }

    fn fetch_standalone_index(&self, uri: &str) -> LuppoResult<LuppoIndex> {
        let mut raw_data = Vec::new();

        if uri.starts_with("http://") || uri.starts_with("https://") {
            let client = self.get_client()?;
            let mut res = client
                .get(uri)
                .send()
                .map_err(|e| LuppoError::RuntimeError(e.to_string()))?;
            if !res.status().is_success() {
                return Err(LuppoError::RuntimeError(t!(
                    "repo_err_fetch_failed",
                    status = res.status().as_u16()
                ).to_string()));
            }
            res.read_to_end(&mut raw_data).map_err(LuppoError::IoError)?;
        } else {
            let path = uri.strip_prefix("file://").unwrap_or(uri);
            let mut file = fs::File::open(path).map_err(LuppoError::IoError)?;
            file.read_to_end(&mut raw_data)
                .map_err(LuppoError::IoError)?;
        }

        let is_xz = uri.ends_with(".xz");
        let content_str = if is_xz {
            let mut decoder = XzDecoder::new(raw_data.as_slice());
            let mut decompressed = String::new();
            decoder
                .read_to_string(&mut decompressed)
                .map_err(LuppoError::IoError)?;
            decompressed
        } else {
            String::from_utf8(raw_data).map_err(|e| LuppoError::RuntimeError(e.to_string()))?
        };

        let is_json = if is_xz {
            uri.trim_end_matches(".xz").ends_with(".json")
        } else {
            uri.ends_with(".json")
        };

        if is_json {
            serde_json::from_str(&content_str)
                .map_err(|e| LuppoError::RuntimeError(t!("repo_error_json", error = e).to_string()))
        } else {
            quick_xml::de::from_str(&content_str)
                .map_err(|e| LuppoError::RuntimeError(t!("error_xml_parse", error = e).to_string()))
        }
    }

    pub fn perform_repo_diff(&self, source_index: &str, binary_index: &str) -> LuppoResult<()> {
        println!(
            "{}",
            t!("repo_diff_started", src = source_index, bin = binary_index)
        );

        let src_index = self.fetch_standalone_index(source_index)?;
        let bin_index = self.fetch_standalone_index(binary_index)?;

        let mut src_map = std::collections::HashMap::new();
        let mut bin_map = std::collections::HashMap::new();

        // Populate src_map and bin_map
        for pkg in src_index.packages {
            let release = pkg.history.updates.first().map(|u| u.release).unwrap_or(0);
            src_map.insert(pkg.name, release);
        }
        for pkg in bin_index.packages {
            let release = pkg.history.updates.first().map(|u| u.release).unwrap_or(0);
            bin_map.insert(pkg.name, release);
        }

        let mut missing_in_bin = Vec::new();
        let mut missing_in_src = Vec::new();
        // src_release > bin_release: kaynak daha yeni → derlenmesi gereken
        let mut needs_compile: Vec<(String, u32, u32)> = Vec::new();
        // bin_release > src_release: binary daha yeni → ters yön (bilgi amaçlı)
        let mut bin_ahead: Vec<(String, u32, u32)> = Vec::new();

        for (pkg_name, src_release) in &src_map {
            if let Some(bin_release) = bin_map.get(pkg_name) {
                if src_release > bin_release {
                    needs_compile.push((pkg_name.clone(), *src_release, *bin_release));
                } else if bin_release > src_release {
                    bin_ahead.push((pkg_name.clone(), *bin_release, *src_release));
                }
            } else {
                missing_in_bin.push(pkg_name.clone());
            }
        }

        // Packages in bin but not in src
        for pkg_name in bin_map.keys() {
            if !src_map.contains_key(pkg_name) {
                missing_in_src.push(pkg_name.clone());
            }
        }

        missing_in_bin.sort();
        missing_in_src.sort();
        needs_compile.sort_by(|a, b| a.0.cmp(&b.0));
        bin_ahead.sort_by(|a, b| a.0.cmp(&b.0));

        if !missing_in_bin.is_empty() {
            println!("\n* Packages missing in {}", binary_index);
            for p in missing_in_bin {
                println!("  {}", p);
            }
        }

        if !missing_in_src.is_empty() {
            println!("\n* Packages missing in {}", source_index);
            for p in missing_in_src {
                println!("  {}", p);
            }
        }

        if !needs_compile.is_empty() {
            println!("\n* Packages that need compiling (src newer than bin)");
            for (p, src_rel, bin_rel) in needs_compile {
                println!("  {}  ({} > {})", p, src_rel, bin_rel);
            }
        }

        if !bin_ahead.is_empty() {
            println!("\n* Packages ahead in binary (bin newer than src)");
            for (p, bin_rel, src_rel) in bin_ahead {
                println!("  {}  ({} > {})", p, bin_rel, src_rel);
            }
        }

        Ok(())
    }
}


