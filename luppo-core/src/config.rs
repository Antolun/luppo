use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub build: BuildSettings,
    pub directories: DirectorySettings,
    pub general: GeneralSettings,
}

#[derive(Debug, Clone)]
pub struct BuildSettings {
    pub build_host: String,
    pub commonflags: String,
    pub cflags: String,
    pub cxxflags: String,
    pub host: String,
    pub cc: String,
    pub cxx: String,
    pub compressionlevel: u8,
    pub enablesandbox: bool,
    pub fallback: String,
    pub generatedebug: bool,
    pub jobs: String,
    pub ldflags: String,
    pub ignored_build_types: String,
}

#[derive(Debug, Clone)]
pub struct DirectorySettings {
    pub cache_root_dir: PathBuf,
    pub archives_dir: PathBuf,
    pub cached_packages_dir: PathBuf,
    pub compiled_packages_dir: PathBuf,
    pub debug_packages_dir: PathBuf,
    pub lib_dir: PathBuf,
    pub history_dir: PathBuf,
    pub index_dir: PathBuf,
    pub info_dir: PathBuf,
    pub kde_dir: PathBuf,
    pub lock_dir: PathBuf,
    pub log_dir: PathBuf,
    pub packages_dir: PathBuf,
    pub qt_dir: PathBuf,
    pub tmp_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct GeneralSettings {
    pub architecture: String,
    pub autoclean: bool,
    pub bandwidth_limit: usize,
    pub destination_directory: PathBuf,
    pub distribution: String,
    pub distribution_release: String,
    pub distribution_id: String,
    pub ignore_delta: bool,
    pub ignore_safety: bool,
    pub package_cache: bool,
    pub package_cache_limit: usize,
    pub ftp_proxy: Option<String>,
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let arch = std::env::consts::ARCH;
        let (architecture, host) = match arch {
            "x86_64" => ("x86_64", "x86_64-pc-linux-gnu"),
            "aarch64" => ("aarch64", "aarch64-unknown-linux-gnu"),
            _ => (arch, arch), // Fallback to raw architecture string
        };

        Self {
            build: BuildSettings {
                build_host: "localhost".to_string(),
                commonflags: "-O2".to_string(),
                cflags: "-O2".to_string(),
                cxxflags: "-O2".to_string(),
                host: host.to_string(),
                cc: "gcc".to_string(),
                cxx: "g++".to_string(),
                compressionlevel: 9,
                enablesandbox: true,
                fallback: "http://source.antolun.com".to_string(),
                generatedebug: false,
                jobs: "-j4".to_string(),
                ldflags: "".to_string(),
                ignored_build_types: "".to_string(),
            },
            directories: DirectorySettings {
                cache_root_dir: PathBuf::from("/var/cache/luppo"),
                archives_dir: PathBuf::from("/var/cache/luppo/archives"),
                cached_packages_dir: PathBuf::from("/var/cache/luppo/packages"),
                compiled_packages_dir: PathBuf::from("/var/cache/luppo/packages"),
                debug_packages_dir: PathBuf::from("/var/cache/luppo/packages-debug"),
                lib_dir: PathBuf::from("/var/lib/luppo"),
                history_dir: PathBuf::from("/var/lib/luppo/history"),
                index_dir: PathBuf::from("/var/lib/luppo/index"),
                info_dir: PathBuf::from("/var/lib/luppo/info"),
                kde_dir: PathBuf::from("/usr"),
                lock_dir: PathBuf::from("/run/lock/subsys"),
                log_dir: PathBuf::from("/var/log"),
                packages_dir: PathBuf::from("/var/lib/luppo/package"),
                qt_dir: PathBuf::from("/usr"),
                tmp_dir: PathBuf::from("/var/luppo"),
            },
            general: GeneralSettings {
                architecture: architecture.to_string(),
                autoclean: false,
                bandwidth_limit: 0,
                destination_directory: PathBuf::from("/"),
                distribution: "LupuS".to_string(),
                distribution_release: "all".to_string(),
                distribution_id: "all".to_string(),
                ignore_delta: false,
                ignore_safety: false,
                package_cache: true,
                package_cache_limit: 0,
                ftp_proxy: None,
                http_proxy: None,
                https_proxy: None,
            },
        }
    }
}

fn read_str(node: &kdl::KdlNode, name: &str) -> String {
    node.get(name)
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
        .unwrap_or_default()
}

fn read_str_opt(node: &kdl::KdlNode, name: &str) -> Option<String> {
    node.get(name).and_then(|v| v.as_string()).map(|s| s.to_string())
}

fn read_u8(node: &kdl::KdlNode, name: &str) -> u8 {
    node.get(name)
        .and_then(|v| v.as_integer().or_else(|| v.as_string().and_then(|s| s.parse().ok())))
        .map(|i| i as u8)
        .unwrap_or(0)
}

fn read_bool(node: &kdl::KdlNode, name: &str) -> bool {
    node.get(name).and_then(|v| v.as_bool()).unwrap_or(false)
}

fn read_usize(node: &kdl::KdlNode, name: &str) -> usize {
    node.get(name)
        .and_then(|v| v.as_integer().or_else(|| v.as_string().and_then(|s| s.parse().ok())))
        .map(|i| i as usize)
        .unwrap_or(0)
}

fn parse_config_kdl(content: &str) -> Result<Config, String> {
    let doc: kdl::KdlDocument = content.parse().map_err(|e| format!("KDL parse error: {}", e))?;
    let mut config = Config::default();

    for node in doc.nodes() {
        match node.name().to_string().as_str() {
            "build" => {
                config.build = BuildSettings {
                    build_host: read_str(node, "build-host"),
                    commonflags: read_str(node, "commonflags"),
                    cflags: read_str(node, "cflags"),
                    cxxflags: read_str(node, "cxxflags"),
                    host: read_str(node, "host"),
                    cc: read_str(node, "cc"),
                    cxx: read_str(node, "cxx"),
                    compressionlevel: read_u8(node, "compressionlevel"),
                    enablesandbox: read_bool(node, "enablesandbox"),
                    fallback: read_str(node, "fallback"),
                    generatedebug: read_bool(node, "generatedebug"),
                    jobs: read_str(node, "jobs"),
                    ldflags: read_str(node, "ldflags"),
                    ignored_build_types: read_str(node, "ignored-build-types"),
                };
            }
            "directories" => {
                config.directories = DirectorySettings {
                    cache_root_dir: PathBuf::from(read_str(node, "cache-root-dir")),
                    archives_dir: PathBuf::from(read_str(node, "archives-dir")),
                    cached_packages_dir: PathBuf::from(read_str(node, "cached-packages-dir")),
                    compiled_packages_dir: PathBuf::from(read_str(node, "compiled-packages-dir")),
                    debug_packages_dir: PathBuf::from(read_str(node, "debug-packages-dir")),
                    lib_dir: PathBuf::from(read_str(node, "lib-dir")),
                    history_dir: PathBuf::from(read_str(node, "history-dir")),
                    index_dir: PathBuf::from(read_str(node, "index-dir")),
                    info_dir: PathBuf::from(read_str(node, "info-dir")),
                    kde_dir: PathBuf::from(read_str(node, "kde-dir")),
                    lock_dir: PathBuf::from(read_str(node, "lock-dir")),
                    log_dir: PathBuf::from(read_str(node, "log-dir")),
                    packages_dir: PathBuf::from(read_str(node, "packages-dir")),
                    qt_dir: PathBuf::from(read_str(node, "qt-dir")),
                    tmp_dir: PathBuf::from(read_str(node, "tmp-dir")),
                };
            }
            "general" => {
                config.general = GeneralSettings {
                    architecture: read_str(node, "architecture"),
                    autoclean: read_bool(node, "autoclean"),
                    bandwidth_limit: read_usize(node, "bandwidth-limit"),
                    destination_directory: PathBuf::from(read_str(node, "destination-directory")),
                    distribution: read_str(node, "distribution"),
                    distribution_release: read_str(node, "distribution-release"),
                    distribution_id: read_str(node, "distribution-id"),
                    ignore_delta: read_bool(node, "ignore-delta"),
                    ignore_safety: read_bool(node, "ignore-safety"),
                    package_cache: read_bool(node, "package-cache"),
                    package_cache_limit: read_usize(node, "package-cache-limit"),
                    ftp_proxy: read_str_opt(node, "ftp-proxy"),
                    http_proxy: read_str_opt(node, "http-proxy"),
                    https_proxy: read_str_opt(node, "https-proxy"),
                };
            }
            _ => {}
        }
    }

    Ok(config)
}

impl Config {
    /// /etc/luppo/luppo.conf (KDL format) dosyasını okur, yoksa varsayılanları döner.
    pub fn load(destdir_override: Option<PathBuf>) -> Self {
        let conf_path = "luppo.conf";
        let mut config = if let Ok(content) = fs::read_to_string(conf_path) {
            parse_config_kdl(&content).unwrap_or_else(|_| Config::default())
        } else if let Ok(content) = fs::read_to_string("/etc/luppo/luppo.conf") {
            parse_config_kdl(&content).unwrap_or_else(|_| Config::default())
        } else {
            Config::default()
        };

        if let Some(d) = destdir_override {
            config.general.destination_directory = d;
        }
        config
    }
}
