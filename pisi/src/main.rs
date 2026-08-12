use clap::{CommandFactory, Parser, Subcommand};
use pisi_builder::build::{BuildOptions, PackageBuilder};
use pisi_spec::models::PisiSpec; // Added for Build command
use std::fs;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf; // Artık ana inşa edici bu
pub mod toolchain;

use chrono::NaiveDate;
use pisi_core::{
    builder::PackageBuilder as PackageArchiveManager, colorize, config::Config,
    database::PisiDatabase, installer::Installer, query::QueryManager, repo::Repository, PisiError,
};

// --- Hata Yönetimi ---
type PisiResult<T> = Result<T, PisiError>;

rust_i18n::i18n!("../locales", fallback = "tr");
use rust_i18n::t;

#[derive(Parser, Debug)]
#[command(author, version, about = "Rust-based Pisi Package Manager", long_about = None, disable_help_subcommand = true)]
pub struct PisiCli {
    #[command(subcommand)]
    command: Commands,

    /// Change system root for PiSi commands
    #[arg(short = 'D', long, global = true, value_name = "DIR")]
    destdir: Option<String>,

    /// Assume yes to all yes/no prompts
    #[arg(short = 'y', long, global = true)]
    yes_all: bool,

    /// Detailed output
    #[arg(short = 'v', long, global = true)]
    verbose: bool,

    /// Show debug information
    #[arg(short = 'd', long, global = true)]
    debug: bool,

    /// Disable color in PiSi outputs
    #[arg(short = 'N', long, global = true)]
    no_color: bool,

    /// Keep bandwidth usage below
    /// the specified kilobytes.
    #[arg(short = 'L', long, global = true, value_name = "KILOBYTES")]
    bandwidth_limit: Option<usize>,

    /// Username for repository authentication.
    #[arg(short = 'u', long, global = true, value_name = "USERNAME")]
    username: Option<String>,

    /// Password for repository authentication.
    #[arg(short = 'p', long, global = true, value_name = "PASSWORD")]
    password: Option<String>,

    /// Number of parallel threads for compilation (e.g., 4, j8 or -j16)
    #[arg(short = 'j', long, global = true, value_name = "JOBS")]
    jobs: Option<String>,

    /// Only download packages, do not install
    #[arg(long, global = true)]
    download_only: bool,

    /// Skip SHA1 verification of packages
    #[arg(long, global = true)]
    ignore_check: bool,

    /// File path to write build logs
    #[arg(long, global = true, value_name = "FILE")]
    log_path: Option<PathBuf>,

    /// Build optimization level (e.g., 2, 3, s)
    #[arg(long, global = true, value_name = "LEVEL")]
    opt_level: Option<String>,

    /// Skip post-install configuration (comar) steps
    #[arg(long, global = true)]
    ignore_comar: bool,

    /// Ignore file conflicts
    #[arg(long, global = true)]
    ignore_file_conflict: bool,

    /// Ignore package conflicts
    #[arg(long, global = true)]
    ignore_package_conflict: bool,

    /// Do not take dependency information into account
    #[arg(long, global = true)]
    ignore_dependency: bool,

    /// Bypass safety switch
    #[arg(long, global = true)]
    ignore_safety: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    // --- Yönetim ve Kurulum Komutları ---
    #[command(alias = "it", about = "Install packages")]
    Install {
        /// List of packages to install
        #[arg(required = false)]
        package_names: Vec<String>,

        #[arg(short, long)]
        force: bool,

        /// Install all packages in a specific component
        #[arg(short = 'c', long)]
        component: Option<String>,

        /// Reinstall the package even if it is already installed
        #[arg(long, visible_alias = "rei")]
        reinstall: bool,
    },
    #[command(
        visible_alias = "em",
        about = "Install package and its dependencies from the repository"
    )]
    Emerge { package_names: Vec<String> },
    #[command(
        visible_alias = "emup",
        about = "Perform a bulk update from source packages in the repository"
    )]
    EmergeUp,
    #[command(visible_alias = "rm", about = "Remove PiSi packages")]
    Remove {
        package_name: String,
        /// Remove a Debian package instead of a PiSi package
        #[arg(long)]
        deb: bool,
    },
    #[command(visible_alias = "up", about = "Upgrade system and packages")]
    Upgrade {
        /// List of packages to upgrade
        #[arg(required = false)]
        package_names: Vec<String>,
        /// Only perform integrity and version check, do not execute
        #[arg(short = 'c', long)]
        check_only: bool,
        /// Only report integrity (missing file) errors
        #[arg(long)]
        integrity_only: bool,
        /// Skip integrity (file existence) check
        #[arg(long)]
        no_integrity: bool,
        /// Only upgrade packages belonging to a specific component
        #[arg(long)]
        component: Option<String>,
    },
    #[command(visible_alias = "dt", about = "Create delta packages")]
    Delta {
        /// Path(s) to the old package(s)
        #[arg(required = true)]
        old_packages: Vec<String>,
        /// Path to the new package
        #[arg(required = true)]
        new_package: String,
        /// Output directory
        #[arg(short, long, default_value = ".")]
        output_dir: String,
    },
    #[command(about = "Clean unused locks")]
    Clean,
    #[command(visible_alias = "cp", about = "Configure pending packages")]
    ConfigurePending,
    #[command(visible_alias = "ro", about = "Remove orphaned packages")]
    RemoveOrphaned,
    #[command(visible_alias = "dc", about = "Clear cache files")]
    DeleteCache,
    #[command(
        visible_alias = "tmp",
        about = "Create a new package template directory"
    )]
    Temp,

    // --- Geliştirici Komutları ---
    #[command(visible_alias = "bi", about = "Build a new PiSi package")]
    Build {
        pspec_path: Option<PathBuf>,
        /// Disable sandbox for build (useful in Docker/CI)
        #[arg(long)]
        no_sandbox: bool,
        /// Install build dependencies to host system (useful with --no-sandbox in Docker)
        #[arg(long, short = 'i')]
        install_deps: bool,
        /// Target architecture (e.g., aarch64, arm64, x86_64)
        #[arg(long, short = 't')]
        target: Option<String>,
        /// Only run build phase (setup/build/check)
        #[arg(long)]
        build: bool,
        /// Only run install phase
        #[arg(long)]
        install: bool,
        /// Only run package phase
        #[arg(long)]
        package: bool,
    },
    #[command(about = "Check components.xml structure and sync with directories")]
    CheckComponents {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, short)]
        fix: bool,
        #[arg(long, short)]
        edit: bool,
    },
    #[command(about = "Reset <History> tag in pspec.xml files to first release")]
    ResetHistory {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    #[command(
        name = "toolchain",
        visible_alias = "tc",
        about = "Chroot Toolchain ve Derleme Yönetimi"
    )]
    Toolchain {
        /// Start stable Chroot environment under /mnt/chroot
        #[arg(long)]
        start: bool,
        /// Update, build and install packages in Chroot order
        #[arg(long)]
        update: bool,
    },

    // --- Sorgu ve Listeleme Komutları ---
    #[command(visible_alias = "sr", about = "Search packages")]
    Search {
        query: String,
    },
    #[command(visible_alias = "sf", about = "Search for a file")]
    SearchFile {
        file_path: String,
    },
    #[command(about = "Show package information")]
    Info {
        package_name: String,
    },
    #[command(visible_alias = "bl", about = "Show package owner and release info")]
    Blame {
        package_name: String,
        #[arg(short, long)]
        release: Option<u32>,
        #[arg(short, long)]
        all: bool,
    },
    #[command(visible_alias = "li", about = "Print a list of all installed packages")]
    ListInstalled {
    },
    #[command(visible_alias = "lo", about = "List orphaned packages")]
    ListOrphaned {
    },
    #[command(visible_alias = "ls", about = "List available sources")]
    ListSources {
    },
    #[command(visible_alias = "lp", about = "List pending packages")]
    ListPending {
    },
    #[command(visible_alias = "lc", about = "List components")]
    ListComponents {
    },
    #[command(visible_alias = "la", about = "List packages in repositories")]
    ListAvailable {
    },
    #[command(
        visible_alias = "ln",
        about = "List the newest packages in repositories"
    )]
    ListNewest {
        /// Number of packages to list
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
    #[command(visible_alias = "lu", about = "List upgradable packages")]
    ListUpgrades {
    },
    #[command(visible_alias = "rdb", about = "Rebuild databases")]
    RebuildDb,
    #[command(
        visible_alias = "ci",
        about = "Check the integrity of installed packages"
    )]
    CheckInstall {
        /// List of packages to check (checks all if left blank)
        package_names: Vec<String>,
        /// Automatically reinstall corrupted packages
        #[arg(short, long)]
        reinstall: bool,
    },
    #[command(
        visible_alias = "cr",
        about = "Check repository health and constraints (e.g. circular dependencies)"
    )]
    CheckRepo {
        #[arg(long)]
        circular: bool,
    },
    #[command(
        visible_alias = "rdiff",
        about = "Compare two repository indices (e.g., source vs binary)"
    )]
    RepoDiff {
        source_index: String,
        binary_index: String,
    },
    #[command(visible_alias = "lf", about = "List files belonging to the package")]
    ListFiles {
        /// Paket adı veya .pisi dosyası yolu
        package_name: String,
    },
    #[command(visible_alias = "hs", about = "History of PiSi operations")]
    History {
        #[arg(short = 't', long)]
        trace_id: Option<u64>,
        /// Başlangıç tarihi (YYYY-MM-DD)
        #[arg(long)]
        from: Option<String>,
        /// Bitiş tarihi (YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,
    },
    #[command(alias = "?", about = "Display help about given commands")]
    Help {
        /// Yardım alınacak komut adı
        command: Option<String>,
    },
    #[command(about = "Draw a graph of package relationships")]
    Graph {
        /// Başlangıç paketleri (boş bırakılırsa tümü taranır)
        package_names: Vec<String>,
        #[arg(short, long, help = "Kurulu paketlerin grafiğini çıkar")]
        installed: bool,
        #[arg(short, long, help = "Ters bağımlılık grafiği çiz")]
        reverse: bool,
        #[arg(short, long, default_value = "pgraph.dot")]
        output: String,
    },
    #[command(
        visible_alias = "ix",
        about = "Create a catalog of PiSi files in the given directory"
    )]
    Index {
        /// Pisi dosyalarının bulunduğu dizin
        #[arg(default_value = ".")]
        source_dir: String,
        /// Oluşturulacak indeks dosyası adı
        #[arg(short, long, default_value = "pisi-index.xml")]
        output: String,
    },
    #[command(visible_alias = "fc", about = "Download package(s)")]
    Fetch {
        /// Packages to downloadin listesi
        #[arg(required = true)]
        package_names: Vec<String>,
        /// İndirilecek hedef dizin
        #[arg(short, long, default_value = ".")]
        output_dir: String,
        /// Bağımlılıkları ile birlikte indir
        #[arg(long)]
        runtime_deps: bool,
    },
    #[command(visible_alias = "rb", about = "Rollback system to a specific Trace ID")]
    Rollback { trace_id: u64 },

    #[command(
        visible_alias = "bd",
        about = "Backup PiSi database to a directory"
    )]
    BackupDb {
        /// Target backup directory
        #[arg(default_value = "/var/backup/pisi")]
        directory: String,
    },

    #[command(
        visible_alias = "rd",
        about = "Restore PiSi database from a backup directory"
    )]
    RestoreDb {
        /// Source backup directory
        #[arg(default_value = "/var/backup/pisi")]
        directory: String,
    },

    #[command(
        visible_alias = "vdb",
        about = "Verify database integrity"
    )]
    VerifyDb,

    // --- Depo Yönetimi Komutları ---
    #[command(visible_alias = "ur", about = "Update repository databases")]
    UpdateRepo,
    #[command(visible_alias = "ar", about = "Add repository")]
    AddRepo { name: String, url: String },
    #[command(visible_alias = "rr", about = "Remove repositories")]
    RemoveRepo { name: String },
    #[command(visible_alias = "er", about = "Enable repository")]
    EnableRepo { name: String },
    #[command(visible_alias = "dr", about = "Disable repository")]
    DisableRepo { name: String },
    #[command(visible_alias = "lr", about = "List repositories")]
    ListRepo {
    },

    // Sistem Testi
    #[command(about = "Database operations test")]
    DbTest,
    #[command(about = "Show detailed version info")]
    Version,
}

fn localize_command(mut cmd: clap::Command) -> clap::Command {
    cmd = cmd.about(t!("cli_help_about"));

    // Global Arguments — güvenli şekilde (argüman yoksa atla)
    let global_args: Vec<(&str, String)> = vec![
        ("destdir", t!("cli_help_destdir").to_string()),
        ("yes_all", t!("cli_help_yes_all").to_string()),
        ("verbose", t!("cli_help_verbose").to_string()),
        ("debug", t!("cli_help_debug").to_string()),
        ("no_color", t!("cli_help_no_color").to_string()),
        ("bandwidth_limit", t!("cli_help_bandwidth").to_string()),
        ("username", t!("cli_help_username").to_string()),
        ("password", t!("cli_help_password").to_string()),
        ("jobs", t!("cli_help_jobs").to_string()),
        ("download_only", t!("cli_help_download_only").to_string()),
        ("ignore_check", t!("cli_help_ignore_check").to_string()),
        ("log_path", t!("cli_help_log_path").to_string()),
        ("opt_level", t!("cli_help_opt_level").to_string()),
        ("ignore_comar", t!("cli_help_ignore_comar").to_string()),
        (
            "ignore_file_conflict",
            t!("cli_help_ignore_file_conflict").to_string(),
        ),
        (
            "ignore_package_conflict",
            t!("cli_help_ignore_package_conflict").to_string(),
        ),
        (
            "ignore_dependency",
            t!("cli_help_ignore_dependency").to_string(),
        ),
        ("ignore_safety", t!("cli_help_ignore_safety").to_string()),
    ];
    for (arg_name, help_text) in global_args {
        if cmd.get_arguments().any(|a| a.get_id() == arg_name) {
            cmd = cmd.mut_arg(arg_name, |a| a.help(help_text.clone()));
        }
    }

    // Subcommands
    let subcommands = [
        (
            "install",
            "cli_cmd_install_about",
            vec![
                ("package_names", "cli_cmd_install_pkg_names"),
                ("component", "cli_cmd_install_component"),
                ("reinstall", "cli_cmd_install_reinstall"),
            ],
        ),
        ("emerge", "cli_cmd_emerge_about", vec![]),
        ("emerge-up", "cli_cmd_emergeup_about", vec![]),
        ("remove", "cli_cmd_remove_about", vec![]),
        (
            "upgrade",
            "cli_cmd_upgrade_about",
            vec![
                ("package_names", "cli_cmd_upgrade_pkg_names"),
                ("check-only", "cli_cmd_upgrade_check_only"),
                ("integrity-only", "cli_cmd_upgrade_integrity_only"),
                ("no-integrity", "cli_cmd_upgrade_no_integrity"),
                ("component", "cli_cmd_upgrade_component"),
            ],
        ),
        (
            "delta",
            "cli_cmd_delta_about",
            vec![
                ("old_packages", "cli_cmd_delta_old_pkgs"),
                ("new_package", "cli_cmd_delta_new_pkg"),
                ("output-dir", "cli_cmd_delta_output_dir"),
            ],
        ),
        ("clean", "cli_cmd_cleanup_about", vec![]),
        ("configure-pending", "cli_cmd_conf_pending_about", vec![]),
        ("remove-orphaned", "cli_cmd_rem_orphaned_about", vec![]),
        ("delete-cache", "cli_cmd_del_cache_about", vec![]),
        ("temp", "cli_cmd_temp_about", vec![]),
        (
            "build",
            "cli_cmd_build_about",
            vec![("no-sandbox", "cli_cmd_build_no_sandbox")],
        ),
        (
            "search",
            "cli_cmd_search_about",
            vec![],
        ),
        (
            "search-file",
            "cli_cmd_sf_about",
            vec![],
        ),
        (
            "info",
            "cli_cmd_info_about",
            vec![],
        ),
        ("blame", "cli_cmd_blame_about", vec![]),
        (
            "list-installed",
            "cli_cmd_li_about",
            vec![],
        ),
        (
            "list-orphaned",
            "cli_cmd_lo_about",
            vec![],
        ),
        (
            "list-sources",
            "cli_cmd_ls_about",
            vec![],
        ),
        (
            "list-pending",
            "cli_cmd_lp_about",
            vec![],
        ),
        (
            "list-components",
            "cli_cmd_lc_about",
            vec![],
        ),
        (
            "list-available",
            "cli_cmd_la_about",
            vec![],
        ),
        (
            "list-newest",
            "cli_cmd_ln_about",
            vec![
                ("limit", "cli_cmd_ln_limit"),
            ],
        ),
        (
            "list-upgrades",
            "cli_cmd_lu_about",
            vec![],
        ),
        ("rebuild-db", "cli_cmd_rdb_about", vec![]),
        (
            "check-install",
            "cli_cmd_ci_about",
            vec![
                ("package_names", "cli_cmd_ci_pkg_names"),
                ("reinstall", "cli_cmd_ci_reinstall"),
            ],
        ),
        (
            "list-files",
            "cli_cmd_lf_about",
            vec![("package_name", "cli_cmd_lf_pkg_name")],
        ),
        (
            "history",
            "cli_cmd_hs_about",
            vec![
                ("from", "cli_cmd_hs_from"),
                ("to", "cli_cmd_hs_to"),
            ],
        ),
        (
            "help",
            "cli_cmd_help_about",
            vec![("command", "cli_cmd_help_cmd")],
        ),
        (
            "graph",
            "cli_cmd_graph_about",
            vec![
                ("package_names", "cli_cmd_graph_pkgs"),
                ("installed", "cli_cmd_graph_inst"),
                ("reverse", "cli_cmd_graph_rev"),
            ],
        ),
        (
            "index",
            "cli_cmd_ix_about",
            vec![
                ("source_dir", "cli_cmd_ix_src"),
                ("output", "cli_cmd_ix_out"),
            ],
        ),
        (
            "fetch",
            "cli_cmd_fc_about",
            vec![
                ("package_names", "cli_cmd_fc_pkgs"),
                ("output-dir", "cli_cmd_fc_out"),
                ("runtime-deps", "cli_cmd_fc_deps"),
            ],
        ),
        ("rollback", "cli_cmd_rb_about", vec![]),
        ("update-repo", "cli_cmd_ur_about", vec![]),
        ("add-repo", "cli_cmd_ar_about", vec![]),
        ("remove-repo", "cli_cmd_rr_about", vec![]),
        ("enable-repo", "cli_cmd_er_about", vec![]),
        ("disable-repo", "cli_cmd_dr_about", vec![]),
        (
            "list-repo",
            "cli_cmd_lr_about",
            vec![],
        ),
        ("db-test", "cli_cmd_dbtest_about", vec![]),
        ("version", "cli_cmd_ver_about", vec![]),
    ];

    for (name, about_key, args) in subcommands {
        cmd = cmd.mut_subcommand(name, |mut sub| {
            sub = sub.about(t!(about_key));
            for (arg_name, arg_key) in args {
                if sub.get_arguments().any(|a| a.get_id() == arg_name) {
                    sub = sub.mut_arg(arg_name, |a| a.help(t!(arg_key)));
                }
            }
            sub
        });
    }

    cmd
}

// ✅ Root yetki kontrolü fonksiyonu
fn check_root() {
    if nix::unistd::geteuid().as_raw() != 0 {
        eprintln!("{}", t!("error_root_required"));
        std::process::exit(1);
    }
}

fn copy_dir_all(
    src: impl AsRef<std::path::Path>,
    dst: impl AsRef<std::path::Path>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn main() -> PisiResult<()> {
    if let Ok(lang) = std::env::var("LC_ALL").or_else(|_| std::env::var("LANG")) {
        if lang.to_lowercase().starts_with("tr") {
            rust_i18n::set_locale("tr");
        } else if lang.to_lowercase().starts_with("en") {
            rust_i18n::set_locale("en");
        }
    }
    // 1. Komut satırı argümanlarını ve CLI yapısını hazırla
    let mut cmd = PisiCli::command();
    cmd = localize_command(cmd);
    let matches = cmd.get_matches();
    let cli = <PisiCli as clap::FromArgMatches>::from_arg_matches(&matches)
        .map_err(|e| PisiError::RuntimeError(e.to_string()))?;

    if cli.no_color {
        pisi_core::safe_env::set_var("NO_COLOR", "1");
    }
    let yes_all = cli.yes_all;
    let download_only = cli.download_only;
    let ignore_check = cli.ignore_check;
    let config = Config::load(cli.destdir.clone().map(PathBuf::from));

    let auth = cli
        .username
        .clone()
        .and_then(|u| cli.password.clone().map(|p| (u, p)));

    // Paralel derleme ayarını (jobs) çevre değişkenine aktar
    if let Some(j) = &cli.jobs {
        let j_val = j.to_lowercase().replace(['j', '-'], "").trim().to_string();
        if !j_val.is_empty() && j_val.chars().all(|c| c.is_ascii_digit()) {
            // Alt süreçlerin ve actionsapi'nin okuyabilmesi için MAKEOPTS set edilir
            pisi_core::safe_env::set_var("MAKEOPTS", format!("-j{}", j_val));
        }
    }

    // --- YETKİ GEREKTİRMEYEN VE DB KULLANMAYAN KOMUTLAR ---
    match &cli.command {
        Commands::Help { command } => {
            return handle_help(command.clone());
        }
        Commands::Version => {
            return handle_version();
        }
        Commands::Temp => {
            return handle_temp();
        }
        Commands::CheckComponents { path, fix, edit } => {
            if *edit {
                pisi_builder::components::edit_components(path)?;
            } else {
                pisi_builder::components::check_components(path, *fix)?;
            }
            return Ok(());
        }
        Commands::ResetHistory { path } => {
            pisi_builder::reset_history::reset_history(path)?;
            return Ok(());
        }
        _ => {}
    }

    let requires_lock = match &cli.command {
        Commands::Install { .. }
        | Commands::Emerge { .. }
        | Commands::EmergeUp
        | Commands::Remove { .. }
        | Commands::Delta { .. }
        | Commands::Clean
        | Commands::ConfigurePending
        | Commands::RemoveOrphaned
        | Commands::DeleteCache
        | Commands::Build { .. }
        | Commands::RebuildDb
        | Commands::Rollback { .. }
        | Commands::UpdateRepo
        | Commands::AddRepo { .. }
        | Commands::RemoveRepo { .. }
        | Commands::EnableRepo { .. }
        | Commands::DisableRepo { .. } => {
            check_root();
            true
        }
        _ => false,
    };

    // --- SİSTEM KİLİDİ (Global Lock) ---
    let _lock = if requires_lock {
        Some(acquire_lock(&config)?)
    } else {
        None
    };

    let bandwidth_limit = cli.bandwidth_limit;
    let mut db_path = config.directories.lib_dir.join("db");

    // Read-only mod simulasyonu: Kilit gerekmiyorsa ve root değilsek veritabanını geçici dizine kopyalayarak aç
    if !requires_lock && nix::unistd::geteuid().as_raw() != 0 {
        let temp_db_dir =
            std::env::temp_dir().join(format!("pisi-db-readonly-{}", nix::unistd::getuid().as_raw()));
        if temp_db_dir.exists() {
            let _ = std::fs::remove_dir_all(&temp_db_dir);
        }
        if let Err(e) = copy_dir_all(&db_path, &temp_db_dir) {
            eprintln!("{}", t!("main_warning_readonly_db_fail", error = e));
        } else {
            db_path = temp_db_dir;
        }
    }

    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(PisiError::IoError)?;
        }
    }

    let db = PisiDatabase::open(db_path)?;
    let trace_id = db.get_next_trace_id()?;
    let installer = Installer::new(db.clone(), config.clone());
    let repo_manager = Repository::new(db.clone(), config.clone());
    let archive_tools = PackageArchiveManager::new(db.clone());
    let query_manager = QueryManager::new(db.clone(), config.clone());

    // 2. Yetki Kontrolü Gerektiren Komutlar Listesi
    // Info, List, Search gibi komutlar root istemez.
    match &cli.command {
        Commands::Install { .. } |
        Commands::Emerge { .. } |
        Commands::EmergeUp |
        Commands::Remove { .. } |
        Commands::Delta { .. } |
        Commands::Clean |
        Commands::ConfigurePending |
        Commands::RemoveOrphaned |
        Commands::DeleteCache |
        Commands::Build { .. } | // Build command now requires root for actual build
        Commands::RebuildDb |
        Commands::Rollback { .. } |
        Commands::UpdateRepo |
        Commands::AddRepo { .. } |
        Commands::RemoveRepo { .. } |
        Commands::EnableRepo { .. } |
        Commands::DisableRepo { .. } => {
            check_root();
        },
        Commands::Upgrade { check_only, .. } => {
            if !*check_only {
                check_root();
            }
        },
        Commands::CheckInstall { reinstall, .. } => {
            if *reinstall {
                check_root();
            }
        },
        Commands::History { trace_id, .. } => {
            // Eğer bir ID varsa (rollback yapacaksa) root kontrolü yap
            if trace_id.is_some() {
                check_root();
            }
            // NOT: Burada handle_history_list_only ÇAĞIRILMAMALI!
        },
        _ => {}
    }

    // 3. Komut İşleme Mantığı
    let jobs_val = cli.jobs.clone();
    let verbose_val = cli.verbose;
    let debug_val = cli.debug;
    let log_path_val = cli.log_path.clone();
    let opt_level_val = cli.opt_level.clone();
    let ignore_comar_val = cli.ignore_comar;
    let ignore_file_conflict_val = cli.ignore_file_conflict;
    let ignore_package_conflict_val = cli.ignore_package_conflict;
    let ignore_dependency_val = cli.ignore_dependency;

    match cli.command {
        // --- ÇOKLU PAKET DESTEĞİ (Emerge/Install) ---
        Commands::Install {
            mut package_names,
            force,
            component,
            reinstall,
        } => {
            if let Some(comp_name) = component {
                let comp_pkgs = query_manager.get_packages_for_component(&comp_name)?;
                if comp_pkgs.is_empty() {
                    eprintln!("{}", t!("main_error_component_not_found", name = comp_name));
                    return Ok(());
                }
                println!(
                    "{}",
                    t!(
                        "main_component_pkgs_added",
                        name = comp_name,
                        count = comp_pkgs.len()
                    )
                );
                package_names.extend(comp_pkgs);
            }
            if package_names.is_empty() {
                eprintln!("{}", t!("main_error_no_packages"));
                return Ok(());
            }

            // Debian paketleri için sorumluluk onayı
            let has_deb = package_names.iter().any(|n| n.ends_with(".deb"));
            if has_deb {
                println!();
                println!("{}", t!("deb_warning_title"));
                println!("{}", "─".repeat(55));
                println!("{}", t!("deb_warning_desc1"));
                println!("{}", t!("deb_warning_desc2"));
                println!();
                println!("{}", t!("deb_warning_desc3"));
                println!();
                println!("{}", t!("deb_warning_bullet1"));
                println!("{}", t!("deb_warning_bullet2"));
                println!("{}", t!("deb_warning_bullet3"));
                println!("{}", t!("deb_warning_bullet4"));
                println!();
                println!("{}", t!("deb_warning_responsibility"));
                println!("{}", "─".repeat(55));
                print!("{}", t!("deb_warning_prompt"));
                use std::io::Write;
                std::io::stdout().flush().unwrap();
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let input = input.trim().to_lowercase();
                if input != "e" && input != "y" {
                    println!("{}", t!("deb_warning_cancelled"));
                    return Ok(());
                }
                println!();
            }

            installer.perform_install(
                package_names.clone(),
                trace_id,
                force,
                yes_all,
                bandwidth_limit,
                auth,
                download_only,
                ignore_check,
                cli.ignore_comar,
                cli.ignore_file_conflict,
                cli.ignore_package_conflict,
                reinstall,
                cli.ignore_dependency,
                None,
            )?
        }
        Commands::Emerge { package_names } => {
            handle_emerge(
                package_names,
                trace_id,
                yes_all,
                bandwidth_limit,
                auth.clone(),
                download_only,
                ignore_check,
                jobs_val,
                verbose_val,
                debug_val,
                log_path_val,
                opt_level_val,
                ignore_comar_val,
                ignore_file_conflict_val,
                ignore_package_conflict_val,
                ignore_dependency_val,
                &config,
                &db,
                &installer,
                &repo_manager,
            )?;
        }
        Commands::EmergeUp => {
            repo_manager.perform_update(trace_id)?;

            let installed = db.list_installed_packages()?;
            let mut emerge_list = Vec::new();

            println!("{}", t!("emerge_searching"));
            for pkg in installed {
                if let Ok(Some(available)) = db.get_available_package(&pkg.name) {
                    if available.latest_version() != pkg.version || available.release > pkg.release
                    {
                        emerge_list.push(pkg.name);
                    }
                }
            }
            if emerge_list.is_empty() {
                println!("{}", t!("emerge_no_packages"));
            } else {
                println!("{}", t!("emerge_count", count = emerge_list.len()));
                handle_emerge(
                    emerge_list,
                    trace_id,
                    yes_all,
                    bandwidth_limit,
                    auth.clone(),
                    download_only,
                    ignore_check,
                    jobs_val,
                    verbose_val,
                    debug_val,
                    log_path_val,
                    opt_level_val,
                    ignore_comar_val,
                    ignore_file_conflict_val,
                    ignore_package_conflict_val,
                    ignore_dependency_val,
                    &config,
                    &db,
                    &installer,
                    &repo_manager,
                )?;
            }
        }
        Commands::Fetch {
            package_names,
            output_dir,
            runtime_deps,
        } => {
            let out_path = PathBuf::from(output_dir);
            repo_manager.perform_fetch(
                package_names,
                out_path,
                runtime_deps,
                bandwidth_limit,
                auth,
            )?;
        }

        Commands::Remove { mut package_name, deb } => {
            if package_name.ends_with(".deb") {
                if let Ok(pkg_data) = pisi_core::packager::Packager::read_package(&package_name) {
                    package_name = pkg_data.metadata.name;
                } else if let Some(filename) = std::path::Path::new(&package_name).file_name() {
                    let filename_str = filename.to_string_lossy().to_string();
                    if let Some(first_part) = filename_str.split('_').next() {
                        package_name = first_part.to_string();
                    }
                }
            }

            // Eğer --deb bayrağı verildiyse veya veritabanındaki paket_hash değeri 'deb-package' ise
            // (DebManager::install_deb bu değeri package_hash olarak yazar)
            let is_deb = deb || db.get_installed_package(&package_name)
                .ok()
                .flatten()
                .map(|p| p.package_hash == "deb-package")
                .unwrap_or(false);

            if is_deb {
                println!("{}", t!("deb_remove_msg", package = package_name));
            }

            installer.perform_remove(
                package_name,
                trace_id,
                yes_all,
                cli.ignore_comar,
                cli.ignore_dependency,
                cli.ignore_safety,
                None,
            )?
        },
        Commands::Upgrade {
            package_names,
            check_only,
            integrity_only,
            component,
            no_integrity,
        } => installer.perform_upgrade(
            package_names,
            trace_id,
            yes_all,
            check_only,
            integrity_only,
            component,
            no_integrity,
            bandwidth_limit,
            auth,
            download_only,
            ignore_check,
            cli.ignore_comar,
            cli.ignore_file_conflict,
            cli.ignore_package_conflict,
            cli.ignore_dependency,
            None,
        )?,
        Commands::Delta {
            old_packages,
            new_package,
            output_dir,
        } => archive_tools.perform_delta(old_packages, new_package, output_dir)?,
        Commands::Clean => repo_manager.perform_clean()?,
        Commands::ConfigurePending => installer.perform_configure_pending()?,
        Commands::RemoveOrphaned => installer.perform_remove_orphaned(trace_id)?,
        Commands::DeleteCache => repo_manager.perform_delete_cache()?,
        Commands::Toolchain { start, update } => {
            if start {
                crate::toolchain::perform_toolchain_start(trace_id)?;
            } else if update {
                crate::toolchain::perform_toolchain_update(trace_id)?;
            } else {
                return Err(PisiError::RuntimeError(
                    "Lütfen --start veya --update seçeneklerinden birini belirtin.".to_string(),
                ));
            }
        }
        Commands::Build { ref pspec_path, .. } => {
            // Otomatik algılama: pspec_path verilmemişse cwd'de pspec.xml/kdl ara
            let pspec_path = match pspec_path {
                Some(p) => p.clone(),
                None => {
                    let cwd = std::env::current_dir().map_err(PisiError::IoError)?;
                    let candidates = ["pspec.kdl", "pspec.xml"];
                    let found = candidates.iter().find(|f| cwd.join(f).exists());
                    match found {
                        Some(f) => cwd.join(f),
                        None => {
                            return Err(PisiError::SpecError(
                                "pspec.kdl veya pspec.xml bulunamadı.".to_string()
                            ));
                        }
                    }
                }
            };

            let pspec_str = pspec_path.to_string_lossy();
            let mut target_url = pspec_str.to_string();

            // Eğer yerel bir dosya değilse ve URL de değilse veritabanından uzak kaynak URL'sini sorgula
            if !pspec_path.exists()
                && !pspec_str.starts_with("http://")
                && !pspec_str.starts_with("https://")
            {
                if let Ok(Some(url)) = db.get_source(&pspec_str) {
                    println!(
                        "{}",
                        t!("build_remote_found", package = pspec_str, url = url)
                    );
                    target_url = url;
                }
            }

            let local_pspec_path =
                if target_url.starts_with("http://") || target_url.starts_with("https://") {
                    repo_manager.download_remote_spec(&target_url)?
                } else {
                    pspec_path.clone()
                };

            println!(
                "{}",
                t!("build_starting", package = local_pspec_path.display())
            );

            let spec = PisiSpec::from_path(&local_pspec_path).map_err(|e| {
                PisiError::SpecError(t!("main_error_spec_parse", error = e).to_string())
            })?;

            // Validate date format in history
            if let Some(ref history) = spec.history {
                let raw_content =
                    fs::read_to_string(&local_pspec_path).map_err(PisiError::IoError)?;
                let lines: Vec<&str> = raw_content.lines().collect();
                for update in &history.updates {
                    if NaiveDate::parse_from_str(&update.date, "%Y-%m-%d").is_err() {
                        let line_num = lines
                            .iter()
                            .position(|l| l.contains(&update.date))
                            .map(|i| i + 1)
                            .unwrap_or(0);
                        return Err(PisiError::SpecError(t!(
                            "emerge_err_invalid_date",
                            line = line_num.to_string(),
                            date = update.date
                        )
                        .to_string()));
                    }
                }
            }

            // BuildOptions Hazırla
            let jobs_count = cli
                .jobs
                .as_ref()
                .map(|j| j.to_lowercase().replace(['j', '-'], "").trim().to_string())
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or_else(|| {
                    std::thread::available_parallelism()
                        .map(|n| n.get())
                        .unwrap_or(1)
                });

            let (run_build, run_install, run_package) =
                if let Commands::Build {
                    build, install, package, ..
                } = &cli.command
                {
                    (*build, *install, *package)
                } else {
                    (false, false, false)
                };

            let build_options = BuildOptions {
                jobs: jobs_count,
                verbose: cli.verbose,
                debug: cli.debug,
                log_path: cli.log_path.clone(),
                optimization_level: cli.opt_level.clone(),
                enable_sandbox: if let Commands::Build { no_sandbox, .. } = &cli.command {
                    config.build.enablesandbox && !*no_sandbox
                } else {
                    config.build.enablesandbox
                },
                architecture: if let Commands::Build {
                    target: Some(t),
                    ..
                } = &cli.command
                {
                    t.clone()
                } else {
                    config.general.architecture.clone()
                },
                yes_all,
                sbindir: "usr/bin".to_string(),
                run_build,
                run_install,
                run_package,
            };

            // Asıl çalışma dizinini kaydet (taşıma işlemi için)
            let original_dir = std::env::current_dir().map_err(PisiError::IoError)?;

            // Python PiSi'deki pkg_dir() mantığıyla uyumlu dizin yap:
            // <tmp_dir>/<name>-<version>-<release>/
            let (version, release) = spec
                .history
                .as_ref()
                .and_then(|h| h.updates.first())
                .map(|u| (u.version.as_str(), u.release.to_string()))
                .unwrap_or(("0.0.0", "1".to_string()));

            let pkg_dir_name = format!("{}-{}-{}", spec.source.name, version, release);
            let work_dir = config.directories.tmp_dir.join(&pkg_dir_name);

            // Faz seçimi: hiçbiri belirtilmemişse temizle, aksi halde mevcut build'i koru
            let has_phase_flags = run_build || run_install || run_package;
            if has_phase_flags {
                // Mevcut build dizinini koru, sadece var olduğundan emin ol
                fs::create_dir_all(&work_dir).map_err(PisiError::IoError)?;
            } else {
                // Temiz bir build: dizini silip yeniden oluştur
                if work_dir.exists() {
                    fs::remove_dir_all(&work_dir).map_err(PisiError::IoError)?;
                }
                fs::create_dir_all(&work_dir).map_err(PisiError::IoError)?;
            }

            // specdir: pspec/kdl dosyasının bulunduğu dizin
            let specdir = local_pspec_path
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();

            // actions.py dosyasını kopyala (eğer varsa)
            let actions_py = specdir.join("actions.py");
            if actions_py.is_file() {
                fs::copy(&actions_py, work_dir.join("actions.py")).map_err(PisiError::IoError)?;
            } else if actions_py.exists() {
                eprintln!("Warning: actions.py exists but is not a regular file, skipping");
            }

            // Yamalar için files/ dizinini kopyala (patch dosyaları)
            let patches = spec
                .source
                .patches
                .as_ref()
                .map(|pw| pw.patches.as_slice())
                .unwrap_or(&[]);
            for patch in patches {
                // Yamalar specdir/files/<patch_file> konumundadır
                let src_patch = specdir.join("files").join(&patch.file);
                if src_patch.is_file() {
                    let dst_patch = work_dir.join("files").join(&patch.file);
                    if let Some(p) = dst_patch.parent() {
                        fs::create_dir_all(p).map_err(PisiError::IoError)?;
                    }
                    fs::copy(&src_patch, &dst_patch).map_err(PisiError::IoError)?;
                } else {
                    // Yamalar specdir/patches/ konumunda da olabilir
                    let alt_src = specdir.join(&patch.file);
                    if alt_src.is_file() {
                        let dst_patch = work_dir.join(&patch.file);
                        if let Some(p) = dst_patch.parent() {
                            fs::create_dir_all(p).map_err(PisiError::IoError)?;
                        }
                        fs::copy(&alt_src, &dst_patch).map_err(PisiError::IoError)?;
                    }
                }
            }

            let builder = PackageBuilder::new(
                spec,
                work_dir.clone(),
                specdir,
                db.clone(),
                config.clone(),
                build_options,
            );

            let res = builder.build(trace_id);
            if let Err(_e) = &res {
                let log_path = work_dir.join("pisi-build-error.log");
                if log_path.exists() {
                    if let Ok(log) = fs::read_to_string(&log_path) {
                        eprintln!("{}", t!("main_build_error_log", log = log));
                    }
                }
            }
            let built_package_paths = res?;

            println!("{}", colorize(&t!("emerge_built_packages"), "green"));
            for path in &built_package_paths {
                let fname = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string());
                println!("  {}", fname);
            }

            for built_package_path in built_package_paths {
                // İnşa edilen paketi orijinal çalışma dizinine taşı
                let final_package_path =
                    original_dir.join(built_package_path.file_name().ok_or_else(|| {
                        PisiError::RuntimeError(t!("main_error_pkg_name").to_string())
                    })?);

                println!(
                    "{}",
                    t!(
                        "main_pkg_moving",
                        src = built_package_path.display(),
                        dest = final_package_path.display()
                    )
                );
                // cross-device link hatasını önlemek için kopyala ve sil
                std::fs::copy(&built_package_path, &final_package_path)
                    .map_err(PisiError::IoError)?;
                let _ = std::fs::remove_file(&built_package_path);

                println!(
                    "{}",
                    t!("success_build_moved", path = final_package_path.display())
                );
            }

            // autoclean config ayarına göre build dizinini temizle
            if config.general.autoclean {
                let _ = fs::remove_dir_all(&work_dir);
            } else {
                println!(
                    "{}",
                    t!("main_build_dir_preserved", path = work_dir.display())
                );
            }
        }

        // --- HISTORY & ROLLBACK ---
        Commands::History {
            trace_id: target_trace_id,
            from,
            to,
            ..
        } => {
            if let Some(id) = target_trace_id {
                println!("{}", t!("main_rollback_starting"));
                installer.perform_rollback(id, trace_id)?;
            } else {
                query_manager.perform_list_history(from, to)?;
            }
        }
        Commands::Rollback {
            trace_id: target_trace_id,
        } => installer.perform_rollback(target_trace_id, trace_id)?,

        Commands::BackupDb { directory } => {
            installer.backup_db(&PathBuf::from(directory))?;
        }

        Commands::RestoreDb { directory } => {
            installer.restore_db(&PathBuf::from(directory))?;
        }

        Commands::VerifyDb => {
            installer.verify_db()?;
        }

        // --- SORGULAMA (ROOT İSTEMEZ) ---
        Commands::Search { query, .. } => query_manager.perform_search(query)?,
        Commands::SearchFile { file_path, .. } => {
            query_manager.perform_search_file(&file_path)?
        }
        Commands::ListInstalled { .. } => query_manager.perform_list_installed()?,
        Commands::ListOrphaned { .. } => query_manager.perform_list_orphaned(&installer)?,
        Commands::ListPending { .. } => query_manager.perform_list_pending()?,
        Commands::ListSources { .. } => query_manager.perform_list_sources()?,
        Commands::Help { .. } => {}
        Commands::Blame {
            package_name,
            release,
            all,
        } => query_manager.perform_blame(&package_name, release, all)?,
        Commands::ListComponents { .. } => query_manager.perform_list_components()?,
        Commands::CheckInstall {
            package_names,
            reinstall,
        } => query_manager.perform_check_install(
            &installer,
            package_names,
            reinstall,
            yes_all,
            trace_id,
            bandwidth_limit,
            None,
        )?,
        Commands::ListNewest { limit, .. } => query_manager.perform_list_newest(limit)?,
        Commands::ListFiles { package_name } => query_manager.perform_list_files(&package_name)?,
        Commands::ListUpgrades { .. } => {
            query_manager.perform_list_upgrades(cli.verbose, false)?
        }
        Commands::Graph {
            package_names,
            installed,
            reverse,
            output,
        } => query_manager.perform_graph(package_names, installed, reverse, &output)?,
        Commands::Index { source_dir, output } => {
            archive_tools.perform_index(&source_dir, &output)?
        }
        Commands::RebuildDb => repo_manager.perform_rebuild_db(trace_id)?,
        Commands::ListAvailable { .. } => query_manager.perform_list_available()?,
        Commands::CheckRepo { circular } => repo_manager.perform_check_repo(circular)?,
        Commands::RepoDiff {
            source_index,
            binary_index,
        } => repo_manager.perform_repo_diff(&source_index, &binary_index)?,

        // --- GELİŞMİŞ INFO (Önce Kurulu, Sonra Depo) ---
        Commands::Info { package_name, .. } => query_manager.perform_info(&package_name)?,
        // --- REPO İŞLEMLERİ ---
        Commands::UpdateRepo => repo_manager.perform_update(trace_id)?,
        Commands::AddRepo { name, url } => repo_manager.perform_add_repo(&name, &url, trace_id)?,
        Commands::RemoveRepo { name } => repo_manager.perform_remove_repo(&name, trace_id)?,
        Commands::EnableRepo { name } => {
            repo_manager.perform_set_repo_status(&name, true, trace_id)?
        }
        Commands::DisableRepo { name } => {
            repo_manager.perform_set_repo_status(&name, false, trace_id)?
        }
        Commands::ListRepo { .. } => repo_manager.perform_list_repos()?,

        Commands::DbTest => println!("{}", t!("success_db_connection")),
        Commands::Version
        | Commands::Temp
        | Commands::CheckComponents { .. }
        | Commands::ResetHistory { .. } => {}
    }

    // 4. Trace ID Güncelleme
    db.increment_trace_id(trace_id)?;
    Ok(())
}

// --- YARDIMCI FONKSİYONLAR ---

fn acquire_lock(config: &Config) -> PisiResult<File> {
    let lock_path = config.directories.lock_dir.join("pisi.lock");

    if let Some(parent) = lock_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(PisiError::IoError)?;
        }
    }

    let file = File::create(&lock_path).map_err(PisiError::IoError)?;
    let fd = file.as_raw_fd();

    use nix::fcntl::{flock, FlockArg};
    use nix::errno::Errno;

    if let Err(e) = flock(fd, FlockArg::LockExclusiveNonblock) {
        if e == Errno::EAGAIN {
            eprintln!(
                "{}",
                t!("another_process_running", lock_path = lock_path.display())
            );
            std::process::exit(1);
        }
        let err: std::io::Error = e.into();
        return Err(PisiError::IoError(err));
    }
    Ok(file)
}

fn handle_version() -> PisiResult<()> {
    println!("{}", t!("main_version_title"));
    println!("{:-<40}", "");
    println!(
        "{:<15}: {}",
        t!("main_version_field_version"),
        env!("CARGO_PKG_VERSION")
    );
    println!(
        "{:<15}: {}",
        t!("main_version_field_platform"),
        std::env::consts::OS
    );
    println!(
        "{:<15}: {}",
        t!("main_version_field_arch"),
        std::env::consts::ARCH
    );
    println!("{:<15}: GPL-3.0", t!("main_version_field_license"));
    println!(
        "{:<15}: {}",
        t!("main_version_field_dev"),
        t!("main_version_field_dev_value")
    );
    println!(
        "{:<15}: https://pisilinux.org",
        t!("main_version_field_web")
    );
    println!("{:-<40}", "");
    Ok(())
}

fn handle_help(command: Option<String>) -> PisiResult<()> {
    let mut cmd = PisiCli::command();
    cmd = localize_command(cmd);
    if let Some(sub) = command {
        // Alt komutu veya alias'ı bul
        let sub_cmd = cmd
            .get_subcommands_mut()
            .find(|c| c.get_name() == sub || c.get_visible_aliases().any(|a| a == sub));

        if let Some(s) = sub_cmd {
            s.print_help()
                .map_err(|e| PisiError::RuntimeError(e.to_string()))?;
            println!();
        } else {
            println!("{}", t!("command_not_found", command = sub));
        }
    } else {
        cmd.print_help()
            .map_err(|e| PisiError::RuntimeError(e.to_string()))?;
        println!();
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn handle_emerge(
    package_names: Vec<String>,
    trace_id: u64,
    yes_all: bool,
    bandwidth_limit: Option<usize>,
    auth: Option<(String, String)>,
    download_only: bool,
    ignore_check: bool,
    jobs: Option<String>,
    verbose: bool,
    debug: bool,
    log_path: Option<PathBuf>,
    opt_level: Option<String>,
    ignore_comar: bool,
    ignore_file_conflict: bool,
    ignore_package_conflict: bool,
    ignore_dependency: bool,
    config: &Config,
    db: &PisiDatabase,
    installer: &Installer,
    repo_manager: &Repository,
) -> PisiResult<()> {
    let clean_package_names: Vec<String> = package_names
        .into_iter()
        .filter(|name| !name.starts_with('-'))
        .collect();

    if clean_package_names.is_empty() {
        return Ok(());
    }

    let order = installer.calculate_install_order(&clean_package_names)?;
    let build_order = if order.is_empty() {
        clean_package_names.clone()
    } else {
        order
    };

    // First pass: classify packages and print plan summary
    let mut to_install_binary: Vec<String> = Vec::new();
    let mut to_build_source: Vec<String> = Vec::new();

    for pkg_name in &build_order {
        let source_url_opt = db.get_source(pkg_name).ok().flatten();
        if source_url_opt.is_some() {
            to_build_source.push(pkg_name.clone());
        } else {
            to_install_binary.push(pkg_name.clone());
        }
    }

    if config.general.package_cache {
        println!("{}", t!("emerge_output_dir"));
    }

    if !to_install_binary.is_empty() {
        println!("\n{}", t!("emerge_plan_install_header"));
        pisi_core::print_in_columns(&to_install_binary);
    }

    if !to_build_source.is_empty() {
        println!("\n{}", t!("emerge_plan_build_header"));
        pisi_core::print_in_columns(&to_build_source);
    }

    // Check for extra packages (not explicitly requested by user)
    let user_requested: std::collections::HashSet<&str> =
        clean_package_names.iter().map(|s| s.as_str()).collect();
    let has_extras = !build_order.iter().all(|p| user_requested.contains(p.as_str()));

    if has_extras && !yes_all {
        println!("\n{}", t!("emerge_extra_packages"));
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let trimmed = input.trim().to_lowercase();
        if trimmed != "y" && trimmed != "yes" && trimmed != "e" && trimmed != "evet" {
            println!("Aborted.");
            return Ok(());
        }
    }

    for pkg_name in build_order {
        let source_url_opt = db.get_source(&pkg_name).ok().flatten();
        if let Some(source_url) = source_url_opt {
            println!(
                "{}",
                t!(
                    "emerge_remote_recipe_found",
                    name = pkg_name,
                    url = source_url
                )
            );
            let local_pspec_path = repo_manager.download_remote_spec(&source_url)?;
            let spec = PisiSpec::from_path(&local_pspec_path).map_err(|e| {
                PisiError::SpecError(t!("main_error_spec_parse", error = e).to_string())
            })?;

            // Validate date format in history
            if let Some(ref history) = spec.history {
                let raw_content =
                    fs::read_to_string(&local_pspec_path).map_err(PisiError::IoError)?;
                let lines: Vec<&str> = raw_content.lines().collect();
                for update in &history.updates {
                    if NaiveDate::parse_from_str(&update.date, "%Y-%m-%d").is_err() {
                        let line_num = lines
                            .iter()
                            .position(|l| l.contains(&update.date))
                            .map(|i| i + 1)
                            .unwrap_or(0);
                        return Err(PisiError::SpecError(t!(
                            "emerge_err_invalid_date",
                            line = line_num.to_string(),
                            date = update.date
                        )
                        .to_string()));
                    }
                }
            }

            let jobs_count = jobs
                .as_ref()
                .map(|j| j.to_lowercase().replace(['j', '-'], "").trim().to_string())
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or_else(|| {
                    std::thread::available_parallelism()
                        .map(|n| n.get())
                        .unwrap_or(1)
                });

            let disable_sandbox = std::env::args().any(|arg| arg == "--no-sandbox");

            let build_options = BuildOptions {
                jobs: jobs_count,
                verbose,
                debug,
                log_path: log_path.clone(),
                optimization_level: opt_level.clone(),
                enable_sandbox: config.build.enablesandbox && !disable_sandbox,
                architecture: config.general.architecture.clone(),
                yes_all,
                sbindir: std::env::var("PISI_SBINDIR").unwrap_or_else(|_| "usr/bin".to_string()),
                run_build: false,
                run_install: false,
                run_package: false,
            };

            let _original_dir = std::env::current_dir().map_err(PisiError::IoError)?;

            let (version, release) = spec
                .history
                .as_ref()
                .and_then(|h| h.updates.first())
                .map(|u| (u.version.as_str(), u.release.to_string()))
                .unwrap_or(("0.0.0", "1".to_string()));

            let pkg_dir_name = format!("{}-{}-{}", spec.source.name, version, release);
            let work_dir = config.directories.tmp_dir.join(&pkg_dir_name);
            if work_dir.exists() {
                fs::remove_dir_all(&work_dir).map_err(PisiError::IoError)?;
            }
            fs::create_dir_all(&work_dir).map_err(PisiError::IoError)?;

            let specdir = local_pspec_path
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();

            let actions_py = specdir.join("actions.py");
            if actions_py.is_file() {
                fs::copy(&actions_py, work_dir.join("actions.py")).map_err(PisiError::IoError)?;
            }

            let patches = spec
                .source
                .patches
                .as_ref()
                .map(|pw| pw.patches.as_slice())
                .unwrap_or(&[]);
            for patch in patches {
                let src_patch = specdir.join("files").join(&patch.file);
                if src_patch.is_file() {
                    let dst_patch = work_dir.join("files").join(&patch.file);
                    if let Some(p) = dst_patch.parent() {
                        fs::create_dir_all(p).map_err(PisiError::IoError)?;
                    }
                    fs::copy(&src_patch, &dst_patch).map_err(PisiError::IoError)?;
                } else {
                    let alt_src = specdir.join(&patch.file);
                    if alt_src.is_file() {
                        let dst_patch = work_dir.join(&patch.file);
                        if let Some(p) = dst_patch.parent() {
                            fs::create_dir_all(p).map_err(PisiError::IoError)?;
                        }
                        fs::copy(&alt_src, &dst_patch).map_err(PisiError::IoError)?;
                    }
                }
            }

            let builder = PackageBuilder::new(
                spec,
                work_dir.clone(),
                specdir,
                db.clone(),
                config.clone(),
                build_options,
            );

            let res = builder.build(trace_id);
            if let Err(_e) = &res {
                let log_path_err = work_dir.join("pisi-build-error.log");
                if log_path_err.exists() {
                    if let Ok(log) = fs::read_to_string(&log_path_err) {
                        eprintln!("{}", t!("main_build_error_log", log = log));
                    }
                }
            }
            let built_package_paths = res?;

            let built_package_paths_str: Vec<String> = built_package_paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();

            println!("{}", colorize(&t!("emerge_built_packages"), "green"));
            for path_str in &built_package_paths_str {
                let fname = std::path::Path::new(path_str)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path_str.clone());
                println!("  {}", fname);
            }

            installer.perform_install(
                built_package_paths_str,
                trace_id,
                true,
                yes_all,
                bandwidth_limit,
                auth.clone(),
                download_only,
                ignore_check,
                ignore_comar,
                ignore_file_conflict,
                ignore_package_conflict,
                true,
                ignore_dependency,
                None,
            )?;

            if config.general.autoclean {
                let _ = fs::remove_dir_all(&work_dir);
            }
        } else {
            println!("{}", t!("emerge_no_remote_recipe", name = pkg_name));
            installer.perform_install(
                vec![pkg_name.clone()],
                trace_id,
                false,
                yes_all,
                bandwidth_limit,
                auth.clone(),
                download_only,
                ignore_check,
                ignore_comar,
                ignore_file_conflict,
                ignore_package_conflict,
                false,
                ignore_dependency,
                None,
            )?;
        }
    }

    Ok(())
}

fn handle_temp() -> PisiResult<()> {
    use std::io::Write;
    print!("{}", t!("temp_enter_pkg_name"));
    std::io::stdout().flush().unwrap();

    let mut pkg_name = String::new();
    std::io::stdin().read_line(&mut pkg_name).unwrap();
    let pkg_name = pkg_name.trim();

    if pkg_name.is_empty() {
        eprintln!("{}", t!("temp_err_empty_name"));
        return Ok(());
    }

    let dir_path = PathBuf::from(pkg_name);
    if dir_path.exists() {
        eprintln!("{}", t!("temp_err_dir_exists", name = pkg_name));
        return Ok(());
    }

    fs::create_dir_all(dir_path.join("files")).map_err(PisiError::IoError)?;
    fs::create_dir_all(dir_path.join("comar")).map_err(PisiError::IoError)?;

    let template_content = include_str!("../pisi-template/template.kdl");
    let kdl_path = dir_path.join(format!("{}.kdl", pkg_name));
    fs::write(&kdl_path, template_content).map_err(PisiError::IoError)?;

    let service_kdl = include_str!("../pisi-template/comar/service.kdl");
    let package_kdl = include_str!("../pisi-template/comar/package.kdl");
    let pakhandler_kdl = include_str!("../pisi-template/comar/pakhandler.kdl");

    fs::write(dir_path.join("comar/service.kdl"), service_kdl).map_err(PisiError::IoError)?;
    fs::write(dir_path.join("comar/package.kdl"), package_kdl).map_err(PisiError::IoError)?;
    fs::write(dir_path.join("comar/pakhandler.kdl"), pakhandler_kdl)
        .map_err(PisiError::IoError)?;

    let template_patch = include_str!("../pisi-template/files/template.patch");
    fs::write(dir_path.join("files/template.patch"), template_patch).map_err(PisiError::IoError)?;

    println!("{}", t!("temp_success_created", name = pkg_name));
    println!("{}", t!("temp_dir_structure"));
    println!("  {}/", pkg_name);
    println!("  ├── files/");
    println!("  ├── comar/");
    println!("  │   ├── service.kdl     {}", t!("temp_desc_service"));
    println!("  │   ├── package.kdl     {}", t!("temp_desc_package"));
    println!("  │   └── pakhandler.kdl  {}", t!("temp_desc_handler"));
    println!("  └── {}.kdl", pkg_name);

    Ok(())
}
