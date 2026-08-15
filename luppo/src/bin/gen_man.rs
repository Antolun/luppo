use clap::CommandFactory;
use clap_complete::Shell;
use clap_mangen::Man;
use std::fs;
use std::path::PathBuf;

#[derive(clap::Parser, Debug, Clone)]
#[command(
    author,
    version,
    about = "Rust-based Luppo Package Manager",
    long_about = None,
    disable_help_subcommand = true
)]
struct LuppoCli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short = 'D', long, global = true, value_name = "DIR")]
    destdir: Option<String>,

    #[arg(short = 'y', long, global = true)]
    yes_all: bool,

    #[arg(short = 'v', long, global = true)]
    verbose: bool,

    #[arg(short = 'd', long, global = true)]
    debug: bool,

    #[arg(short = 'N', long, global = true)]
    no_color: bool,

    #[arg(short = 'L', long, global = true, value_name = "KILOBYTES")]
    bandwidth_limit: Option<usize>,

    #[arg(short = 'u', long, global = true, value_name = "USERNAME")]
    username: Option<String>,

    #[arg(short = 'p', long, global = true, value_name = "PASSWORD")]
    password: Option<String>,

    #[arg(short = 'j', long, global = true, value_name = "JOBS")]
    jobs: Option<String>,

    #[arg(long, global = true)]
    download_only: bool,

    #[arg(long, global = true)]
    ignore_check: bool,

    #[arg(long, global = true, value_name = "FILE")]
    log_path: Option<PathBuf>,

    #[arg(long, global = true, value_name = "LEVEL")]
    opt_level: Option<String>,
}

#[derive(clap::Subcommand, Debug, Clone)]
enum Commands {
    #[command(name = "add-repo", alias = "ar")]
    AddRepo { name: String, url: String },

    #[command(name = "blame", alias = "bl")]
    Blame { package: String },

    #[command(name = "build", alias = "bi")]
    Build {
        #[arg(short = 'j', long)]
        jobs: Option<String>,
        #[arg(long)]
        no_sandbox: bool,
        #[arg(long)]
        install_deps: bool,
        #[arg(long)]
        target: Option<String>,
        #[arg(long)]
        log_path: Option<PathBuf>,
        #[arg(long)]
        opt_level: Option<String>,
        spec: Option<String>,
    },

    #[command(name = "check-install", alias = "ci")]
    CheckInstall { packages: Vec<String> },

    #[command(name = "check-components")]
    CheckComponents { path: Option<String> },

    #[command(name = "reset-history")]
    ResetHistory { path: Option<String> },

    #[command(name = "check-repo")]
    CheckRepo {
        #[arg(long)]
        circular: bool,
    },

    #[command(name = "repo-diff")]
    RepoDiff { index1: String, index2: String },

    #[command(name = "toolchain")]
    Toolchain {
        #[arg(long)]
        start: bool,
        #[arg(long)]
        update: bool,
    },

    #[command(name = "clean")]
    Clean,

    #[command(name = "configure-pending", alias = "cp")]
    ConfigurePending,

    #[command(name = "delete-cache", alias = "dc")]
    DeleteCache,

    #[command(name = "delta", alias = "dt")]
    Delta {
        #[arg(required = true)]
        old_packages: Vec<String>,
        #[arg(required = true)]
        new_package: String,
        #[arg(long)]
        output_dir: Option<String>,
    },

    #[command(name = "disable-repo", alias = "dr")]
    DisableRepo { repo: String },

    #[command(name = "emerge", alias = "em")]
    Emerge {
        packages: Vec<String>,
        #[arg(long)]
        no_deps: bool,
    },

    #[command(name = "emerge-up", alias = "emup")]
    EmergeUp,

    #[command(name = "enable-repo", alias = "er")]
    EnableRepo { repo: String },

    #[command(name = "fetch", alias = "fc")]
    Fetch {
        packages: Vec<String>,
        #[arg(short = 'o', long)]
        output_dir: Option<String>,
        #[arg(long)]
        runtime_deps: bool,
    },

    #[command(name = "graph")]
    Graph {
        package: String,
        #[arg(long)]
        reverse: bool,
    },

    #[command(name = "help")]
    Help { command: Option<String> },

    #[command(name = "history", alias = "hs")]
    History {
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
    },

    #[command(name = "index", alias = "ix")]
    Index {
        path: String,
        #[arg(long)]
        output: Option<String>,
    },

    #[command(name = "info")]
    Info { package: String },

    #[command(name = "install", alias = "it")]
    Install {
        packages: Vec<String>,
        #[arg(long)]
        reinstall: bool,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        download_only: bool,
        #[arg(long)]
        ignore_check: bool,
        #[arg(long)]
        ignore_dependency: bool,
        #[arg(long)]
        ignore_comar: bool,
        #[arg(long)]
        ignore_file_conflict: bool,
        #[arg(long)]
        ignore_package_conflict: bool,
        #[arg(short = 'D', long)]
        destdir: Option<String>,
        #[arg(long)]
        ignore_safety: bool,
        #[arg(long)]
        no_sandbox: bool,
        #[arg(long)]
        install_deps: bool,
    },

    #[command(name = "list-available", alias = "la")]
    ListAvailable,

    #[command(name = "list-components", alias = "lc")]
    ListComponents,

    #[command(name = "list-files", alias = "lf")]
    ListFiles { package: String },

    #[command(name = "list-installed", alias = "li")]
    ListInstalled,

    #[command(name = "list-newest", alias = "ln")]
    ListNewest { #[arg(long)] limit: Option<usize> },

    #[command(name = "list-orphaned", alias = "lo")]
    ListOrphaned,

    #[command(name = "list-pending", alias = "lp")]
    ListPending,

    #[command(name = "list-repo", alias = "lr")]
    ListRepo,

    #[command(name = "list-sources", alias = "ls")]
    ListSources,

    #[command(name = "list-upgrades", alias = "lu")]
    ListUpgrades,

    #[command(name = "rebuild-db", alias = "rdb")]
    RebuildDb,

    #[command(name = "remove", alias = "rm")]
    Remove {
        packages: Vec<String>,
        #[arg(long)]
        ignore_dependency: bool,
        #[arg(long)]
        ignore_safety: bool,
        #[arg(long)]
        ignore_comar: bool,
    },

    #[command(name = "remove-orphaned", alias = "ro")]
    RemoveOrphaned,

    #[command(name = "remove-repo", alias = "rr")]
    RemoveRepo { repo: String },

    #[command(name = "rollback", alias = "rb")]
    Rollback { trace_id: u64 },

    #[command(name = "search", alias = "sr")]
    Search { query: String },

    #[command(name = "search-file", alias = "sf")]
    SearchFile { path: String },

    #[command(name = "update-repo", alias = "ur")]
    UpdateRepo,

    #[command(name = "upgrade", alias = "up")]
    Upgrade {
        packages: Vec<String>,
        #[arg(long)]
        check_only: bool,
        #[arg(long)]
        integrity_only: bool,
        #[arg(long)]
        no_integrity: bool,
        #[arg(long)]
        component: Option<String>,
    },

    #[command(name = "version")]
    Version,

    #[command(name = "temp", alias = "tmp")]
    Temp { name: Option<String> },
}

fn main() {
    let out_dir = PathBuf::from("man");
    fs::create_dir_all(&out_dir).unwrap();

    let mut cmd = LuppoCli::command();

    let man = Man::new(cmd);
    man.generate_to(&out_dir).unwrap();
    println!("Generated man pages in: {}", out_dir.display());

    let mut cmd2 = LuppoCli::command();
    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
        let mut buf: Vec<u8> = Vec::new();
        clap_complete::generate(shell, &mut cmd2, "luppo", &mut buf);
        let ext = match shell {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::PowerShell => "ps1",
            _ => "txt",
        };
        let comp_path = out_dir.join(format!("luppo.{}", ext));
        fs::write(&comp_path, buf).unwrap();
        println!("Generated completion: {}", comp_path.display());
    }
}