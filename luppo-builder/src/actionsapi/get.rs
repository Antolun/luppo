use std::env;
use std::path::Path;

pub fn arch() -> String {
    std::env::consts::ARCH.to_string()
}
pub fn cflags() -> String {
    env::var("CFLAGS").unwrap_or_default()
}
pub fn cxxflags() -> String {
    env::var("CXXFLAGS").unwrap_or_default()
}
pub fn ldflags() -> String {
    env::var("LDFLAGS").unwrap_or_default()
}
pub fn host() -> String {
    env::var("HOST").unwrap_or_else(|_| format!("{}-pc-linux-gnu", arch()))
}
pub fn chost() -> String {
    host()
}

pub fn cc() -> String {
    env::var("CC").unwrap_or_else(|_| format!("{}-gcc", host()))
}
pub fn cxx() -> String {
    env::var("CXX").unwrap_or_else(|_| format!("{}-g++", host()))
}
pub fn ar() -> String {
    get_binutils_info("ar")
}
pub fn ld() -> String {
    get_binutils_info("ld")
}
pub fn ranlib() -> String {
    get_binutils_info("ranlib")
}
pub fn r#as() -> String {
    get_binutils_info("as")
}
pub fn nm() -> String {
    get_binutils_info("nm")
}
pub fn f77() -> String {
    get_binutils_info("g77")
}
pub fn gcj() -> String {
    get_binutils_info("gcj")
}

fn get_binutils_info(util: &str) -> String {
    let cross_name = format!("{}-{}", host(), util);
    if exist_binary(&cross_name) {
        cross_name
    } else {
        util.to_string()
    }
}

pub fn exist_binary(name: &str) -> bool {
    if let Ok(path_var) = env::var("PATH") {
        for dir in path_var.split(':') {
            if Path::new(dir).join(name).exists() {
                return true;
            }
        }
    }
    false
}

pub fn env_var(key: &str) -> Option<String> {
    env::var(key).ok()
}

pub fn make_jobs() -> String {
    if let Ok(v) = env::var("MAKEOPTS") {
        let cleaned = v
            .to_lowercase()
            .replace("-j", "")
            .replace('j', "")
            .trim()
            .to_string();
        if !cleaned.is_empty() && cleaned.chars().all(|c| c.is_ascii_digit()) {
            return cleaned;
        }
    }
    if let Ok(v) = env::var("JOBS") {
        if !v.is_empty() && v.chars().all(|c| c.is_ascii_digit()) {
            return v;
        }
    }
    std::thread::available_parallelism()
        .map(|n| n.get().to_string())
        .unwrap_or_else(|_| "1".to_string())
}

pub fn src_name() -> String {
    env::var("SRC_NAME").unwrap_or_default()
}
pub fn src_version() -> String {
    env::var("SRC_VERSION").unwrap_or_default()
}
pub fn src_release() -> String {
    env::var("SRC_RELEASE").unwrap_or_else(|_| "1".to_string())
}
pub fn src_tag() -> String {
    format!("{}-{}-{}", src_name(), src_version(), src_release())
}
pub fn src_dir() -> String {
    format!("{}-{}", src_name(), src_version())
}

pub fn install_dir() -> String {
    env::var("INSTALL_DIR").unwrap_or_else(|_| "install_root".to_string())
}
pub fn work_dir() -> String {
    env::var("WORK_DIR").unwrap_or_else(|_| ".".to_string())
}
pub fn pkg_dir() -> String {
    env::var("PKG_DIR").unwrap_or_else(|_| "/var/cache/luppo/packages".to_string())
}

pub fn cur_dir() -> String {
    env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}
pub fn cur_kernel() -> String {
    nix_uname()
}
pub fn kernel_release() -> String {
    nix_uname()
}
pub fn cur_python() -> String {
    std::process::Command::new("python3")
        .args([
            "-c",
            "import sys; print(f'python{sys.version_info.major}.{sys.version_info.minor}')",
        ])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "python3".to_string())
}
pub fn cur_perl() -> String {
    std::process::Command::new("perl")
        .args(["-e", "printf '%vd', $^V"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "5".to_string())
}

pub fn build_type() -> String {
    env::var("LUPPO_BUILD_TYPE").unwrap_or_default()
}

pub fn doc_dir() -> String {
    "usr/share/doc".to_string()
}
pub fn sbin_dir() -> String {
    env::var("LUPPO_SBINDIR").unwrap_or_else(|_| "usr/bin".to_string())
}
pub fn man_dir() -> String {
    "usr/share/man".to_string()
}
pub fn info_dir() -> String {
    "usr/share/info".to_string()
}
pub fn data_dir() -> String {
    "usr/share".to_string()
}
pub fn conf_dir() -> String {
    "etc".to_string()
}
pub fn localstate_dir() -> String {
    "var".to_string()
}
pub fn libexec_dir() -> String {
    "usr/libexec".to_string()
}
pub fn default_prefix_dir() -> String {
    "usr".to_string()
}
pub fn emul32_prefix_dir() -> String {
    "emul32".to_string()
}
pub fn kde_dir() -> String {
    env::var("KDEDIR").unwrap_or_else(|_| "/usr".to_string())
}
pub fn qt_dir() -> String {
    env::var("QTDIR").unwrap_or_else(|_| "/usr".to_string())
}

fn nix_uname() -> String {
    std::process::Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}
