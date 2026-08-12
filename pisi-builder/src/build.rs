use crate::actionsapi as pisi;
use crate::sandbox::SandboxContext;
use indicatif::{ProgressBar, ProgressStyle};
use pisi_core::colorize;
use rayon::prelude::*;
use pisi_core::config::Config;
use pisi_core::database::PisiDatabase;
use pisi_core::packager::Packager;
use pisi_core::repo::Repository;
use pisi_core::resolver::resolve_build_deps_static;
use pisi_spec::models::PisiSpec;
use rust_i18n::t;
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// İnşa sürecine özel seçenekleri tutan yapı.
#[derive(Debug, Clone)]
pub struct BuildOptions {
    pub jobs: usize,
    pub verbose: bool,
    pub debug: bool,
    pub log_path: Option<PathBuf>,
    pub optimization_level: Option<String>,
    pub enable_sandbox: bool,
    pub architecture: String,
    pub yes_all: bool,
    /// Sbindir yolu (örn. "usr/sbin" veya "usr/bin").
    /// Varsayılan: "usr/bin" (usrmerge).
    /// Paketçi "--sbindir usr/sbin" ile geçersiz kılabilir.
    pub sbindir: String,
    /// Yalnızca build aşamasını çalıştır (kaynak hazırlama + derleme)
    pub run_build: bool,
    /// Yalnızca install aşamasını çalıştır
    pub run_install: bool,
    /// Yalnızca package aşamasını çalıştır
    pub run_package: bool,
}

/// Gerçek paket inşa sürecini yöneten yapı.
pub struct PackageBuilder {
    spec: PisiSpec,
    /// pkg_dir: /var/pisi/<name>-<ver>-<rel>/
    work_dir: PathBuf,
    /// specdir: pspec.xml veya .kdl dosyasının bulunduğu dizin
    specdir: PathBuf,
    db: PisiDatabase,
    config: Config,
    options: BuildOptions,
}

impl PackageBuilder {
    pub fn new(
        spec: PisiSpec,
        work_dir: PathBuf,
        specdir: PathBuf,
        db: PisiDatabase,
        config: Config,
        options: BuildOptions,
    ) -> Self {
        Self {
            spec,
            work_dir,
            specdir,
            db,
            config,
            options,
        }
    }

    /// Python PiSi'nin pkg_work_dir(): <pkg_dir>/work/ veya <pkg_dir>/work-<build_type>/
    fn pkg_work_dir(&self, build_type: &str) -> PathBuf {
        let base = self.work_dir.join("work");
        if build_type.is_empty() {
            base
        } else {
            self.work_dir.join(format!("work-{}", build_type))
        }
    }

    /// Python PiSi'nin pkg_install_dir(): <pkg_dir>/install/
    fn pkg_install_dir(&self) -> PathBuf {
        self.work_dir.join("install")
    }

    /// Build state dosyasının yolu: <pkg_dir>/work/pisiBuildState
    fn state_file_path(&self, build_type: &str) -> PathBuf {
        self.pkg_work_dir(build_type).join("pisiBuildState")
    }

    /// Kaydedilmiş build state'i okur, yoksa "none" döndürür.
    fn read_state(&self, build_type: &str) -> String {
        let path = self.state_file_path(build_type);
        if path.exists() {
            fs::read_to_string(&path).unwrap_or_else(|_| "none".to_string())
        } else {
            "none".to_string()
        }
    }

    /// Build state'i dosyaya yazar.
    fn write_state(&self, build_type: &str, state: &str) {
        let path = self.state_file_path(build_type);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&path, state);
    }

    /// State sıralaması: "none" < "unpack" < "build" < "install" < "package"
    fn state_order(state: &str) -> u8 {
        match state {
            "unpack" => 1,
            "build" => 2,
            "install" => 3,
            "package" => 4,
            _ => 0,
        }
    }

    /// Belirtilen state zaten yapılmış mı?
    fn is_state_done(&self, build_type: &str, required: &str) -> bool {
        let current = self.read_state(build_type);
        Self::state_order(&current) >= Self::state_order(required)
    }

    /// İnşa sürecini sandbox içerisinde başlatır.
    pub fn build(&self, _trace_id: u64) -> Result<Vec<PathBuf>, String> {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✔️"])
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));

        match self.internal_build(_trace_id, &spinner) {
            Ok(paths) => {
                spinner.finish_with_message(t!("build_success_msg"));
                Ok(paths)
            }
            Err(e) => {
                spinner.finish_with_message(t!("build_failed_msg"));
                self.write_debug_log(&e);
                Err(e)
            }
        }
    }

    /// Asıl inşa adımlarını içeren iç metod.
    fn internal_build(
        &self,
        _trace_id: u64,
        spinner: &ProgressBar,
    ) -> Result<Vec<PathBuf>, String> {
        spinner.println(colorize(&t!("build_starting", package = self.spec.source.name), "green"));
        spinner.set_message(colorize(&t!("build_step_1", name = self.spec.source.name), "brightcyan"));

        // Bu sürecin çalışma dizinini work_dir olarak ayarla
        pisi::cd(&self.work_dir)?;

        // 0. Çevre Değişkenlerini Seçeneklere Göre Ayarla
        pisi_core::safe_env::set_var("MAKEOPTS", format!("-j{}", self.options.jobs));
        pisi_core::safe_env::set_var("MAKEFLAGS", format!("-j{}", self.options.jobs));
        if self.options.verbose {
            pisi_core::safe_env::set_var("VERBOSE", "1");
        }
        if self.options.debug {
            pisi_core::safe_env::set_var("PISI_DEBUG", "1");
        }
        if let Some(ref level) = self.options.optimization_level {
            // Eğer sadece "2" veya "s" verildiyse başına "-O" ekle, zaten "-O2" verildiyse olduğu gibi kullan.
            let opt_flag = if level.starts_with("-O") {
                level.clone()
            } else {
                format!("-O{}", level)
            };
            pisi_core::safe_env::set_var("CFLAGS", &opt_flag);
            pisi_core::safe_env::set_var("CXXFLAGS", &opt_flag);
            // LDFLAGS: derleme optimizasyon seviyesine göre bağlayıcı optimizasyonunu ayarla
            let opt_num_str = opt_flag.trim_start_matches("-O");
            let ld_opt = if let Ok(n) = opt_num_str.parse::<i32>() {
                let capped = n.clamp(0, 2);
                format!("-Wl,-O{}", capped)
            } else {
                "-Wl,-O1".to_string()
            };
            pisi_core::safe_env::set_var("LDFLAGS", format!("{} -Wl,-z,relro -Wl,--hash-style=gnu", ld_opt));
        }
        if let Some(ref log) = self.options.log_path {
            // actionsapi'nin log yolunu görebilmesi için çevre değişkeni ayarla
            pisi_core::safe_env::set_var("PISI_BUILD_LOG", log.to_string_lossy());
        }

        // 0. Çapraz Derleme Ortamını Seçeneklere Göre Hazırla
        let host_arch = std::env::consts::ARCH;
        if self.options.architecture != host_arch {
            self.setup_cross_compiler_env(spinner)?;
        }

        // 0. Build Bağımlılıklarını Çöz ve Kur (kaynak + alt paketler)
        let mut all_build_deps: Vec<String> = Vec::new();

        if let Some(build_deps_config) = &self.spec.source.build_dependencies {
            for dep in &build_deps_config.dependencies {
                if !all_build_deps.contains(&dep.name) {
                    all_build_deps.push(dep.name.clone());
                }
            }
        }

        for pkg in &self.spec.packages {
            for dep_name in &pkg.deps.build {
                if !all_build_deps.contains(dep_name) {
                    all_build_deps.push(dep_name.clone());
                }
            }
        }

        if !all_build_deps.is_empty() {
            // Build ortamında hâlihazırda kurulu olan paketler
            let build_env_installed: HashSet<String> = if self.options.enable_sandbox {
                HashSet::new()
            } else {
                self.db.list_installed_package_names().unwrap_or_default()
            };

            // Karşılanmamış build bağımlılıkları
            let unsatisfied: Vec<&str> = all_build_deps
                .iter()
                .filter(|n| !build_env_installed.contains(*n))
                .map(|s| s.as_str())
                .collect();

            if !unsatisfied.is_empty() {
                // 1. Prompt: unsatisfied build deps
                println!("Unsatisfied Build Dependencies: {}", unsatisfied.join(" "));

                let mut cancelled = false;
                if !self.options.yes_all {
                    spinner.suspend(|| {
                        use std::io::Write;
                        print!(
                            "Do you want to install the unsatisfied build dependencies (yes/no) "
                        );
                        std::io::stdout().flush().unwrap();
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap();
                        let trimmed = input.trim().to_lowercase();
                        if trimmed != "y"
                            && trimmed != "yes"
                            && trimmed != "e"
                            && trimmed != "evet"
                        {
                            cancelled = true;
                        }
                    });

                    if cancelled {
                        return Err("Build cancelled by user.".to_string());
                    }
                }

                // Transitive bağımlılıkları çöz
                // DB'yi ısınmak için mevcut paket isimlerini çek (sadece key'ler, hızlı),
                // sonra iterative DFS + cache ile tek tek yükle
                let transitive_deps =
                    resolve_build_deps_static(&self.db, &all_build_deps, &build_env_installed)
                        .map_err(|e| t!("build_err_deps", error = e).to_string())?;

                // 2. Prompt: full plan
                println!("\nFollowing packages will be installed:");
                let dep_names: Vec<String> = transitive_deps.iter().map(|d| d.name.clone()).collect();
                pisi_core::print_in_columns(&dep_names);


                let build_deps_set: HashSet<&str> =
                    all_build_deps.iter().map(|s| s.as_str()).collect();
                let has_extras = transitive_deps
                    .iter()
                    .any(|d| !build_deps_set.contains(d.name.as_str()));

                let mut cancelled = false;
                if has_extras && !self.options.yes_all {
                    spinner.suspend(|| {
                        use std::io::Write;
                        print!(
                            "\nThere are extra packages due to dependencies. Do you want to continue? (yes/no) "
                        );
                        std::io::stdout().flush().unwrap();
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap();
                        let trimmed = input.trim().to_lowercase();
                        if trimmed != "y"
                            && trimmed != "yes"
                            && trimmed != "e"
                            && trimmed != "evet"
                        {
                            cancelled = true;
                        }
                    });

                    if cancelled {
                        return Err("Build cancelled by user.".to_string());
                    }
                }

                // Kurulum
                if self.options.enable_sandbox {
                    spinner.println(colorize(&t!("build_deps_preparing", count = transitive_deps.len()), "cyan"));
                    let repo_manager = Repository::new(self.db.clone(), self.config.clone());
                    let all_available = self
                        .db
                        .list_available_packages()
                        .map_err(|e| e.to_string())?;

                    for dep_def in &transitive_deps {
                        if let Some(pkg_data) =
                            all_available.iter().find(|p| p.name == dep_def.name)
                        {
                            let pisi_path = repo_manager
                                .fetch_package(pkg_data, None, None, None, None)
                                .map_err(|e| {
                                    t!(
                                        "build_err_dep_fetch",
                                        name = dep_def.name,
                                        error = format!("{:?}", e)
                                    )
                                    .to_string()
                                })?;

                            let package_data = Packager::read_package(
                                pisi_path.to_str().unwrap(),
                            )
                            .map_err(|e| {
                                t!("build_err_pkg_parse", error = format!("{:?}", e))
                                    .to_string()
                            })?;

                            for file in package_data.files {
                                let target_path =
                                    self.work_dir.join(file.path.trim_start_matches('/'));
                                if let Some(parent) = target_path.parent() {
                                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                                }
                                fs::write(&target_path, &file.content)
                                    .map_err(|e| e.to_string())?;
                            }
                        }
                    }
                } else {
                    spinner.println(t!(
                        "build_installing_host_deps",
                        count = transitive_deps.len()
                    ));
                    let installer = pisi_core::installer::Installer::new(
                        self.db.clone(),
                        self.config.clone(),
                    );
                    let dep_names: Vec<String> =
                        transitive_deps.into_iter().map(|d| d.name).collect();
                    spinner.suspend(|| {
                        installer.perform_install(dep_names, _trace_id, false, true, None, None, false, false, false, true, false, false, true, None)
                    })
                    .map_err(|e| {
                        t!("build_err_host_dep_fail", name = "dependencies", error = e)
                            .to_string()
                    })?;
                    spinner.println(colorize(&t!("build_host_deps_ready"), "green"));
                }
            } else {
                spinner.println(colorize(&t!("build_deps_satisfied"), "green"));
            }
        }

        // install/ ve work/ dizinlerini hazırla
        let install_root_full = self.pkg_install_dir();
        let install_root_relative = "install";
        fs::create_dir_all(&install_root_full).map_err(|e| e.to_string())?;

        let latest_update = self
            .spec
            .history
            .as_ref()
            .and_then(|h| h.updates.first())
            .ok_or_else(|| t!("build_err_history_empty").to_string())?;
        let _pkg_version = latest_update.version.clone();
        let _pkg_release = latest_update.release.to_string();

        // Faz seçimi:
        //   --build:    configure/build/check + install/strip + package (extract/patches atlanır)
        //   --install:  install/strip + package (extract/patches + build atlanır)
        //   --package:  sadece paket oluşturma
        //   (hiçbiri):  tüm aşamalar
        let has_phase_flags =
            self.options.run_build || self.options.run_install || self.options.run_package;
        let do_prepare = !has_phase_flags;  // extract + patches (yalnızca hiçbir flag yoksa)
        let do_build = !has_phase_flags || self.options.run_build;
        let do_install = !has_phase_flags || self.options.run_build || self.options.run_install;
        let do_package = !has_phase_flags || self.options.run_build || self.options.run_install || self.options.run_package;

        if do_build || do_install {
            // Install dizinini oluştur
            fs::create_dir_all(&install_root_full).map_err(|e| e.to_string())?;

            if self.options.enable_sandbox {
                let sandbox = SandboxContext::new(self.work_dir.clone());
                sandbox.setup_and_run(|| {
                    self.run_build_steps(
                        &install_root_full,
                        install_root_relative,
                        spinner,
                        do_prepare,
                        do_build,
                        do_install,
                    )
                })?;
            } else {
                spinner.println(t!("build_sandbox_disabled"));
                self.run_build_steps(
                    &install_root_full,
                    install_root_relative,
                    spinner,
                    do_prepare,
                    do_build,
                    do_install,
                )?;
            }
        } else {
            spinner.println(colorize("→ Build/Install aşaması atlandı (--package).", "yellow"));
        }

        // Package-level AdditionalFiles: specdir/files/<file> → install_root/<target>
        if do_package {
            spinner.println(colorize("→ Paket seviyesindeki AdditionalFiles kopyalanıyor...", "cyan"));
            for pkg_def in &self.spec.packages {
                let pkg_install_root = self.pkg_install_dir();
                if let Some(ref afw) = pkg_def.additional_files {
                    for af in &afw.files {
                        let candidates = if !af.filename.is_empty() && af.filename != af.target {
                            vec![af.filename.clone(), af.target.clone()]
                        } else {
                            vec![af.target.clone(), af.filename.clone()]
                        };
                        let mut copied = false;
                        for rel in &candidates {
                            let src = self.specdir.join("files").join(rel.trim_start_matches('/'));
                            if src.is_file() {
                                let target_path = af.target.trim_start_matches('/');
                                let dst = pkg_install_root.join(target_path);
                                if let Some(p) = dst.parent() {
                                    fs::create_dir_all(p).map_err(|e| e.to_string())?;
                                }
                                if let Err(e) = fs::copy(&src, &dst) {
                                    eprintln!(
                                        "Warning: could not copy additional file {} -> {}: {}",
                                        src.display(),
                                        dst.display(),
                                        e
                                    );
                                }
                                copied = true;
                                break;
                            }
                        }
                        if !copied {
                            eprintln!(
                                "Warning: additional file '{}' not found in files/ directory",
                                af.target
                            );
                        }
                    }
                }
            }

            // 4. Paketleme (Tüm paketleri döngüye al)
            spinner.set_message(colorize(&t!("build_step_7"), "brightcyan"));
            let mut created_packages = Vec::new();

            let results: Vec<Result<PathBuf, String>> = self.spec.packages.par_iter().enumerate().map(|(pkg_idx, pkg_def)| {
                spinner.println(colorize(&t!("build_packaging_pkg", name = &pkg_def.name), "cyan"));

                let collisions = crate::package::check_path_collision(pkg_idx, &self.spec.packages);
                let pkg_install_root = self.pkg_install_dir();

                crate::package::create_pisi_package(
                    &self.spec,
                    pkg_idx,
                    &pkg_install_root,
                    &self.work_dir,
                    &self.config.general.distribution_id,
                    &self.config.general.architecture,
                    &self.specdir,
                    None,
                    &collisions,
                )
            }).collect();

            for res in results {
                let output_path = res?;
                if !output_path.as_os_str().is_empty() {
                    created_packages.push(output_path);
                }
            }

            Ok(created_packages)
        } else {
            spinner.println(colorize("→ Paketleme aşaması atlandı.", "yellow"));
            Ok(Vec::new())
        }
    }

    /// Derleme adımlarını çalıştıran yardımcı fonksiyon.
    /// `do_prepare`: Kaynak çıkarma + yamalar (yalnızca hiçbir flag yoksa)
    /// `do_build`: configure/setup + build + check
    /// `do_install`: Kurulum + strip/post-processing
    fn run_build_steps(
        &self,
        _install_root_full: &PathBuf,
        install_root_relative: &str,
        spinner: &ProgressBar,
        do_prepare: bool,
        do_build: bool,
        do_install: bool,
    ) -> Result<(), String> {
        let has_phase_flags =
            self.options.run_build || self.options.run_install || self.options.run_package;
        let latest_update = self
            .spec
            .history
            .as_ref()
            .and_then(|h| h.updates.first())
            .ok_or_else(|| t!("build_err_history_empty").to_string())?;
        let pkg_version = latest_update.version.clone();
        let pkg_release = latest_update.release.to_string();

        let pkg_name = &self.spec.source.name;

        pisi_core::safe_env::set_var("SRC_NAME", pkg_name);
        pisi_core::safe_env::set_var("SRC_VERSION", &pkg_version);
        pisi_core::safe_env::set_var("SRC_RELEASE", &pkg_release);

        let host = format!("{}-pc-linux-gnu", std::env::consts::ARCH);
        pisi_core::safe_env::set_var("HOST", &host);
        if std::env::var("CC").is_err() {
            pisi_core::safe_env::set_var("CC", format!("{}-gcc", &host));
        }
        if std::env::var("CXX").is_err() {
            pisi_core::safe_env::set_var("CXX", format!("{}-g++", &host));
        }
        if std::env::var("CFLAGS").is_err() {
            pisi_core::safe_env::set_var("CFLAGS", "-O2 -fomit-frame-pointer");
        }
        if std::env::var("CXXFLAGS").is_err() {
            pisi_core::safe_env::set_var("CXXFLAGS", "-O2 -fomit-frame-pointer");
        }
        if std::env::var("LDFLAGS").is_err() {
            pisi_core::safe_env::set_var("LDFLAGS", "-Wl,-O1 -Wl,-z,relro -Wl,--hash-style=gnu");
        }
        if std::env::var("JOBS").is_err() {
            pisi_core::safe_env::set_var("JOBS", pisi::get::make_jobs());
        }

        let mut build_types: Vec<String> = vec!["".to_string()];
        for pkg in &self.spec.packages {
            if let Some(ref bt) = pkg.build_type {
                let trimmed = bt.trim();
                if !trimmed.is_empty() && !build_types.contains(&trimmed.to_string()) {
                    build_types.push(trimmed.to_string());
                }
            }
        }

        // ignored_build_types config'te belirtilen tipleri filtrele
        let ignored: std::collections::HashSet<&str> = self
            .config
            .build
            .ignored_build_types
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if !ignored.is_empty() {
            build_types.retain(|bt| bt.is_empty() || !ignored.contains(bt.as_str()));
        }

        pisi_core::safe_env::set_var("PISI_SBINDIR", &self.options.sbindir);

        let actions_py_path = self.work_dir.join("actions.py");
        let octal_re = regex::Regex::new(r"\b0([0-7]+)\b");

        // NoStrip listesi: actions.py'deki Python globallerinden toplanır
        let mut no_strip: Vec<String> = Vec::new();

        // Build type döngüsü öncesi çevre değişkenlerini kaydet (emul32 sızıntısını önlemek için)
        let initial_env = std::collections::HashMap::from([
            ("HOST", std::env::var("HOST").unwrap_or_default()),
            ("CHOST", std::env::var("CHOST").unwrap_or_default()),
            ("CC", std::env::var("CC").unwrap_or_default()),
            ("CXX", std::env::var("CXX").unwrap_or_default()),
            ("CFLAGS", std::env::var("CFLAGS").unwrap_or_default()),
            ("CXXFLAGS", std::env::var("CXXFLAGS").unwrap_or_default()),
            ("LDFLAGS", std::env::var("LDFLAGS").unwrap_or_default()),
            ("PKG_CONFIG_LIBDIR", std::env::var("PKG_CONFIG_LIBDIR").unwrap_or_default()),
            ("PKG_CONFIG_SYSROOT_DIR", std::env::var("PKG_CONFIG_SYSROOT_DIR").unwrap_or_default()),
        ]);

        for bt in &build_types {
            if !bt.is_empty() {
                spinner.println(t!("build_extra_build_type", bt = bt));
            }
            pisi_core::safe_env::set_var("PISI_BUILD_TYPE", bt);

            let bt_work_dir = self.pkg_work_dir(bt);
            pisi_core::safe_env::set_var("WORK_DIR", bt_work_dir.to_str().unwrap_or("work"));
            pisi_core::safe_env::set_var("HOME", bt_work_dir.to_str().unwrap_or("work"));
            pisi_core::safe_env::set_var("PKG_DIR", self.work_dir.to_str().unwrap_or("/var/pisi"));
            pisi_core::safe_env::set_var("PYTHONDONTWRITEBYTECODE", "1");

            let cargo_home = bt_work_dir.join(".cargo");
            if !cargo_home.exists() {
                fs::create_dir_all(&cargo_home).map_err(|e| e.to_string())?;
            }
            pisi_core::safe_env::set_var("CARGO_HOME", cargo_home.to_str().unwrap_or(""));

            let install_root_bt = self.pkg_install_dir();
            pisi_core::safe_env::set_var("INSTALL_DIR", install_root_bt.to_str().unwrap_or("install"));

            // emul32 (32-bit) build type için derleyici ve pkg-config ortam değişkenlerini ayarla
            if bt == "emul32" {
                pisi_core::safe_env::set_var("HOST", "i686-pc-linux-gnu");
                pisi_core::safe_env::set_var("CHOST", "i686-pc-linux-gnu");
                pisi_core::safe_env::set_var("CC", "gcc -m32");
                pisi_core::safe_env::set_var("CXX", "g++ -m32");
                let cflags = std::env::var("CFLAGS").unwrap_or_default();
                if !cflags.contains("-m32") {
                    pisi_core::safe_env::set_var("CFLAGS", format!("{} -m32", cflags).trim());
                }
                let cxxflags = std::env::var("CXXFLAGS").unwrap_or_default();
                if !cxxflags.contains("-m32") {
                    pisi_core::safe_env::set_var("CXXFLAGS", format!("{} -m32", cxxflags).trim());
                }
                let ldflags = std::env::var("LDFLAGS").unwrap_or_default();
                if !ldflags.contains("-m32") {
                    pisi_core::safe_env::set_var("LDFLAGS", format!("{} -m32", ldflags).trim());
                }
                let existing = std::env::var("PKG_CONFIG_LIBDIR").unwrap_or_default();
                let emul32_pkg = "/usr/lib32/pkgconfig";
                if !existing.contains(emul32_pkg) {
                    if existing.is_empty() {
                        pisi_core::safe_env::set_var("PKG_CONFIG_LIBDIR", emul32_pkg);
                    } else {
                        pisi_core::safe_env::set_var(
                            "PKG_CONFIG_LIBDIR",
                            format!("{}:{}", emul32_pkg, existing),
                        );
                    }
                }
                pisi_core::safe_env::set_var("PKG_CONFIG_SYSROOT_DIR", "/");
            }

            // Her döngü başında çalışma dizinini sıfırla, böylece
            // actions.py içindeki cd işlemleri bir sonraki döngüyü etkilemez
            pisi::cd(&bt_work_dir)?;

            // Kaynak kod hazırlığı ve yamalar.
            // do_prepare true ise her zaman çalışır.
            // do_prepare false ise (--build/--install/--package) sadece
            // kaynak bu build type dizininde henüz yoksa extraction yap.
            let src_wd = format!("{}-{}", self.spec.source.name, &pkg_version);
            let src_dir_check = bt_work_dir.join(&src_wd);
            let source_missing = !src_dir_check.exists() || !src_dir_check.is_dir();

            if do_prepare || source_missing {
                pisi::check_required_tools()?;

                // Önceki çalışma dizinini temizle ve yeniden oluştur
                if bt_work_dir.exists() {
                    fs::remove_dir_all(&bt_work_dir).map_err(|e| e.to_string())?;
                }
                fs::create_dir_all(&bt_work_dir).map_err(|e| e.to_string())?;
                pisi::cd(&bt_work_dir)?;

                spinner.set_message(colorize(&t!("build_step_2"), "brightcyan"));
                for archive in &self.spec.source.archives {
                    let archive_name = std::path::Path::new(&archive.url)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&archive.url);

                    let archive_path = PathBuf::from(archive_name);
                    if !archive_path.exists() {
                        let cached_path = self.config.directories.archives_dir.join(archive_name);
                        if cached_path.exists() {
                            spinner.println(t!("build_source_cache_hit", name = archive_name));
                            fs::copy(&cached_path, &archive_path).map_err(|e| e.to_string())?;
                        } else {
                            spinner.println(t!("build_source_downloading", url = &archive.url));
                            self.download_source(&archive.url, &cached_path)?;
                            fs::copy(&cached_path, &archive_path).map_err(|e| e.to_string())?;
                        }
                    }

                    let (hash_val, hash_type) = archive.get_hash();
                    pisi::verify_archive(archive_name, &hash_val, &hash_type)?;

                    let full_archive_path = bt_work_dir.join(archive_name);
                    if archive.archive_type == "binary" {
                        if let Some(ref target) = archive.target {
                            let target_dir = bt_work_dir.join(target);
                            fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
                            let dest = target_dir.join(archive_name);
                            fs::copy(&full_archive_path, &dest).map_err(|e| e.to_string())?;
                        }
                        spinner.println(colorize(
                            &t!("build_binary_archive", name = archive_name),
                            "yellow",
                        ));
                    } else if let Some(ref target) = archive.target {
                        let target_dir = bt_work_dir.join(target);
                        fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
                        self.unpack_archive_with_progress(&full_archive_path, &target_dir, spinner)?;
                    } else {
                        self.unpack_archive_with_progress(
                            &full_archive_path,
                            &bt_work_dir,
                            spinner,
                        )?;
                    }
                }

                // Tüm kaynak dizinlerini tespit et
                let default_wd = format!("{}-{}", self.spec.source.name, &pkg_version);
                let mut src_dirs: Vec<PathBuf> = Vec::new();
                let default_dir = bt_work_dir.join(&default_wd);
                if default_dir.exists() && default_dir.is_dir() {
                    src_dirs.push(default_dir);
                }
                if src_dirs.is_empty() {
                    for archive in &self.spec.source.archives {
                        let base = std::path::Path::new(&archive.url)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();
                        let mut stem = base.as_str();
                        for ext in &[
                            ".tar.gz", ".tar.xz", ".tar.bz2", ".tar.zst", ".tgz", ".zip", ".gz", ".xz",
                        ] {
                            if let Some(s) = stem.strip_suffix(ext) {
                                stem = s;
                                break;
                            }
                        }
                        let c = bt_work_dir.join(stem);
                        if c.exists() && c.is_dir() {
                            src_dirs.push(c);
                        }
                    }
                }
                if src_dirs.is_empty() {
                    // Flat archive: hiçbir tanınan alt dizin yoksa bt_work_dir'in
                    // kendisinde Makefile/Kbuild olup olmadığına bak.
                    // Bu durumda arşiv tüm dosyaları doğrudan work_dir'e çıkarmıştır.
                    let has_makefile = bt_work_dir.join("Makefile").exists()
                        || bt_work_dir.join("makefile").exists()
                        || bt_work_dir.join("Kbuild").exists()
                        || bt_work_dir.join("GNUmakefile").exists();
                    if has_makefile {
                        src_dirs.push(bt_work_dir.clone());
                    } else if let Ok(entries) = std::fs::read_dir(&bt_work_dir) {
                        for entry in entries.flatten() {
                            let name = entry.file_name();
                            let s = name.to_string_lossy();
                            if entry.path().is_dir() && !s.starts_with('.') {
                                src_dirs.push(entry.path());
                            }
                        }
                    }
                }
                // Yamaları tüm kaynak dizinlerine uygula
                spinner.set_message(colorize(&t!("build_step_3"), "brightcyan"));

                let files_dir = self.work_dir.join("files");
                let patches = self
                    .spec
                    .source
                    .patches
                    .as_ref()
                    .map(|pw| pw.patches.as_slice())
                    .unwrap_or(&[]);
                if !patches.is_empty() {
                    // Yamalar için CWD'yi ilk kaynak dizinine çek (dry-run için)
                    let first_src = src_dirs.first().cloned()
                        .filter(|d| d.exists() && d.is_dir())
                        .unwrap_or_else(|| bt_work_dir.clone());
                    pisi::cd(&first_src)?;

                    let patch_paths: Vec<(String, u8)> = patches.iter().map(|patch| {
                        let patch_compression_exts = ["", ".gz", ".xz", ".bz2", ".zst"];
                        let patch_path = {
                            let mut found = None;
                            for comp_ext in &patch_compression_exts {
                                let candidate_name = format!("{}{}", patch.file, comp_ext);
                                let candidate = files_dir.join(&candidate_name);
                                if candidate.exists() {
                                    found = Some(candidate);
                                    break;
                                }
                            }
                            if let Some(p) = found {
                                p.to_string_lossy().to_string()
                            } else {
                                let mut found = None;
                                for comp_ext in &patch_compression_exts {
                                    let candidate_name = format!("{}{}", patch.file, comp_ext);
                                    let candidate = self.work_dir.join(&candidate_name);
                                    if candidate.exists() {
                                        found = Some(candidate);
                                        break;
                                    }
                                }
                                found
                                    .unwrap_or_else(|| self.work_dir.join(&patch.file))
                                    .to_string_lossy()
                                    .to_string()
                            }
                        };
                        let level = patch.level.unwrap_or(0);
                        (patch_path, level)
                    }).collect();

                    for src_dir in &src_dirs {
                        if src_dir.exists() && src_dir.is_dir() {
                            pisi::cd(src_dir)?;
                            spinner.println(colorize(&format!("Patching: {}", src_dir.display()), "cyan"));
                            for (patch_path, level) in &patch_paths {
                                match pisi::do_patch(patch_path, *level) {
                                    Ok(_) => {
                                        spinner.println(colorize(
                                            &t!("build_patch_applied", patch = patch_path),
                                            "green",
                                        ));
                                    }
                                    Err(e) => {
                                        spinner.println(colorize(
                                            &t!("build_patch_failed", patch = patch_path, error = e),
                                            "brightred",
                                        ));
                                        return Err(e);
                                    }
                                }
                            }
                        }
                    }
                }

                // GNU Config dosyalarını güncelle
                pisi::gnuconfig_update()?;

                // Unpack başarıyla tamamlandı, state'i kaydet
                self.write_state(bt, "unpack");
            }

            // Kaynak tanımlı ortam değişkenlerini uygula
            if let Some(env_vars) = &self.spec.source.environment {
                for env in &env_vars.vars {
                    let val = env.value.as_deref().unwrap_or("");
                    if env.force || std::env::var(&env.name).is_err() {
                        pisi_core::safe_env::set_var(&env.name, val);
                        spinner.println(format!(
                            "  → {}={}", &env.name, val
                        ));
                    }
                }
            }

            // Build/install adımlarını çalıştır
            spinner.set_message(colorize(&t!("build_step_4"), "brightcyan"));

            let bt_pkg = self
                .spec
                .packages
                .iter()
                .find(|p| {
                    let p_bt = p.build_type.as_deref().unwrap_or("");
                    p_bt == bt && !p.actions.steps.is_empty()
                })
                .or_else(|| {
                    self.spec.packages.iter().find(|p| !p.actions.steps.is_empty())
                });

            if let Some(first_pkg) = bt_pkg {
                spinner.println(t!("build_steps_detected"));

                let default_wd = format!("{}-{}", self.spec.source.name, &pkg_version);
                let wd_path = bt_work_dir.join(&default_wd);
                if wd_path.exists() && wd_path.is_dir() {
                    pisi::cd(&wd_path)?;
                } else if let Some(dir) = self.find_source_dir(&bt_work_dir) {
                    pisi::cd(&bt_work_dir.join(&dir))?;
                }

                // AdditionalFile: specdir/files/<file> → CWD/<target>
                let cwd = std::env::current_dir().unwrap_or_else(|_| bt_work_dir.clone());
                if let Some(ref afw) = self.spec.source.additional_files {
                    for af in &afw.files {
                        let candidates = if !af.filename.is_empty() && af.filename != af.target {
                            vec![af.filename.clone(), af.target.clone()]
                        } else {
                            vec![af.target.clone(), af.filename.clone()]
                        };
                        let mut copied = false;
                        for rel in &candidates {
                            let src = self.specdir.join("files").join(rel.trim_start_matches('/'));
                            if src.is_file() {
                                let dst = cwd.join(&af.target);
                                if let Some(p) = dst.parent() {
                                    fs::create_dir_all(p).map_err(|e| e.to_string())?;
                                }
                                if let Err(e) = fs::copy(&src, &dst) {
                                    eprintln!("Warning: could not copy additional file {} -> {}: {}", src.display(), dst.display(), e);
                                }
                                copied = true;
                                break;
                            }
                        }
                        if !copied {
                            eprintln!("Warning: additional file '{}' not found in files/ directory", af.target);
                        }
                    }
                }

                // Autoreconf/Intltool desteği için ACLOCAL_PATH ayarla (varsa koru)
                {
                    let existing = std::env::var("ACLOCAL_PATH").unwrap_or_default();
                    let mut dirs = if existing.is_empty() {
                        "/usr/share/aclocal".to_string()
                    } else if !existing.contains("/usr/share/aclocal") {
                        format!("{}:/usr/share/aclocal", existing)
                    } else {
                        existing
                    };
                    if bt == "emul32" && !dirs.contains("/usr/share/aclocal32") {
                        dirs = format!("{}:/usr/share/aclocal32", dirs);
                    }
                    pisi_core::safe_env::set_var("ACLOCAL_PATH", &dirs);
                }

                // Python PiSi: clean install dir before default build type install
                if bt.is_empty() && do_install {
                    fs::remove_dir_all(&install_root_bt).map_err(|e| e.to_string())?;
                    fs::create_dir_all(&install_root_bt).map_err(|e| e.to_string())?;
                }

                // State dosyasından kaldığımız yeri oku
                let build_done = self.is_state_done(bt, "build");
                let install_done = self.is_state_done(bt, "install");

                let has_step_types = !first_pkg.actions.step_types.is_empty()
                    && first_pkg.actions.step_types.len() == first_pkg.actions.steps.len();

                let mut ran_build = false;
                let mut ran_install = false;

                for (i, step) in first_pkg.actions.steps.iter().enumerate() {
                    // Adım tipini belirle (varsa step_types'dan, yoksa varsayılan olarak build)
                    let is_install_step = has_step_types
                        && first_pkg.actions.step_types[i] == "install";

                    // Aşama kontrolü: do_build/do_install'a göre adımı atla
                    if !has_phase_flags {
                        // Tüm adımları çalıştır (normal build, flag yok)
                    } else if is_install_step {
                        if install_done {
                            spinner.println(colorize(
                                &format!("→ Install adımı zaten tamamlanmış, atlanıyor: {}", step),
                                "yellow",
                            ));
                            ran_install = true;
                            continue;
                        }
                        if !do_install {
                            spinner.println(colorize(
                                &format!("→ Install adımı atlanıyor (--install belirtilmemiş): {}", step),
                                "yellow",
                            ));
                            continue;
                        }
                    } else {
                        // Build adımı (setup, build, check)
                        if build_done {
                            spinner.println(colorize(
                                &format!("→ Build adımı zaten tamamlanmış, atlanıyor: {}", step),
                                "yellow",
                            ));
                            ran_build = true;
                            continue;
                        }
                        if !do_build {
                            spinner.println(colorize(
                                &format!("→ Build adımı atlanıyor (--build belirtilmemiş): {}", step),
                                "yellow",
                            ));
                            continue;
                        }
                    }

                    let mut cmd = step.replace("{jobs}", &pisi::get::make_jobs());
                    cmd = cmd.replace(
                        "{install_root}",
                        install_root_bt.to_str().unwrap_or("install_root"),
                    );
                    cmd = cmd.replace("{source_dir}", ".");
                    cmd = cmd.replace("{KERNEL_RELEASE}", &pisi::cur_kernel());
                    cmd = cmd.replace("{srcVERSION}", &crate::actionsapi::get::src_version());
                    cmd = cmd.replace("{srcNAME}", &crate::actionsapi::get::src_name());
                    cmd = cmd.replace("{srcRELEASE}", &crate::actionsapi::get::src_release());

                    // {VARNAME} → env var genişletmesi (ham değer, shell tırnaklaması yok)
                    let env_re = regex::Regex::new(r"\{(\w+)\}").unwrap();
                    loop {
                        let expanded = env_re.replace_all(&cmd, |caps: &regex::Captures| {
                            let var = caps[1].to_string();
                            match std::env::var(&var) {
                                Ok(val) => val,
                                Err(_) => format!("${{{}}}", var),
                            }
                        }).to_string();
                        if expanded == cmd { break; }
                        cmd = expanded;
                    }

                    spinner.set_message(colorize(&t!("build_step_run_step", cmd = &cmd), "brightcyan"));

                    if let Some(dir) = cmd.strip_prefix("cd ") {
                        pisi::cd(dir.trim())?;
                        continue;
                    }

                    // Autotools ./configure → raw_configure (varsayılan argüman EKLEMEZ,
                    // kullanıcının yazdığı argümanları aynen iletir, actions.py shelltools.system gibi)
                    if let Some(configure_args) = cmd.strip_prefix("./configure") {
                        let args: Vec<String> = configure_args
                            .split_whitespace()
                            .filter(|s| !s.is_empty())
                            .map(|s| s.trim_matches('\'').trim_matches('"').to_string())
                            .collect();
                        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                        crate::actionsapi::raw_configure(&refs)?;
                        continue;
                    }

                    // Generic module.function(args) dispatch for all actionsapi modules
                    let dispatched = Self::dispatch_actionsapi_call(cmd.trim(), &spinner)?;
                    if dispatched {
                        continue;
                    }

                    pisi::run_command(&cmd, &[])?;

                    // Adım başarılı olduysa state güncelle
                    if is_install_step {
                        ran_install = true;
                    } else {
                        ran_build = true;
                    }
                }

                // Faz state'lerini yaz
                if ran_build && !build_done {
                    self.write_state(bt, "build");
                }
                if ran_install && !install_done {
                    self.write_state(bt, "install");
                }
            } else if actions_py_path.exists() {
                spinner.println(t!("build_actions_detected"));
                use pyo3::prelude::*;
                use pyo3::types::PyDict;

                pyo3::Python::with_gil(|py| -> Result<(), String> {
                    let actionsapi_mod =
                        PyModule::new(py, "pisi.actionsapi").map_err(|e| e.to_string())?;
                    crate::python_api::init_actionsapi_module(py, actionsapi_mod)
                        .map_err(|e| e.to_string())?;

                    let sys = py.import("sys").map_err(|e| e.to_string())?;
                    let sys_modules = sys.getattr("modules").map_err(|e| e.to_string())?;
                    sys_modules
                        .set_item("pisi.actionsapi", actionsapi_mod)
                        .map_err(|e| e.to_string())?;

                    let mut code =
                        std::fs::read_to_string(&actions_py_path).map_err(|e| e.to_string())?;

                    if let Ok(ref re) = octal_re {
                        code = re.replace_all(&code, "0o$1").to_string();
                    }

                    let lines: Vec<&str> = code.lines().collect();
                    let mut filtered = Vec::new();
                    for line in &lines {
                        let trimmed = line.trim();
                        if trimmed.starts_with("from pisi.actionsapi") || trimmed.starts_with("import pisi") {
                            continue;
                        }
                        filtered.push(*line);
                    }
                    code = filtered.join("\n");
                    let globals = PyDict::new(py);
                    for key in ["get", "shelltools", "pisitools", "autotools", "cmaketools", "mesontools", "pythonmodules", "python3modules", "qt5", "qt6", "kde6", "sconstools", "cargotools", "perlmodules", "kerneltools", "waftools", "anttools", "npmtools", "gotools", "libtools"] {
                        let sub = actionsapi_mod.getattr(key).map_err(|e| format!("failed to get {}: {}", key, e))?;
                        globals.set_item(key, sub).map_err(|e| format!("failed to set globals.{}: {}", key, e))?;
                    }

                    py.run(
                        r"
import subprocess as _sp
_original_check_output = _sp.check_output
def _patched_check_output(*args, **kw):
    if 'text' not in kw and 'universal_newlines' not in kw:
        kw['text'] = True
    return _original_check_output(*args, **kw)
_sp.check_output = _patched_check_output
",
                        None,
                        None,
                    )
                    .map_err(|e| format!("check_output monte edilemedi: {:?}", e))?;

                    py.run(&code, Some(globals), None)
                        .map_err(|e| {
                            let tb = e.traceback(py);
                            let pye = e.value(py);
                            let tb_str = tb
                                .and_then(|tbo| {
                                    py.import("traceback").ok().and_then(|tb_mod| {
                                        let typ = e.get_type(py);
                                        tb_mod
                                            .call_method1(
                                                "format_exception",
                                                (typ, pye, tbo),
                                            )
                                            .ok()
                                            .and_then(|lines| lines.extract::<Vec<String>>().ok())
                                            .map(|lines| lines.join(""))
                                    })
                                })
                                .unwrap_or_default();
                            let msg = if !tb_str.is_empty() {
                                format!("Python hatası:\n{}", tb_str)
                            } else {
                                format!("{:?}", e)
                            };
                            t!("build_python_error_syntax", error = msg).to_string()
                        })?;

                    if std::env::var("VERBOSE").is_err() {
                        let _ = py.run("import sys; import io; sys.stdout = io.StringIO(); sys.stderr = io.StringIO()", None, None);
                    }

                    let src_dir = if let Ok(Some(abs_wd)) = globals.get_item("absoluteWorkDir") {
                        if let Ok(wd_str) = abs_wd.extract::<String>() {
                            if wd_str.starts_with('/') {
                                PathBuf::from(&wd_str)
                            } else {
                                bt_work_dir.join(&wd_str)
                            }
                        } else {
                            bt_work_dir.clone()
                        }
                    } else if let Ok(Some(our_wd)) = globals.get_item("OurWorkDir") {
                        if let Ok(wd_str) = our_wd.extract::<String>() {
                            bt_work_dir.join(&wd_str)
                        } else {
                            bt_work_dir.clone()
                        }
                    } else if let Ok(Some(workdir)) = globals.get_item("WorkDir") {
                        if let Ok(wd_str) = workdir.extract::<String>() {
                            if wd_str.starts_with('/') {
                                PathBuf::from(&wd_str)
                            } else {
                                bt_work_dir.join(&wd_str)
                            }
                        } else {
                            bt_work_dir.clone()
                        }
                    } else {
                        let default_wd = format!("{}-{}", self.spec.source.name, &pkg_version);
                        let candidate = bt_work_dir.join(&default_wd);
                        if candidate.exists() && candidate.is_dir() {
                            candidate
                        } else {
                            let mut found = None;
                            for archive in &self.spec.source.archives {
                                let base = std::path::Path::new(&archive.url)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("")
                                    .to_string();
                                let mut stem = base.as_str();
                                for ext in &[
                                    ".tar.gz", ".tar.xz", ".tar.bz2", ".tar.zst", ".zip", ".gz",
                                    ".xz",
                                ] {
                                    if let Some(s) = stem.strip_suffix(ext) {
                                        stem = s;
                                        break;
                                    }
                                }
                                let c = bt_work_dir.join(stem);
                                if c.exists() && c.is_dir() {
                                    found = Some(c);
                                    break;
                                }
                            }
                            if let Some(dir) = found {
                                dir
                            } else {
                                let mut fallback = bt_work_dir.clone();
                                if let Ok(entries) = std::fs::read_dir(&bt_work_dir) {
                                    for entry in entries.flatten() {
                                        let name = entry.file_name();
                                        let s = name.to_string_lossy();
                                        if entry.path().is_dir() && !s.starts_with('.') {
                                            fallback = entry.path();
                                            break;
                                        }
                                    }
                                }
                                fallback
                            }
                        }
                    };

                    if !src_dir.exists() {
                        return Err(t!(
                            "build_err_workdir_not_found",
                            path = src_dir.display().to_string()
                        )
                        .to_string());
                    }
                    pisi::cd(&src_dir)?;

                    if let Some(ref afw) = self.spec.source.additional_files {
                        for af in &afw.files {
                            let candidates = if !af.filename.is_empty() && af.filename != af.target {
                                vec![af.filename.clone(), af.target.clone()]
                            } else {
                                vec![af.target.clone(), af.filename.clone()]
                            };
                            let mut copied = false;
                            for rel in &candidates {
                            let src = self.specdir.join("files").join(rel.trim_start_matches('/'));
                            if src.is_file() {
                                let dst = src_dir.join(&af.target);
                                    if let Some(p) = dst.parent() {
                                        fs::create_dir_all(p).map_err(|e| e.to_string())?;
                                    }
                                    if let Err(e) = fs::copy(&src, &dst) {
                                        eprintln!("Warning: could not copy additional file {} -> {}: {}", src.display(), dst.display(), e);
                                    }
                                    copied = true;
                                    break;
                                }
                            }
                            if !copied {
                                eprintln!("Warning: additional file '{}' not found in files/ directory", af.target);
                            }
                        }
                    }

                    if do_build {
                        if !bt.is_empty() {
                            let tmp_path = std::path::Path::new("tmp");
                            if tmp_path.exists() {
                                let _ = std::fs::remove_dir_all(tmp_path);
                            }
                        }
                        if let Ok(Some(setup_fn)) = globals.get_item("setup") {
                            spinner.set_message(colorize(&t!("build_step_4_setup"), "brightcyan"));
                            setup_fn
                                .call0()
                                .map_err(|e| t!("build_python_error_setup", error = e).to_string())?;
                            pisi::cd(&src_dir)?;
                        }

                        if let Ok(Some(build_fn)) = globals.get_item("build") {
                            spinner.set_message(colorize(&t!("build_step_5_build"), "brightcyan"));
                            build_fn
                                .call0()
                                .map_err(|e| t!("build_python_error_build", error = e).to_string())?;
                            pisi::cd(&src_dir)?;
                        }

                        if let Ok(Some(check_fn)) = globals.get_item("check") {
                            spinner.set_message(colorize(&t!("build_step_5_check"), "brightcyan"));
                            check_fn
                                .call0()
                                .map_err(|e| t!("build_python_error_check", error = e).to_string())?;
                            pisi::cd(&src_dir)?;
                        }
                    }

                    if do_install {
                        if let Ok(Some(install_fn)) = globals.get_item("install") {
                            spinner.set_message(colorize(&t!("build_step_6_install"), "brightcyan"));
                            install_fn
                                .call0()
                                .map_err(|e| t!("build_python_error_install", error = e).to_string())?;
                            pisi::cd(&src_dir)?;
                        }
                    }

                    if let Ok(Some(ns)) = globals.get_item("NoStrip") {
                        if let Ok(list) = ns.extract::<Vec<String>>() {
                            no_strip.extend(list);
                        } else if let Ok(s) = ns.extract::<String>() {
                            no_strip.push(s);
                        }
                    }

                    // Ayrıca KDL spec'teki Actions.NoStrip listesini de ekle
                    for pkg in &self.spec.packages {
                        no_strip.extend(pkg.actions.no_strip.clone());
                    }

                    if do_build && !self.is_state_done(bt, "build") {
                        self.write_state(bt, "build");
                    }
                    if do_install && !self.is_state_done(bt, "install") {
                        self.write_state(bt, "install");
                    }

                    Ok(())
                })?;
            } else if !self.spec.packages.is_empty() {
                // AdditionalFile: specdir/files/<file> → CWD/<target>
                let cwd = std::env::current_dir().unwrap_or_else(|_| bt_work_dir.clone());
                if let Some(ref afw) = self.spec.source.additional_files {
                    for af in &afw.files {
                        let candidates = if !af.filename.is_empty() && af.filename != af.target {
                            vec![af.filename.clone(), af.target.clone()]
                        } else {
                            vec![af.target.clone(), af.filename.clone()]
                        };
                        let mut copied = false;
                        for rel in &candidates {
                            let src = self.specdir.join("files").join(rel.trim_start_matches('/'));
                            if src.is_file() {
                                let dst = cwd.join(&af.target);
                                if let Some(p) = dst.parent() {
                                    fs::create_dir_all(p).map_err(|e| e.to_string())?;
                                }
                                if let Err(e) = fs::copy(&src, &dst) {
                                    eprintln!("Warning: could not copy additional file {} -> {}: {}", src.display(), dst.display(), e);
                                }
                                copied = true;
                                break;
                            }
                        }
                        if !copied {
                            eprintln!("Warning: additional file '{}' not found in files/ directory", af.target);
                        }
                    }
                }

                if do_build && !self.is_state_done(bt, "build") {
                    spinner.set_message(colorize(&t!("build_step_4_autotools"), "brightcyan"));
                    pisi::autotools_configure(&[])?;
                    spinner.set_message(colorize(&t!("build_step_5_make"), "brightcyan"));
                    pisi::autotools_make(&[])?;
                    self.write_state(bt, "build");
                } else if do_build && self.is_state_done(bt, "build") {
                    spinner.println(colorize("→ Build aşaması zaten tamamlanmış, atlanıyor.", "yellow"));
                }
                if do_install && !self.is_state_done(bt, "install") {
                    // Python PiSi: clean install dir before default build type install
                    if bt.is_empty() {
                        fs::remove_dir_all(&install_root_bt).map_err(|e| e.to_string())?;
                        fs::create_dir_all(&install_root_bt).map_err(|e| e.to_string())?;
                    }
                    spinner.set_message(colorize(&t!("build_step_6_make_install"), "brightcyan"));
                    pisi::autotools_install(install_root_relative, &[])?;
                    self.write_state(bt, "install");
                } else if do_install && self.is_state_done(bt, "install") {
                    spinner.println(colorize("→ Install aşaması zaten tamamlanmış, atlanıyor.", "yellow"));
                }
            }
            // Build type'a özel post-processing
            if do_install {
                pisi::strip_dir(install_root_bt.to_str().unwrap_or("install"), &no_strip)?;
                pisi::fix_pkgconfig(install_root_bt.to_str().unwrap_or("install"))?;
                pisi::merge_usr_dirs(install_root_bt.to_str().unwrap_or("install"))?;
            }

            // Build type döngüsü sonunda çevre değişkenlerini sıfırla (emul32 sızıntısını önle)
            for (key, val) in &initial_env {
                pisi_core::safe_env::set_var(key, val);
            }
        }

        spinner.println(colorize(&t!("success_build_complete"), "green"));
        Ok(())
    }

    /// Hata durumunda hata detaylarını ve inşa ortamı bilgilerini günlüğe yazar.
    fn write_debug_log(&self, error: &str) {
        let log_path = self
            .options
            .log_path
            .clone()
            .unwrap_or_else(|| self.work_dir.join("pisi-build-error.log"));

        let mut log_content = format!("{}\n", t!("build_debug_log_title"));
        log_content.push_str(&t!("build_debug_log_separator"));
        log_content.push('\n');
        log_content.push_str(&t!("build_debug_log_pkg", name = &self.spec.source.name));
        log_content.push('\n');
        log_content.push_str(&t!(
            "build_debug_log_version",
            version = self
                .spec
                .history
                .as_ref()
                .and_then(|h| h.updates.first())
                .map(|u| u.version.as_str())
                .unwrap_or("?")
        ));
        log_content.push('\n');
        log_content.push_str(&t!("build_debug_log_jobs", jobs = self.options.jobs));
        log_content.push('\n');
        log_content.push_str(&t!(
            "build_debug_log_dir",
            path = self.work_dir.display().to_string()
        ));
        log_content.push('\n');
        log_content.push_str(&t!("build_debug_log_date", date = chrono::Local::now()));
        log_content.push('\n');

        log_content.push_str(&t!("build_debug_log_env"));
        log_content.push('\n');
        log_content.push_str(&format!("MAKEOPTS   : -j{}\n", self.options.jobs));
        log_content.push_str(&format!("MAKEFLAGS  : -j{}\n", self.options.jobs));
        log_content.push_str(&format!("VERBOSE    : {}\n\n", self.options.verbose));

        log_content.push_str(&t!("build_debug_log_error"));
        log_content.push('\n');
        log_content.push_str(error);
        log_content.push('\n');

        let _ = fs::write(&log_path, log_content);
        println!(
            "{}",
            t!("build_failed_log_created", path = log_path.display())
        );
    }

    /// İnşa dizinini ve içindeki tüm dosyaları (sandbox ortamı ve bağımlılıklar) temizler.
    pub fn cleanup(&self) -> Result<(), String> {
        println!("{}", t!("build_cleaning", path = self.work_dir.display()));
        if self.work_dir.exists() {
            fs::remove_dir_all(&self.work_dir)
                .map_err(|e| t!("build_err_cleanup", error = e).to_string())?;
        }
        Ok(())
    }

    /// /etc/pisi/mirrors.conf (ve cwd/mirrors.conf) dosyasındaki URL alias'larını okur
    fn read_url_aliases(&self) -> std::collections::HashMap<String, String> {
        use std::collections::HashMap;
        let mut aliases = HashMap::new();
        let config_paths = ["/etc/pisi/mirrors.conf", "mirrors.conf"];
        for path in &config_paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines() {
                    let line = line.trim();
                    if let Some(rest) = line.strip_prefix("alias ") {
                        let parts: Vec<&str> = rest.split_whitespace().collect();
                        if parts.len() == 2 {
                            aliases.entry(parts[0].to_string()).or_insert(parts[1].to_string());
                        }
                    }
                }
            }
        }
        aliases
    }

    /// Paket adı ve sürümünden GitHub release/archive URL'leri türetir (son çare denemesi)
    fn generate_github_fallbacks(&self, url: &str) -> Vec<String> {
        let mut fallbacks = Vec::new();
        let filename = match std::path::Path::new(url).file_name().and_then(|n| n.to_str()) {
            Some(f) => f,
            None => return fallbacks,
        };
        let pkg_name = &self.spec.source.name;
        let pkg_version = self
            .spec
            .history
            .as_ref()
            .and_then(|h| h.updates.first())
            .map(|u| u.version.as_str())
            .unwrap_or("");
        if pkg_version.is_empty() {
            return fallbacks;
        }
        let version_us = pkg_version.replace('.', "_");
        let pkg_name_upper = pkg_name.to_uppercase();

        // Olası tag formatları
        let tags = vec![
            format!("v{}", pkg_version),
            format!("R_{}", version_us),
            format!("{}-{}", pkg_name, pkg_version),
            pkg_version.to_string(),
            format!("release-{}", pkg_version),
            format!("{}{}", pkg_name_upper, version_us),       // FILE5_46
            format!("{}_{}", pkg_name_upper, version_us),      // FILE_5_46
            format!("REL_{}", version_us),
            format!("{}-{}", pkg_name_upper, pkg_version),
        ];

        // releases/download/{tag}/{filename} (GitHub Releases assets)
        for tag in &tags {
            let gh_url = format!(
                "https://github.com/{}/{}/releases/download/{}/{}",
                pkg_name, pkg_name, tag, filename
            );
            if !fallbacks.contains(&gh_url) {
                fallbacks.push(gh_url);
            }
        }

        // archive/refs/tags/{tag}.tar.gz (git tag arşivi)
        for tag in &tags {
            let gh_url = format!(
                "https://github.com/{}/{}/archive/refs/tags/{}.tar.gz",
                pkg_name, pkg_name, tag
            );
            if !fallbacks.contains(&gh_url) {
                fallbacks.push(gh_url);
            }
        }

        fallbacks
    }

    /// Kaynak arşivi belirtilen konuma indirir.
    fn download_source(&self, url: &str, dest_path: &PathBuf) -> Result<(), String> {
        let mut urls_to_try = Vec::new();

        // FTP URL'lerini HTTPS/HTTP'e çevir (reqwest FTP desteklemez)
        let url = if url.starts_with("ftp://") {
            let https_url = url.replacen("ftp://", "https://", 1);
            urls_to_try.push(https_url.clone());
            let http_url = url.replacen("ftp://", "http://", 1);
            if http_url != https_url {
                urls_to_try.push(http_url);
            }
            https_url
        } else {
            url.to_string()
        };

        // URL alias'larını dene (kullanıcı tanımlı override)
        for (original, replacement) in &self.read_url_aliases() {
            if url.as_str() == original.as_str() {
                urls_to_try.push(replacement.clone());
            }
        }

        if let Some(stripped) = url.strip_prefix("mirrors://") {
            if let Some(slash_idx) = stripped.find('/') {
                let mirror_name = &stripped[..slash_idx];
                let remaining_path = &stripped[slash_idx + 1..];

                // /etc/pisi/mirrors.conf dosyasını oku
                if let Ok(content) = std::fs::read_to_string("/etc/pisi/mirrors.conf") {
                    for line in content.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() == 2 && parts[0] == mirror_name {
                            let base_url = parts[1].trim_end_matches('/');
                            if base_url.starts_with("http://") || base_url.starts_with("https://") {
                                urls_to_try.push(format!("{}/{}", base_url, remaining_path));
                            }
                        }
                    }
                }

                // Python PiSi'deki gibi bilinen projeler için fallback URL'ler
                match mirror_name {
                    name if name == "sourceforge" || name.contains("sourceforge") => {
                        // remaining_path direkt downloads.sourceforge.net'te dene
                        urls_to_try.push(format!(
                            "https://downloads.sourceforge.net/project/{}",
                            remaining_path
                        ));
                        // SourceForge'un kendi download sayfasını da dene (yönlendirme yapabilir)
                        if let Some(slash_idx) = remaining_path.find('/') {
                            let project = &remaining_path[..slash_idx];
                            let file_path = &remaining_path[slash_idx + 1..];
                            urls_to_try.push(format!(
                                "https://sourceforge.net/projects/{}/files/{}/download",
                                project, file_path
                            ));
                        }
                    }
                    "gnu" => {
                        urls_to_try.push(format!("https://ftp.gnu.org/gnu/{}", remaining_path));
                    }
                    "gnome" => {
                        urls_to_try.push(format!(
                            "https://ftp.gnome.org/pub/GNOME/sources/{}",
                            remaining_path
                        ));
                    }
                    "kde" => {
                        urls_to_try.push(format!("https://download.kde.org/{}", remaining_path));
                    }
                    _ => {}
                }

                // Genel çözüm: mirrors:// sonrasını doğrudan https:// ile dene
                urls_to_try.push(format!("https://{}", stripped));
                // sourceforge.net URL'leri için /download sonekini de dene
                if stripped.contains("sourceforge.net") {
                    urls_to_try.push(format!("https://{}/download", stripped));
                }
            }
        } else {
            if url.contains("sourceforge.net/projects/") {
                // sourceforge.net/projects/X/files/Y HTML sayfası döndürür,
                // downloads.sourceforge.net/project/X/Y doğrudan dosyayı döndürür.
                if let Some(path) = url.split("/projects/").nth(1) {
                    let dl_path = path.trim_end_matches('/').replacen("/files/", "/", 1);
                    urls_to_try.push(format!(
                        "https://downloads.sourceforge.net/project/{}",
                        dl_path
                    ));
                }
                if !url.ends_with("/download") {
                    urls_to_try.push(format!("{}/download", url.trim_end_matches('/')));
                }
            } else if url.contains("downloads.sourceforge.net/project/") {
                // https://downloads.sourceforge.net/project/X/Y → https://sourceforge.net/projects/X/files/Y/download
                if let Some(path) = url.split("/project/").nth(1) {
                    if let Some(slash) = path.find('/') {
                        let project = &path[..slash];
                        let file_path = &path[slash + 1..];
                        urls_to_try.push(format!(
                            "https://sourceforge.net/projects/{}/files/{}/download",
                            project, file_path
                        ));
                    }
                }
            } else {
                urls_to_try.push(url.to_string());
            }
        }

        // build.fallback (Python PiSi'deki ctx.config.values.build.fallback)
        let fallback_urls = [
            self.config.build.fallback.trim_end_matches('/'),
            "https://source.pisilinux.org",
        ];
        if let Some(archive_name) = std::path::Path::new(&url).file_name() {
            if let Some(name) = archive_name.to_str() {
                for fb in &fallback_urls {
                    urls_to_try.push(format!("{}/{}", fb, name));
                }
            }
        }

        // Son çare: GitHub release URL kalıplarını dene
        for fb in self.generate_github_fallbacks(&url) {
            if !urls_to_try.contains(&fb) {
                urls_to_try.push(fb);
            }
        }

        if urls_to_try.is_empty() {
            return Err(t!("build_err_no_mirror", url = url).to_string());
        }

        let mut last_error = String::new();
        for try_url in &urls_to_try {
            println!("{}", t!("build_downloading_from", url = try_url));
            match self.download_url(try_url, dest_path) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    println!("{}", t!("build_mirror_failed", url = try_url, error = &e));
                    last_error = e;
                }
            }
        }
        Err(t!("build_err_all_mirrors_failed", error = last_error).to_string())
    }

    fn download_url(&self, url: &str, dest_path: &PathBuf) -> Result<(), String> {
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let mut builder = reqwest::blocking::Client::builder().danger_accept_invalid_certs(true);

        if let Some(ref proxy) = self.config.general.http_proxy {
            builder = builder.proxy(reqwest::Proxy::http(proxy).map_err(|e| e.to_string())?);
        }
        if let Some(ref proxy) = self.config.general.https_proxy {
            builder = builder.proxy(reqwest::Proxy::https(proxy).map_err(|e| e.to_string())?);
        }

        let client = builder
            .user_agent(concat!("pisi/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| e.to_string())?;
        let mut response = client.get(url).send().map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(t!(
                "build_err_download_failed",
                status = response.status().as_u16(),
                url = url
            )
            .to_string());
        }

        // HTML yanıtını reddet (SourceForge vs. hata sayfası döndürebilir)
        if let Some(content_type) = response.headers().get(reqwest::header::CONTENT_TYPE) {
            if let Ok(ct) = content_type.to_str() {
                if ct.contains("text/html") {
                    return Err(t!("build_err_download_failed", status = 200, url = url).to_string());
                }
            }
        }

        let mut file = fs::File::create(dest_path).map_err(|e| e.to_string())?;
        std::io::copy(&mut response, &mut file).map_err(|e| e.to_string())?;

        Ok(())
    }

    fn setup_cross_compiler_env(&self, spinner: &ProgressBar) -> Result<(), String> {
        let target = &self.options.architecture;
        let host_arch = std::env::consts::ARCH;
        spinner.println(t!("build_cross_setup", host = host_arch, target = target));

        if target == "aarch64" || target == "arm64" {
            let triples = [
                "aarch64-linux-gnu",
                "aarch64-unknown-linux-gnu",
                "aarch64-lfs-linux-gnu",
            ];
            let mut found_triple = "aarch64-linux-gnu";
            for triple in &triples {
                let gcc_bin = format!("{}-gcc", triple);
                if crate::actionsapi::get::exist_binary(&gcc_bin) {
                    found_triple = triple;
                    break;
                }
            }

            spinner.println(t!("build_cross_triple_selected", triple = found_triple));

            pisi_core::safe_env::set_var("CHOST", found_triple);
            pisi_core::safe_env::set_var("HOST", found_triple);
            pisi_core::safe_env::set_var("CC", format!("{}-gcc", found_triple));
            pisi_core::safe_env::set_var("CXX", format!("{}-g++", found_triple));
            pisi_core::safe_env::set_var("AR", format!("{}-ar", found_triple));
            pisi_core::safe_env::set_var("AS", format!("{}-as", found_triple));
            pisi_core::safe_env::set_var("LD", format!("{}-ld", found_triple));
            pisi_core::safe_env::set_var("RANLIB", format!("{}-ranlib", found_triple));
            pisi_core::safe_env::set_var("NM", format!("{}-nm", found_triple));
            pisi_core::safe_env::set_var("STRIP", format!("{}-strip", found_triple));

            // Cross pkg-config
            pisi_core::safe_env::set_var("PKG_CONFIG_PATH", "");
            let sysroot = std::env::var("SYSROOT").unwrap_or_else(|_| "/".to_string());
            let pkg_config_libdir = if sysroot == "/" {
                format!("/usr/lib/{}/pkgconfig:/usr/share/pkgconfig", found_triple)
            } else {
                format!(
                    "{}/usr/lib/pkgconfig:{}/usr/share/pkgconfig",
                    sysroot, sysroot
                )
            };
            pisi_core::safe_env::set_var("PKG_CONFIG_LIBDIR", pkg_config_libdir);
            pisi_core::safe_env::set_var("PKG_CONFIG_SYSROOT_DIR", sysroot);

            // Rust/Cargo
            pisi_core::safe_env::set_var("CARGO_BUILD_TARGET", "aarch64-unknown-linux-gnu");
            pisi_core::safe_env::set_var(
                "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER",
                format!("{}-gcc", found_triple),
            );
        } else if target == "x86_64" {
            let triples = [
                "x86_64-linux-gnu",
                "x86_64-pc-linux-gnu",
                "x86_64-lfs-linux-gnu",
            ];
            let mut found_triple = "x86_64-linux-gnu";
            for triple in &triples {
                let gcc_bin = format!("{}-gcc", triple);
                if crate::actionsapi::get::exist_binary(&gcc_bin) {
                    found_triple = triple;
                    break;
                }
            }

            spinner.println(t!("build_cross_triple_selected", triple = found_triple));

            pisi_core::safe_env::set_var("CHOST", found_triple);
            pisi_core::safe_env::set_var("HOST", found_triple);
            pisi_core::safe_env::set_var("CC", format!("{}-gcc", found_triple));
            pisi_core::safe_env::set_var("CXX", format!("{}-g++", found_triple));
            pisi_core::safe_env::set_var("AR", format!("{}-ar", found_triple));
            pisi_core::safe_env::set_var("AS", format!("{}-as", found_triple));
            pisi_core::safe_env::set_var("LD", format!("{}-ld", found_triple));
            pisi_core::safe_env::set_var("RANLIB", format!("{}-ranlib", found_triple));
            pisi_core::safe_env::set_var("NM", format!("{}-nm", found_triple));
            pisi_core::safe_env::set_var("STRIP", format!("{}-strip", found_triple));

            // Cross pkg-config
            pisi_core::safe_env::set_var("PKG_CONFIG_PATH", "");
            let sysroot = std::env::var("SYSROOT").unwrap_or_else(|_| "/".to_string());
            let pkg_config_libdir = if sysroot == "/" {
                format!("/usr/lib/{}/pkgconfig:/usr/share/pkgconfig", found_triple)
            } else {
                format!(
                    "{}/usr/lib/pkgconfig:{}/usr/share/pkgconfig",
                    sysroot, sysroot
                )
            };
            pisi_core::safe_env::set_var("PKG_CONFIG_LIBDIR", pkg_config_libdir);
            pisi_core::safe_env::set_var("PKG_CONFIG_SYSROOT_DIR", sysroot);

            // Rust/Cargo
            pisi_core::safe_env::set_var("CARGO_BUILD_TARGET", "x86_64-unknown-linux-gnu");
            pisi_core::safe_env::set_var(
                "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER",
                format!("{}-gcc", found_triple),
            );
        }

        Ok(())
    }

    fn unpack_archive_with_progress(
        &self,
        archive_path: &std::path::Path,
        dest_dir: &std::path::Path,
        spinner: &ProgressBar,
    ) -> Result<(), String> {


        let file_name = archive_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let ext = archive_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let spinner_style = ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✔️"])
            .template("{spinner:.green} {msg}")
            .unwrap();

        if ext == "zip" {
            use std::fs::File;
            let file = File::open(archive_path).map_err(|e| e.to_string())?;
            let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
            let total_entries = zip.len();

            spinner.set_length(total_entries as u64);
            spinner.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
                    .unwrap()
                    .progress_chars("#>-")
            );
            spinner.set_message(colorize(&t!("build_extracting", name = file_name), "brightcyan"));

            for i in 0..total_entries {
                let mut file = zip.by_index(i).map_err(|e| e.to_string())?;
                let outpath = match file.enclosed_name() {
                    Some(path) => dest_dir.join(path),
                    None => continue,
                };

                if (*file.name()).ends_with('/') {
                    std::fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
                        }
                    }
                    let mut outfile = std::fs::File::create(&outpath).map_err(|e| e.to_string())?;
                    std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
                }

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = file.unix_mode() {
                        std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))
                            .ok();
                    }
                }
                spinner.set_position(i as u64 + 1);
            }

            spinner.set_style(spinner_style);
            spinner.set_position(0);
            spinner.println(colorize(
                &t!("build_archive_extracted", name = file_name),
                "green",
            ));
            spinner.set_message(colorize(&t!("build_step_2"), "brightcyan"));
        } else {
            let archive_path_str = archive_path
                .to_str()
                .ok_or(t!("build_err_invalid_archive_path"))?;
            let dest_str = dest_dir.to_str().ok_or(t!("build_err_invalid_dest_dir"))?;

            spinner.set_message(colorize(&t!("build_extracting", name = file_name), "brightcyan"));

            // Sıkıştırılmış patch/diff dosyası mı? (.patch.gz, .diff.xz vb.)
            let stem = archive_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let is_compressed_patch = stem.ends_with(".patch") || stem.ends_with(".diff");

            if is_compressed_patch {
                let decompress_cmd: &[&str] = match ext {
                    "xz" => &["unxz", "-f", archive_path_str],
                    "bz2" => &["bunzip2", "-f", archive_path_str],
                    "zst" => &["unzstd", "-f", archive_path_str],
                    _ => &["gunzip", "-f", archive_path_str],
                };
                pisi::run_command(decompress_cmd[0], &decompress_cmd[1..])
                    .map_err(|e| t!("build_err_extract_failed", name = file_name, error = e).to_string())?;
            } else {
                // Sırayla her formatı dene (xz → bz2 → gz → auto)
                let extract_cmds: &[&[&str]] = &[
                    &["tar", "-xJf", archive_path_str, "-C", dest_str],
                    &["tar", "-xjf", archive_path_str, "-C", dest_str],
                    &["tar", "-xzf", archive_path_str, "-C", dest_str],
                    &["tar", "-xf", archive_path_str, "-C", dest_str],
                ];

                let mut last_err = String::from("all decompressors failed");
                for cmd in extract_cmds {
                    match pisi::run_command(cmd[0], &cmd[1..]) {
                        Ok(_) => { last_err.clear(); break; }
                        Err(e) => { last_err = e; }
                    }
                }
                if !last_err.is_empty() {
                    return Err(t!("build_err_extract_failed", name = file_name, error = last_err).to_string());
                }
            }
            spinner.println(colorize(
                &t!("build_archive_extracted", name = file_name),
                "green",
            ));
            spinner.set_message(colorize(&t!("build_step_2"), "brightcyan"));
        }

        Ok(())
    }

    /// Extracted kaynak dizinini bul (önce {name}-{version}, sonra arşiv stem, sonra ilk alt dizin)
    fn find_source_dir(&self, bt_work_dir: &PathBuf) -> Option<String> {
        let pkg_version = self
            .spec
            .history
            .as_ref()
            .and_then(|h| h.updates.first())
            .map(|u| u.version.clone())
            .unwrap_or_default();

        let default_wd = format!("{}-{}", self.spec.source.name, &pkg_version);
        let default_dir = bt_work_dir.join(&default_wd);
        if default_dir.exists() && default_dir.is_dir() {
            return Some(default_wd);
        }

        for archive in &self.spec.source.archives {
            let base = std::path::Path::new(&archive.url)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let mut stem = base.as_str();
            for ext in &[".tar.gz", ".tar.xz", ".tar.bz2", ".tar.zst", ".tgz", ".zip", ".gz", ".xz"] {
                if let Some(s) = stem.strip_suffix(ext) {
                    stem = s;
                    break;
                }
            }
            if bt_work_dir.join(stem).exists() && bt_work_dir.join(stem).is_dir() {
                return Some(stem.to_string());
            }
        }

        // Flat archive: bt_work_dir'de Makefile/Kbuild varsa kök dizini kaynak dizinidir
        let has_makefile = bt_work_dir.join("Makefile").exists()
            || bt_work_dir.join("makefile").exists()
            || bt_work_dir.join("Kbuild").exists()
            || bt_work_dir.join("GNUmakefile").exists();
        if has_makefile {
            return Some("".to_string());
        }

        if let Ok(entries) = std::fs::read_dir(bt_work_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let s = name.to_string_lossy().to_string();
                if entry.path().is_dir() && !s.starts_with('.') {
                    return Some(s);
                }
            }
        }

        None
    }

    /// Genel actionsapi dispatch: module.function(args) kalıbını parse edip
    /// ilgili Rust fonksiyonuna yönlendirir.
    fn dispatch_actionsapi_call(cmd: &str, spinner: &indicatif::ProgressBar) -> Result<bool, String> {
        if let Some(paren_start) = cmd.find('(') {
            if let Some(dot_pos) = cmd[..paren_start].rfind('.') {
                let module = cmd[..dot_pos].trim();
                let func = &cmd[dot_pos + 1..paren_start];
                let args_raw = cmd[paren_start + 1..].trim_end_matches(')');
                let args: Vec<String> = args_raw
                    .split(',')
                    .map(|a| a.trim().trim_matches('"').trim_matches('\'').to_string())
                    .filter(|a| !a.is_empty())
                    .collect();
                let idir = pisi::get::install_dir();
                let pkg_name = pisi::get::src_name();
                match module {
                    "pisitools" => Self::dispatch_pisitools(func, &args, &idir, &pkg_name, spinner),
                    "kerneltools" => Self::dispatch_kerneltools(func, &args, &idir),
                    "shelltools" => Self::dispatch_shelltools(func, &args),
                    "autotools" => Self::dispatch_autotools(func, &args, &idir),
                    "cmaketools" => Self::dispatch_cmaketools(func, &args, &idir),
                    "mesontools" => Self::dispatch_mesontools(func, &args, &idir),
                    "pythonmodules" | "python3modules" => {
                        Self::dispatch_pythonmodules(func, &args, &idir)
                    }
                    "sconstools" => Self::dispatch_sconstools(func, &args, &idir),
                    "cargotools" => Self::dispatch_cargotools(func, &args, &idir),
                    "perlmodules" => Self::dispatch_perlmodules(func, &args, &idir),
                    "waftools" => Self::dispatch_waftools(func, &args, &idir),
                    "anttools" => Self::dispatch_anttools(func, &args, &idir),
                    "npmtools" => Self::dispatch_npmtools(func, &args, &idir),
                    "gotools" => Self::dispatch_gotools(func, &args, &idir),
                    "libtools" => Self::dispatch_libtools(func, &args, &idir),
                    "qt5" => Self::dispatch_qt5(func, &args, &idir),
                    "qt6" => Self::dispatch_qt6(func, &args, &idir),
                    "kde6" => Self::dispatch_kde6(func, &args, &idir),
                    "get" => Self::dispatch_get(func, &args),
                    _ => {
                        spinner.println(t!(
                            "build_unknown_module",
                            module = module,
                            cmd = cmd
                        ));
                        return Ok(false);
                    }
                }?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    // ────────── pisitools dispatch ──────────

    fn dispatch_pisitools(
        func: &str,
        args: &[String],
        idir: &str,
        pkg_name: &str,
        spinner: &indicatif::ProgressBar,
    ) -> Result<(), String> {
        match func {
            "dodoc" => {
                for a in args {
                    pisi::dodoc(idir, pkg_name, a).map_err(|e| e.to_string())?;
                }
            }
            "doman" => {
                for a in args {
                    pisi::doman(idir, a).map_err(|e| e.to_string())?;
                }
            }
            "dobin" => {
                for a in args {
                    pisi::dobin(idir, a).map_err(|e| e.to_string())?;
                }
            }
            "dosbin" => {
                for a in args {
                    pisi::dosbin(idir, a).map_err(|e| e.to_string())?;
                }
            }
            "dolib" => {
                for a in args {
                    pisi::dolib(idir, a).map_err(|e| e.to_string())?;
                }
            }
            "dodir" => {
                for a in args {
                    pisi::dodir(idir, a).map_err(|e| e.to_string())?;
                }
            }
            "dosym" if args.len() == 2 => {
                pisi::dosym(idir, &args[0], &args[1]).map_err(|e| e.to_string())?;
            }
            "domove" if args.len() == 2 => {
                pisi::domove(idir, &args[0], &args[1]).map_err(|e| e.to_string())?;
            }
            "remove" => {
                for a in args {
                    let full = format!(
                        "{}/{}",
                        idir.trim_end_matches('/'),
                        a.trim_start_matches('/')
                    );
                    pisi::remove_path(full).map_err(|e| e.to_string())?;
                }
            }
            "dosed" if args.len() >= 2 => {
                let path = &args[0];
                let pattern = &args[1];
                let replace = args.get(2).map(|s| s.as_str()).unwrap_or("");
                let delete_line = args.len() < 3;
                pisi::dosed(path, pattern, replace, delete_line).map_err(|e| e.to_string())?;
            }
            "insinto" if args.len() >= 2 => {
                let dest_dir = &args[0];
                let source = &args[1];
                if args.len() >= 3 {
                    let target_name = &args[2];
                    let full_dest = std::path::Path::new(idir).join(dest_dir.trim_start_matches('/'));
                    std::fs::create_dir_all(&full_dest).map_err(|e| e.to_string())?;
                    let dst = full_dest.join(target_name);
                    let _ = std::fs::copy(source, &dst);
                } else {
                    pisi::insinto(idir, dest_dir, source).map_err(|e| e.to_string())?;
                }
            }
            "removeDir" => {
                for a in args {
                    pisi::remove_dir(idir, a).map_err(|e| e.to_string())?;
                }
            }
            "installHeaders" => {
                for a in args {
                    pisi::install_headers(idir, a).map_err(|e| e.to_string())?;
                }
            }
            "install" if args.len() == 2 => {
                pisi::install(&args[0], &args[1]).map_err(|e| e.to_string())?;
            }
            "rename" if args.len() == 2 => {
                pisi::move_path(&args[0], &args[1]).map_err(|e| e.to_string())?;
            }
            "doexe" if args.len() >= 1 => {
                let dest_dir = args.get(1).map(|s| s.as_str()).unwrap_or("");
                pisi::doexe(idir, &args[0], dest_dir).map_err(|e| e.to_string())?;
            }
            "dolib_a" if args.len() >= 1 => {
                let dest_dir = args.get(1).map(|s| s.as_str()).unwrap_or("");
                pisi::dolib_a(idir, &args[0], dest_dir).map_err(|e| e.to_string())?;
            }
            "dolib_so" if args.len() >= 1 => {
                let dest_dir = args.get(1).map(|s| s.as_str()).unwrap_or("");
                pisi::dolib_so(idir, &args[0], dest_dir).map_err(|e| e.to_string())?;
            }
            "dopixmaps" => {
                for a in args {
                    pisi::dopixmaps(idir, a).map_err(|e| e.to_string())?;
                }
            }
            "dohtml" => {
                for a in args {
                    let s: &str = a;
                    pisi::dohtml(idir, &[s], None).map_err(|e| e.to_string())?;
                }
            }
            "domo" if args.len() == 3 => {
                pisi::domo(idir, &args[0], &args[1], &args[2])
                    .map_err(|e| e.to_string())?;
            }
            "newdoc" if args.len() == 2 => {
                pisi::newdoc(idir, &args[0], &args[1]).map_err(|e| e.to_string())?;
            }
            "newman" if args.len() == 2 => {
                pisi::newman(idir, &args[0], &args[1]).map_err(|e| e.to_string())?;
            }
            other => {
                spinner.println(t!("build_unknown_pisitools", cmd = other));
            }
        }
        Ok(())
    }

    // ────────── kerneltools dispatch ──────────

    fn dispatch_kerneltools(func: &str, args: &[String], _idir: &str) -> Result<(), String> {
        match func {
            "__getSuffix" | "getSuffix" => {
                let _ = pisi::get::src_version();
            }
            "getKernelVersion" => {
                let flavour = args.first().cloned().unwrap_or_else(|| "kernel".to_string());
                let path = format!("/etc/kernel/{}", flavour);
                let _ = std::fs::read_to_string(&path);
            }
            "configure" => {
                let raw_arch = pisi::get::arch().replace("i686", "i386");
                let kernel_arch = raw_arch.replace("x86_64", "x86");
                let config_src = format!("configs/kernel-{}-config", raw_arch);
                std::fs::copy(&config_src, ".config")
                    .map_err(|e| format!("Cannot copy {}: {}", config_src, e))?;
                let version = pisi::get::src_version();
                let extra = Self::kernel_extra_version(&version);
                pisi::dosed("Makefile", "EXTRAVERSION =.*", &format!("EXTRAVERSION = {}", extra), false)?;
                let make_arch = format!("ARCH={}", kernel_arch);
                crate::actionsapi::buildtools::autotools_make(&[&make_arch, "oldconfig"])
                    .map_err(|e| e.to_string())?;
                let _ = crate::actionsapi::buildtools::autotools_make(&[&make_arch, "listnewconfig"]);
            }
            "build" => {
                let raw_arch = pisi::get::arch().replace("i686", "i386");
                let kernel_arch = raw_arch.replace("x86_64", "x86");
                let debug = args.first().and_then(|s| {
                    if s == "true" || s == "1" { Some(true) } else if s == "false" || s == "0" { Some(false) } else { None }
                }).unwrap_or(false);
                let make_arch = format!("ARCH={}", kernel_arch);
                if debug {
                    crate::actionsapi::buildtools::autotools_make(&[&make_arch, "CONFIG_DEBUG_INFO=y"])
                        .map_err(|e| e.to_string())?;
                } else {
                    crate::actionsapi::buildtools::autotools_make(&[&make_arch])
                        .map_err(|e| e.to_string())?;
                }
            }
            "install" => {
                Self::kerneltools_install()?;
            }
            "installHeaders" => {
                Self::kerneltools_install_headers()?;
            }
            "installLibcHeaders" => {
                let excludes: Option<Vec<String>> = if args.is_empty() { None } else { Some(args.to_vec()) };
                Self::kerneltools_install_libc_headers(excludes)?;
            }
            other => {
                return Err(format!("Unknown kerneltools function: {}", other));
            }
        }
        Ok(())
    }

    fn kernel_extra_version(version: &str) -> String {
        let parts: Vec<&str> = version.splitn(4, '.').collect();
        if parts.len() >= 4 {
            format!(".{}", parts[3])
        } else {
            let third = parts.last().unwrap_or(&"");
            if let Some(pos) = third.find(|c: char| c == '_' || c == '-' || c == '~') {
                third[pos..].to_string()
            } else {
                String::new()
            }
        }
    }

    fn kerneltools_install() -> Result<(), String> {
        let suffix = pisi::get::src_version();
        let idir = pisi::get::install_dir();
        let kernel_dir = format!("{}/etc/kernel", idir);
        std::fs::create_dir_all(&kernel_dir).map_err(|e| e.to_string())?;
        std::fs::write(format!("{}/kernel", kernel_dir), &suffix).map_err(|e| e.to_string())?;
        pisi::insinto(&idir, "/boot/", "arch/x86/boot/bzImage").map_err(|e| e.to_string())?;
        let kimg_dest = format!("{}/boot/kernel-{}", idir, suffix);
        std::fs::rename(format!("{}/boot/bzImage", idir), &kimg_dest).map_err(|e| e.to_string())?;
        crate::actionsapi::buildtools::raw_install(&[
            &format!("INSTALL_MOD_PATH={}/", idir),
            "DEPMOD=/bin/true",
            "modules_install",
            "mod-fw=",
        ]).map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(format!("{}/lib/modules/{}/source", idir, suffix));
        let _ = std::fs::remove_file(format!("{}/lib/modules/{}/build", idir, suffix));
        for f in &["Module.symvers", "System.map"] {
            std::fs::copy(f, format!("{}/lib/modules/{}/{}", idir, suffix, f))
                .map_err(|e| e.to_string())?;
        }
        for d in &["extra", "updates"] {
            pisi::dodir(&idir, &format!("/lib/modules/{}/{}", suffix, d))
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn kerneltools_install_headers() -> Result<(), String> {
        let suffix = pisi::get::src_version();
        let idir = pisi::get::install_dir();
        let hdir = format!("usr/src/linux-headers-{}", suffix);
        let dest = format!("{}/{}", idir, hdir);
        pisi::run_command("mkdir", &["-p", &dest]).map_err(|e| e.to_string())?;
        let find_cmd = format!(
            "find . -path './include/*' -prune -o -path './scripts/*' -prune -o -path './Documentation/*' -prune -o \
             -type f \\( -name 'Makefile*' -o -name 'Kconfig*' -o -name 'Kbuild*' -o -name '*.sh' -o -name '*.pl' -o -name '*.lds' \\) \
             -print | cpio -pVd --preserve-modification-time {}",
            dest
        );
        pisi::run_command(&find_cmd, &[]).map_err(|e| e.to_string())?;
        pisi::run_command("cp", &["-a", "include", "scripts", "Documentation", &dest])
            .map_err(|e| e.to_string())?;
        let _ = pisi::run_command(&format!("rm -rf {}/scripts/*.o", dest), &[]);
        let _ = pisi::run_command(&format!("rm -rf {}/scripts/*/*.o", dest), &[]);
        let _ = pisi::run_command(&format!("rm -rf {}/Documentation/DocBook", dest), &[]);
        pisi::run_command(&format!(
            "(find arch -name include -type d -print | xargs -n1 -i: find : -type f) | \
             cpio -pd --preserve-modification-time {}", dest
        ), &[]).map_err(|e| e.to_string())?;
        for f in &["Module.symvers", "System.map", ".config"] {
            let _ = std::fs::copy(f, format!("{}/{}", dest, f));
        }
        pisi::dosym(&idir, &format!("/{}", hdir), &format!("/lib/modules/{}/build", suffix))
            .map_err(|e| e.to_string())?;
        pisi::dosym(&idir, "build", &format!("/lib/modules/{}/source", suffix))
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn kerneltools_install_libc_headers(excludes: Option<Vec<String>>) -> Result<(), String> {
        let idir = pisi::get::install_dir();
        let htmp = format!("{}/tmp-headers", idir);
        let hdir = format!("{}/usr/include", idir);
        let _ = pisi::run_command("rm", &["-rf", &htmp]);
        pisi::run_command("mkdir", &["-p", &htmp, &hdir]).map_err(|e| e.to_string())?;
        let raw_arch = pisi::get::arch().replace("i686", "i386");
        let kernel_arch = raw_arch.replace("x86_64", "x86");
        let o_arg = format!("O={}", htmp);
        let arch_arg = format!("ARCH={}", kernel_arch);
        let hdr_arg = format!("INSTALL_HDR_PATH={}/install", htmp);
        let work_dir = pisi::get::work_dir();
        let _ = pisi::run_command(&format!(
            "cp -Rv {}/linux-*/arch/x86/include/generated {}/arch/x86/include/",
            work_dir, htmp
        ), &[]);
        crate::actionsapi::buildtools::raw_install(&[&o_arg, &arch_arg, &hdr_arg, "headers_install"])
            .map_err(|e| e.to_string())?;
        pisi::run_command(&format!(
            "cd {}/install/include && find . -name '.' -o -name '.*' -prune -o -print | \
             cpio -pVd --preserve-modification-time {}", htmp, hdir
        ), &[]).map_err(|e| e.to_string())?;
        let _ = pisi::run_command("rm", &["-rf", &format!("{}/sound", hdir)]);
        if let Some(exc) = excludes {
            for e in &exc {
                let _ = pisi::run_command("rm", &["-rf", &format!("{}/{}", hdir, e.trim_start_matches('/'))]);
            }
        }
        let _ = pisi::run_command("rm", &["-rf", &htmp]);
        Ok(())
    }

    // ────────── shelltools dispatch ──────────

    fn dispatch_shelltools(func: &str, args: &[String]) -> Result<(), String> {
        match func {
            "cd" => {
                if let Some(path) = args.first() {
                    pisi::cd(path).map_err(|e| e.to_string())?;
                }
            }
            "system" => {
                if let Some(cmd) = args.first() {
                    pisi::run_command(cmd, &[]).map_err(|e| e.to_string())?;
                }
            }
            "copy" | "cp" if args.len() == 2 => {
                std::fs::copy(&args[0], &args[1]).map_err(|e| e.to_string())?;
            }
            "copytree" if args.len() == 2 => {
                Self::cp_r(&args[0], &args[1])?;
            }
            "makedirs" => {
                for a in args {
                    std::fs::create_dir_all(a).map_err(|e| e.to_string())?;
                }
            }
            "unlink" => {
                for a in args {
                    std::fs::remove_file(a).map_err(|e| e.to_string())?;
                }
            }
            "unlinkDir" => {
                for a in args {
                    std::fs::remove_dir_all(a).map_err(|e| e.to_string())?;
                }
            }
            "chmod" if args.len() >= 1 => {
                let mode = args.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0o755);
                std::fs::set_permissions(&args[0], std::fs::Permissions::from_mode(mode))
                    .map_err(|e| e.to_string())?;
            }
            "chown" if args.len() >= 3 => {
                let uid = &args[1];
                let gid = &args[2];
                pisi::run_command("chown", &[&format!("{}:{}", uid, gid), &args[0]])
                    .map_err(|e| e.to_string())?;
            }
            "export" if args.len() == 2 => {
                pisi_core::safe_env::set_var(&args[0], &args[1]);
            }
            "exportFlags" => pisi::export_flags(),
            "echo" if args.len() >= 2 => {
                let content = args[1..].join(" ");
                std::fs::write(&args[0], &content).map_err(|e| e.to_string())?;
            }
            "touch" => {
                for a in args {
                    std::fs::File::create(a).map_err(|e| e.to_string())?;
                }
            }
            "write" if args.len() == 2 => {
                std::fs::write(&args[0], &args[1]).map_err(|e| e.to_string())?;
            }
            "symlink" | "sym" if args.len() == 2 => {
                pisi::symlink(&args[0], &args[1]).map_err(|e| e.to_string())?;
            }
            "move" | "move_" if args.len() == 2 => {
                std::fs::rename(&args[0], &args[1]).map_err(|e| e.to_string())?;
            }
            other => {
                return Err(format!("Unknown shelltools function: {}", other));
            }
        }
        Ok(())
    }

    fn cp_r(src: &str, dst: &str) -> Result<(), String> {
        let src_path = std::path::Path::new(src);
        let dst_path = std::path::Path::new(dst);
        if src_path.is_dir() {
            std::fs::create_dir_all(dst_path).map_err(|e| e.to_string())?;
            for entry in std::fs::read_dir(src_path).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let name = entry.file_name();
                Self::cp_r(
                    &src_path.join(&name).to_string_lossy(),
                    &dst_path.join(&name).to_string_lossy(),
                )?;
            }
        } else if src_path.is_file() {
            std::fs::copy(src_path, dst_path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    // ────────── autotools dispatch ──────────

    fn dispatch_autotools(func: &str, args: &[String], idir: &str) -> Result<(), String> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match func {
            "configure" | "rawConfigure" => {
                crate::actionsapi::buildtools::raw_configure(&refs)
            }
            "make" => {
                crate::actionsapi::buildtools::autotools_make(&refs)
            }
            "install" => {
                crate::actionsapi::buildtools::autotools_install(idir, &refs)
            }
            "rawInstall" => {
                crate::actionsapi::buildtools::raw_install(&refs)
            }
            "aclocal" => crate::actionsapi::buildtools::aclocal(&refs),
            "autoconf" => crate::actionsapi::buildtools::autoconf(&refs),
            "libtoolize" => crate::actionsapi::buildtools::libtoolize(&refs),
            "autoreconf" => crate::actionsapi::buildtools::autoreconf(&refs),
            "automake" => crate::actionsapi::buildtools::automake(&refs),
            "autoheader" => crate::actionsapi::buildtools::autoheader(&refs),
            "fixInfoDir" => pisi::fix_info_dir(idir),
            "gnuconfig_update" => pisi::gnuconfig_update(),
            other => Err(format!("Unknown autotools function: {}", other)),
        }
    }

    // ────────── cmaketools dispatch ──────────

    fn dispatch_cmaketools(func: &str, args: &[String], idir: &str) -> Result<(), String> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match func {
            "configure" => {
                crate::actionsapi::buildtools::cmake_configure_skip_build_dir(&refs)
            }
            "make" => {
                crate::actionsapi::buildtools::autotools_make(&refs)
            }
            "install" => {
                crate::actionsapi::buildtools::autotools_install(idir, &refs)
            }
            "rawInstall" => {
                crate::actionsapi::buildtools::raw_install(&refs)
            }
            other => Err(format!("Unknown cmaketools function: {}", other)),
        }
    }

    // ────────── mesontools dispatch ──────────

    fn dispatch_mesontools(func: &str, args: &[String], idir: &str) -> Result<(), String> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match func {
            "configure" => crate::actionsapi::buildtools::meson_configure(&refs),
            "build" => crate::actionsapi::buildtools::ninja_build(&refs),
            "install" => crate::actionsapi::buildtools::ninja_install(idir, &refs),
            other => Err(format!("Unknown mesontools function: {}", other)),
        }
    }

    // ────────── pythonmodules / python3modules dispatch ──────────

    fn dispatch_pythonmodules(func: &str, args: &[String], idir: &str) -> Result<(), String> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match func {
            "build" => crate::actionsapi::buildtools::python3_setup_build(&refs),
            "install" => crate::actionsapi::buildtools::python3_setup_install(idir, &refs),
            "compile" => crate::actionsapi::buildtools::python_fix_compiled_py(None),
            "configure" => crate::actionsapi::buildtools::python3_setup_configure(&refs),
            other => Err(format!("Unknown pythonmodules function: {}", other)),
        }
    }

    // ────────── sconstools dispatch ──────────

    fn dispatch_sconstools(func: &str, args: &[String], idir: &str) -> Result<(), String> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match func {
            "build" => crate::actionsapi::buildtools::scons_build(&refs),
            "install" => crate::actionsapi::buildtools::scons_install(idir, &refs),
            other => Err(format!("Unknown sconstools function: {}", other)),
        }
    }

    // ────────── cargotools dispatch ──────────

    fn dispatch_cargotools(func: &str, args: &[String], idir: &str) -> Result<(), String> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match func {
            "setup" | "fetch" => crate::actionsapi::buildtools::cargo_fetch(&refs),
            "build" => crate::actionsapi::buildtools::cargo_build(&refs),
            "test" => crate::actionsapi::buildtools::cargo_test(&refs),
            "install" => crate::actionsapi::buildtools::cargo_install(idir, &refs),
            other => Err(format!("Unknown cargotools function: {}", other)),
        }
    }

    // ────────── perlmodules dispatch ──────────

    fn dispatch_perlmodules(func: &str, args: &[String], idir: &str) -> Result<(), String> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match func {
            "configure" => crate::actionsapi::buildtools::perl_makefile_configure(&refs),
            "make" => crate::actionsapi::buildtools::autotools_make(&refs),
            "install" => crate::actionsapi::buildtools::perl_makefile_install(idir, &refs),
            "removePacklist" => crate::actionsapi::buildtools::remove_packlist(idir),
            "removePodfiles" => crate::actionsapi::buildtools::remove_podfiles(idir),
            other => Err(format!("Unknown perlmodules function: {}", other)),
        }
    }

    // ────────── waftools dispatch ──────────

    fn dispatch_waftools(func: &str, args: &[String], idir: &str) -> Result<(), String> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match func {
            "build" => crate::actionsapi::buildtools::waf_build(&refs),
            "install" => crate::actionsapi::buildtools::waf_install(idir, &refs),
            other => Err(format!("Unknown waftools function: {}", other)),
        }
    }

    // ────────── anttools dispatch ──────────

    fn dispatch_anttools(func: &str, args: &[String], idir: &str) -> Result<(), String> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match func {
            "build" => crate::actionsapi::buildtools::ant_build(&refs),
            "install" => crate::actionsapi::buildtools::ant_install(idir, &refs),
            other => Err(format!("Unknown anttools function: {}", other)),
        }
    }

    // ────────── npmtools dispatch ──────────

    fn dispatch_npmtools(func: &str, args: &[String], idir: &str) -> Result<(), String> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match func {
            "build" => crate::actionsapi::buildtools::npm_build(&refs),
            "install" => crate::actionsapi::buildtools::npm_install(idir, &refs),
            other => Err(format!("Unknown npmtools function: {}", other)),
        }
    }

    // ────────── gotools dispatch ──────────

    fn dispatch_gotools(func: &str, args: &[String], idir: &str) -> Result<(), String> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match func {
            "build" => crate::actionsapi::buildtools::go_build(&refs),
            "install" => crate::actionsapi::buildtools::go_install(idir, &refs),
            other => Err(format!("Unknown gotools function: {}", other)),
        }
    }

    // ────────── libtools dispatch ──────────

    fn dispatch_libtools(func: &str, args: &[String], idir: &str) -> Result<(), String> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match func {
            "preplib" => {
                let dir = args.first().map(|s| s.as_str()).unwrap_or("/usr/lib");
                let dir_path = std::path::Path::new(idir).join(dir.trim_start_matches('/'));
                pisi::run_command("ldconfig", &["-n", "-N", dir_path.to_str().unwrap_or(dir)])
                    .map_err(|e| e.to_string())
            }
            "gnuconfig_update" => pisi::gnuconfig_update(),
            "libtoolize" => crate::actionsapi::buildtools::libtoolize(&refs),
            "gen_usr_ldscript" => {
                if let Some(lib) = args.first() {
                    let lib_dir = std::path::Path::new(idir).join("usr/lib");
                    std::fs::create_dir_all(&lib_dir).map_err(|e| e.to_string())?;
                    let path = lib_dir.join(lib);
                    let content = format!(
                        "/* GNU ld script\n\
                         Since Pardus has critical dynamic libraries\n\
                         in /lib, and the static versions in /usr/lib,\n\
                         we need to have a \"fake\" dynamic lib in /usr/lib,\n\
                         otherwise we run into linking problems.\n\
                         */\n\
                         GROUP ( /lib/{} )\n",
                        lib
                    );
                    std::fs::write(&path, &content).map_err(|e| e.to_string())?;
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
                        .map_err(|e| e.to_string())?;
                }
                Ok(())
            }
            other => Err(format!("Unknown libtools function: {}", other)),
        }
    }

    // ────────── qt5 / qt6 / kde6 dispatch ──────────

    fn dispatch_qt5(func: &str, args: &[String], idir: &str) -> Result<(), String> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match func {
            "configure" => crate::actionsapi::buildtools::qt5_configure(&refs),
            "make" => crate::actionsapi::buildtools::autotools_make(&refs),
            "install" => crate::actionsapi::buildtools::autotools_install(idir, &refs),
            other => Err(format!("Unknown qt5 function: {}", other)),
        }
    }

    fn dispatch_qt6(func: &str, args: &[String], idir: &str) -> Result<(), String> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match func {
            "configure" => crate::actionsapi::buildtools::qt6_configure(&refs),
            "make" => crate::actionsapi::buildtools::autotools_make(&refs),
            "install" => crate::actionsapi::buildtools::autotools_install(idir, &refs),
            other => Err(format!("Unknown qt6 function: {}", other)),
        }
    }

    fn dispatch_kde6(func: &str, args: &[String], idir: &str) -> Result<(), String> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match func {
            "configure" => crate::actionsapi::buildtools::kde6_configure(&refs),
            "make" => crate::actionsapi::buildtools::autotools_make(&refs),
            "install" => crate::actionsapi::buildtools::autotools_install(idir, &refs),
            other => Err(format!("Unknown kde6 function: {}", other)),
        }
    }

    // ────────── get dispatch (read-only bilgi fonksiyonları) ──────────

    fn dispatch_get(func: &str, args: &[String]) -> Result<(), String> {
        match func {
            "make_jobs" | "makeJOBS" => { let _ = pisi::make_jobs(); }
            "cflags" | "CFLAGS" => { let _ = pisi::cflags(); }
            "cxxflags" | "CXXFLAGS" => { let _ = pisi::cxxflags(); }
            "ldflags" | "LDFLAGS" => { let _ = pisi::ldflags(); }
            "arch" | "ARCH" => { let _ = pisi::arch(); }
            "host" | "HOST" => { let _ = pisi::host(); }
            "cc" | "CC" => { let _ = pisi::cc(); }
            "cxx" | "CXX" => { let _ = pisi::cxx(); }
            "ar" | "AR" => { let _ = pisi::ar(); }
            "ld" | "LD" => { let _ = pisi::ld(); }
            "ranlib" | "RANLIB" => { let _ = pisi::ranlib(); }
            "nm" | "NM" => { let _ = pisi::nm(); }
            "src_name" | "srcNAME" | "srcName" => { let _ = pisi::src_name(); }
            "src_version" | "srcVERSION" | "srcVersion" => { let _ = pisi::src_version(); }
            "src_release" | "srcRELEASE" | "srcRelease" => { let _ = pisi::src_release(); }
            "install_dir" | "installDIR" | "installDir" => { let _ = pisi::install_dir(); }
            "work_dir" | "workDIR" | "workDir" => { let _ = pisi::work_dir(); }
            "src_dir" | "srcDIR" | "srcDir" => { let _ = pisi::src_dir(); }
            "src_tag" | "srcTAG" | "srcTag" => { let _ = pisi::src_tag(); }
            "pkg_dir" | "pkgDIR" | "pkgDir" => { let _ = pisi::pkg_dir(); }
            "cur_dir" | "curDIR" | "curDir" => { let _ = pisi::cur_dir(); }
            "doc_dir" | "docDIR" | "docDir" => { let _ = pisi::doc_dir(); }
            "man_dir" | "manDIR" | "manDir" => { let _ = pisi::man_dir(); }
            "info_dir" | "infoDIR" | "infoDir" => { let _ = pisi::info_dir(); }
            "data_dir" | "dataDIR" | "dataDir" => { let _ = pisi::data_dir(); }
            "conf_dir" | "confDIR" | "confDir" => { let _ = pisi::conf_dir(); }
            "libexec_dir" | "libexecDIR" | "libexecDir" => { let _ = pisi::libexec_dir(); }
            "kde_dir" | "kdeDIR" | "kdeDir" => { let _ = pisi::kde_dir(); }
            "qt_dir" | "qtDIR" | "qtDir" => { let _ = pisi::qt_dir(); }
            "cur_kernel" | "curKERNEL" | "curKernel" => { let _ = pisi::cur_kernel(); }
            "cur_python" | "curPYTHON" | "curPython" => { let _ = pisi::cur_python(); }
            "cur_perl" | "curPERL" | "curPerl" => { let _ = pisi::cur_perl(); }
            "build_type" | "buildTYPE" | "buildType" => { let _ = pisi::build_type(); }
            "emul32_prefix_dir" | "emul32_prefixDIR" => { let _ = pisi::emul32_prefix_dir(); }
            "env_var" | "ENV" => {
                if let Some(key) = args.first() {
                    let _ = pisi::env_var(key);
                }
            }
            "exist_binary" => {
                if let Some(name) = args.first() {
                    let _ = pisi::exist_binary(name);
                }
            }
            other => {
                let _ = other;
            }
        }
        Ok(())
    }
}
