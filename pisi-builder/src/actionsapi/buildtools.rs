use crate::actionsapi::core::{cd, run_command, set_env};
use rust_i18n::t;
use std::env;
use std::fs;
use std::path::Path;

fn is_emul32() -> bool {
    env::var("PISI_BUILD_TYPE").as_deref() == Ok("emul32")
}

fn lib_name() -> &'static str {
    if is_emul32() { "lib32" } else { "lib" }
}

fn usr_lib_dir() -> String {
    format!("/usr/{}", lib_name())
}

/// Verilen flag'ler içinde belirtilen prefix ile başlayan bir arg var mı?
fn has_flag(args: &[&str], prefix: &str) -> bool {
    args.iter().any(|a| a.starts_with(prefix))
}

/// Varsayılan arg listesinden, kullanıcının verdiği flag'lerle çakışanları temizler.
/// Örn: kullanıcı `--libexecdir=/usr/lib/lvm2` verdiyse, default olarak eklenmiş
/// `--libexecdir=...` satırını temizler.
fn strip_conflicting(defaults: &mut Vec<String>, args: &[&str], prefixes: &[&str]) {
    defaults.retain(|d| {
        !prefixes.iter().any(|p| {
            d.starts_with(p) && has_flag(args, p)
        })
    });
}

/// Autotools tabanlı derleme sistemleri için temel configure çağrısını yapar.
pub fn autotools_configure(args: &[&str]) -> Result<(), String> {
    let sbindir_val = env::var("PISI_SBINDIR").unwrap_or_else(|_| "usr/bin".to_string());
    let mut default_args = vec!["--prefix=/usr".to_string(), "--sysconfdir=/etc".to_string()];

    if !has_flag(args, "--sbindir") {
        default_args.push(format!(
            "--sbindir=/{}",
            sbindir_val.trim_start_matches('/')
        ));
    }

    if !has_flag(args, "--with-systemdsystemunitdir") {
        default_args.push("--with-systemdsystemunitdir=no".to_string());
    }

    if !has_flag(args, "--libexecdir") {
        if let Ok(pkg_name) = env::var("SRC_NAME") {
            default_args.push(format!("--libexecdir=/usr/lib/{}", pkg_name));
        }
    }

    if is_emul32() && !has_flag(args, "--libdir") {
        default_args.push(format!("--libdir={}", usr_lib_dir()));
    }

    if let Ok(chost) = env::var("CHOST") {
        if !has_flag(args, "--host=") {
            default_args.push(format!("--host={}", chost));
        }
    }

    // Varsayılanları çakışma kontrolünden geçir: kullanıcının verdiği flag varsa default'u kaldır
    strip_conflicting(&mut default_args, args, &[
        "--prefix", "--sysconfdir", "--sbindir",
        "--with-systemdsystemunitdir", "--libexecdir", "--libdir", "--host=",
    ]);

    for arg in args {
        default_args.push(arg.to_string());
    }

    let default_args_refs: Vec<&str> = default_args.iter().map(|s| s.as_str()).collect();
    println!(
        "{}",
        t!("api_run_config", args = default_args_refs.join(" "))
    );
    run_command("./configure", &default_args_refs)
}

/// Autotools 'make' (derleme) adımını uygular.
pub fn autotools_make(args: &[&str]) -> Result<(), String> {
    let jobs = super::get::make_jobs();
    let mut default_args = vec!["-j", jobs.as_str()];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_run_make", args = default_args.join(" ")));
    run_command("make", &default_args)
}

/// Autotools 'make install' (kurulum) adımını uygular.
pub fn autotools_install(dest_dir: &str, args: &[&str]) -> Result<(), String> {
    let jobs = super::get::make_jobs();
    let destdir_arg = format!("DESTDIR={}", dest_dir);
    let mut default_args = vec!["-j", jobs.as_str(), "install", destdir_arg.as_str()];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_run_make_install", dest = dest_dir));
    run_command("make", &default_args)
}

/// Autotools 'make' ham kurulum adımı (rawInstall muadili).
/// Args doğrudan make'e iletilir (hedef de args içinde belirtilmeli).
/// Örn: raw_install(["install"]) veya raw_install(["modules_install"])
pub fn raw_install(args: &[&str]) -> Result<(), String> {
    let jobs = super::get::make_jobs();
    let mut default_args = vec!["-j", jobs.as_str()];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_run_make", args = default_args.join(" ")));
    run_command("make", &default_args)
}

/// Autotools 'libtoolize' adımını uygular.
pub fn libtoolize(args: &[&str]) -> Result<(), String> {
    let mut default_args = vec!["--copy", "--force"];
    default_args.extend_from_slice(args);
    println!("Running libtoolize {}", default_args.join(" "));
    run_command("libtoolize", &default_args)
}

/// Ham configure çağrısı (autotools.rawConfigure muadili).
pub fn raw_configure(args: &[&str]) -> Result<(), String> {
    let mut default_args: Vec<&str> = Vec::new();
    default_args.extend_from_slice(args);
    println!(
        "{}",
        t!("api_running_config", args = default_args.join(" "))
    );
    run_command("./configure", &default_args)
}

/// aclocal komutu (autotools.aclocal muadili).
pub fn aclocal(args: &[&str]) -> Result<(), String> {
    let mut default_args: Vec<&str> = Vec::new();
    default_args.extend_from_slice(args);
    println!(
        "{}",
        t!("api_running_aclocal", args = default_args.join(" "))
    );
    run_command("aclocal", &default_args)
}

/// autoconf komutu (autotools.autoconf muadili).
pub fn autoconf(args: &[&str]) -> Result<(), String> {
    let mut default_args: Vec<&str> = Vec::new();
    default_args.extend_from_slice(args);
    println!(
        "{}",
        t!("api_running_autoconf", args = default_args.join(" "))
    );
    run_command("autoconf", &default_args)
}

/// autoreconf komutu (autotools.autoreconf muadili).
pub fn autoreconf(args: &[&str]) -> Result<(), String> {
    let mut default_args: Vec<&str> = Vec::new();
    default_args.extend_from_slice(args);
    println!(
        "{}",
        t!("api_running_autoreconf", args = default_args.join(" "))
    );
    run_command("autoreconf", &default_args)
}

/// automake komutu (autotools.automake muadili).
pub fn automake(args: &[&str]) -> Result<(), String> {
    let mut default_args: Vec<&str> = Vec::new();
    default_args.extend_from_slice(args);
    println!(
        "{}",
        t!("api_running_automake", args = default_args.join(" "))
    );
    run_command("automake", &default_args)
}

/// autoheader komutu (autotools.autoheader muadili).
pub fn autoheader(args: &[&str]) -> Result<(), String> {
    let mut default_args: Vec<&str> = Vec::new();
    default_args.extend_from_slice(args);
    println!(
        "{}",
        t!("api_running_autoheader", args = default_args.join(" "))
    );
    run_command("autoheader", &default_args)
}

/// CMake tabanlı derleme sistemleri için yapılandırma adımını uygular.
pub fn cmake_configure(args: &[&str]) -> Result<(), String> {
    cmake_configure_impl(args, true)
}

pub fn cmake_configure_skip_build_dir(args: &[&str]) -> Result<(), String> {
    cmake_configure_impl(args, false)
}

fn cmake_configure_impl(args: &[&str], manage_build_dir: bool) -> Result<(), String> {
    let sbindir_val = env::var("PISI_SBINDIR").unwrap_or_else(|_| "bin".to_string());
    let cmake_libdir = format!("-DCMAKE_INSTALL_LIBDIR={}", lib_name());
    let mut default_args = vec![
        "-DCMAKE_INSTALL_PREFIX=/usr".to_string(),
        "-DCMAKE_BUILD_TYPE=Release".to_string(),
        cmake_libdir,
        "-DLIB_SUFFIX=".to_string(),
        "-DCMAKE_VERBOSE_MAKEFILE=ON".to_string(),
    ];

    let has_sbindir = args.iter().any(|a| a.contains("CMAKE_INSTALL_SBINDIR"));
    if !has_sbindir {
        default_args.push(format!("-DCMAKE_INSTALL_SBINDIR={}", sbindir_val));
    }

    let has_libexecdir = args.iter().any(|a| a.contains("CMAKE_INSTALL_LIBEXECDIR"));
    if !has_libexecdir {
        if let Ok(pkg_name) = env::var("SRC_NAME") {
            default_args.push(format!("-DCMAKE_INSTALL_LIBEXECDIR=lib/{}", pkg_name));
        }
    }

    if let Ok(chost) = env::var("CHOST") {
        let target_arch = if chost.starts_with("aarch64") {
            "aarch64"
        } else if chost.starts_with("x86_64") {
            "x86_64"
        } else {
            "generic"
        };
        default_args.push("-DCMAKE_SYSTEM_NAME=Linux".to_string());
        default_args.push(format!("-DCMAKE_SYSTEM_PROCESSOR={}", target_arch));
        if let Ok(cc) = env::var("CC") {
            default_args.push(format!("-DCMAKE_C_COMPILER={}", cc));
        }
        if let Ok(cxx) = env::var("CXX") {
            default_args.push(format!("-DCMAKE_CXX_COMPILER={}", cxx));
        }
    }

    for arg in args {
        default_args.push(arg.to_string());
    }

    if !args.iter().any(|arg| arg.starts_with("..") || arg.starts_with('/')) {
        default_args.push("..".to_string());
    }

    if manage_build_dir {
        if !Path::new("build").exists() {
            fs::create_dir("build").map_err(|e| t!("api_err_build_dir", error = e).to_string())?;
        }
        cd("build")?;
    }

    println!("{}", t!("api_run_cmake"));
    let default_args_refs: Vec<&str> = default_args.iter().map(|s| s.as_str()).collect();
    run_command("cmake", &default_args_refs)
}

/// KDE Frameworks 5 tabanlı projeler için özelleştirilmiş CMake yapılandırması.
pub fn kde5_configure(args: &[&str]) -> Result<(), String> {
    let kde_libdir = format!("-DKDE_INSTALL_LIBDIR={}", lib_name());
    let mut kde_args = vec![
        "-DKDE_INSTALL_USE_QT_SYS_PATHS=ON",
        kde_libdir.as_str(),
        "-DBUILD_TESTING=OFF",
    ];
    kde_args.extend_from_slice(args);
    cmake_configure(&kde_args)
}

/// KDE Frameworks 6 tabanlı projeler için özelleştirilmiş CMake yapılandırması.
pub fn kde6_configure(args: &[&str]) -> Result<(), String> {
    let kde_libdir = format!("-DKDE_INSTALL_LIBDIR={}", lib_name());
    let mut kde_args = vec![
        "-DKDE_INSTALL_USE_QT_SYS_PATHS=ON",
        kde_libdir.as_str(),
        "-DBUILD_QT6=ON",
        "-DBUILD_TESTING=OFF",
    ];
    kde_args.extend_from_slice(args);
    cmake_configure(&kde_args)
}

/// Meson tabanlı derleme sistemleri için yapılandırma adımını uygular.
pub fn meson_configure(args: &[&str]) -> Result<(), String> {
    let meson_libdir = format!("--libdir={}", lib_name());
    let mut default_args = vec![
        "setup".to_string(),
        "builddir".to_string(),
        "--prefix=/usr".to_string(),
        "--sysconfdir=/etc".to_string(),
        "--buildtype=plain".to_string(),
        meson_libdir,
    ];

    let has_sbindir = args.iter().any(|a| a.starts_with("--sbindir"));
    if !has_sbindir {
        let sbindir_val = env::var("PISI_SBINDIR").unwrap_or_else(|_| "bin".to_string());
        default_args.push(format!("--sbindir={}", sbindir_val));
    }

    let has_libexecdir = args.iter().any(|a| a.starts_with("--libexecdir"));
    if !has_libexecdir {
        if let Ok(pkg_name) = env::var("SRC_NAME") {
            default_args.push(format!("--libexecdir=lib/{}", pkg_name));
        }
    }

    for arg in args {
        default_args.push(arg.to_string());
    }

    let refs: Vec<&str> = default_args.iter().map(|s| s.as_str()).collect();
    println!("{}", t!("api_run_meson", args = refs.join(" ")));
    run_command("meson", &refs)
}

/// Ninja derleme sistemini kullanarak projeyi derler.
pub fn ninja_build(args: &[&str]) -> Result<(), String> {
    let jobs = super::get::make_jobs();
    let mut default_args = vec!["-C", "builddir", "-j", jobs.as_str()];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_run_ninja", args = default_args.join(" ")));
    run_command("ninja", &default_args)
}

/// Ninja derleme sistemini kullanarak projeyi kurar.
pub fn ninja_install(dest_dir: &str, args: &[&str]) -> Result<(), String> {
    pisi_core::safe_env::set_var("DESTDIR", dest_dir);
    let jobs = super::get::make_jobs();
    let mut default_args = vec!["-C", "builddir", "-j", jobs.as_str(), "install"];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_run_ninja_install", dest = dest_dir));
    let res = run_command("ninja", &default_args);
    pisi_core::safe_env::remove_var("DESTDIR");
    res
}

/// Qt5 projeleri için yapılandırma (qmake-qt5) adımını uygular.
pub fn qt5_configure(args: &[&str]) -> Result<(), String> {
    let qt_libdir = format!("LIBDIR={}", usr_lib_dir());
    let mut default_args = vec![
        "PREFIX=/usr",
        qt_libdir.as_str(),
        "QMAKE_CFLAGS+='-O2'",
        "QMAKE_CXXFLAGS+='-O2'",
    ];
    default_args.extend_from_slice(args);
    println!(
        "{}",
        t!(
            "api_run_config",
            args = format!("qmake-qt5 {}", default_args.join(" "))
        )
    );
    run_command("qmake-qt5", &default_args)
}

/// Qt6 projeleri için yapılandırma (qmake6) adımını uygular.
pub fn qt6_configure(args: &[&str]) -> Result<(), String> {
    let qt_libdir = format!("LIBDIR={}", usr_lib_dir());
    let mut default_args = vec!["PREFIX=/usr", qt_libdir.as_str()];
    default_args.extend_from_slice(args);
    println!(
        "{}",
        t!(
            "api_run_config",
            args = format!("qmake6 {}", default_args.join(" "))
        )
    );
    run_command("qmake6", &default_args)
}

/// Python 2 modülleri için yapılandırma adımını (setup.py configure) uygular.
pub fn python2_setup_configure(args: &[&str]) -> Result<(), String> {
    let mut default_args = vec!["setup.py", "configure"];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_python_config"));
    run_command("python", &default_args)
}

/// Python 2 modülleri için inşa adımını (setup.py build) uygular.
pub fn python2_setup_build(args: &[&str]) -> Result<(), String> {
    let mut default_args = vec!["setup.py", "build"];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_python2_build"));
    run_command("python", &default_args)
}

/// Python 2 modülleri için kurulum adımını (setup.py install) uygular.
pub fn python2_setup_install(dest_dir: &str, args: &[&str]) -> Result<(), String> {
    let root_arg = format!("--root={}", dest_dir);
    let mut default_args = vec![
        "setup.py",
        "install",
        root_arg.as_str(),
        "--prefix=/usr",
        "--optimize=1",
    ];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_python2_install", dest = dest_dir));
    run_command("python", &default_args)
}

/// Python 3 modülleri için inşa adımını (python -m build --wheel --no-isolation) uygular.
pub fn python3_setup_build(args: &[&str]) -> Result<(), String> {
    let mut default_args = vec!["-m", "build", "--wheel", "--no-isolation"];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_python_build"));
    run_command("python3", &default_args)
}

/// Python 3 modülleri için kurulum adımını uygular.
/// Önce dist/*.whl (pip/build ile derlenmiş) arar, bulamazsa setup.py install (orijinal Pisi) dener.
pub fn python3_setup_install(dest_dir: &str, args: &[&str]) -> Result<(), String> {
    let destdir_arg = format!("--destdir={}", dest_dir);

    if let Some(wheel) = glob::glob("dist/*.whl")
        .ok()
        .and_then(|mut g| g.find_map(std::result::Result::ok))
        .map(|p| p.to_string_lossy().to_string())
    {
        let mut default_args = vec!["-m", "installer", destdir_arg.as_str(), wheel.as_str()];
        default_args.extend_from_slice(args);
        println!("{}", t!("api_python_install", dest = dest_dir));
        return run_command("python3", &default_args);
    }

    // Hiç .whl yoksa setup.py install dene (orijinal Pisi uyumluluğu)
    let root_arg = format!("--root={}", dest_dir);
    let mut setup_args = vec!["setup.py", "install", root_arg.as_str()];
    setup_args.extend_from_slice(args);
    println!("{}", t!("api_python_install", dest = dest_dir));
    run_command("python3", &setup_args)
}

/// Python 3 modülleri için configure adımını (python -m build --wheel --no-isolation configure) uygular.
pub fn python3_setup_configure(args: &[&str]) -> Result<(), String> {
    let mut default_args = vec!["-m", "build", "--wheel", "--no-isolation", "configure"];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_python_config"));
    run_command("python3", &default_args)
}

/// Python modülleri için derlenmiş .pyc ve .pyo dosyalarını temizler.
pub fn python_fix_compiled_py(look_into: Option<String>) -> Result<(), String> {
    let install_dir = super::get::install_dir();
    let look_path = match look_into {
        Some(p) => format!("{}/{}/", install_dir, p.trim_start_matches('/')),
        None => format!("{}/usr/lib/python3*/", install_dir),
    };

    let glob_pattern = format!("{}**/*.pyc", look_path);
    if let Ok(entries) = glob::glob(&glob_pattern) {
        for entry in entries.flatten() {
            let _ = std::fs::remove_file(&entry);
        }
    }

    let glob_pattern2 = format!("{}**/*.pyo", look_path);
    if let Ok(entries) = glob::glob(&glob_pattern2) {
        for entry in entries.flatten() {
            let _ = std::fs::remove_file(&entry);
        }
    }

    Ok(())
}

/// Waf tabanlı derleme sistemini çalıştırır (python waf build).
pub fn waf_build(args: &[&str]) -> Result<(), String> {
    let jobs = super::get::make_jobs();
    let mut default_args = vec!["waf", "build", "-j", jobs.as_str()];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_run_waf", args = default_args.join(" ")));
    run_command("python3", &default_args)
}

/// Waf tabanlı kurulum adımını çalıştırır (python waf install).
pub fn waf_install(dest_dir: &str, args: &[&str]) -> Result<(), String> {
    let destdir_arg = format!("--destdir={}", dest_dir);
    let mut default_args = vec!["waf", "install", &destdir_arg];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_run_waf_install", dest = dest_dir));
    run_command("python3", &default_args)
}

/// Ant/Java tabanlı derleme sistemini çalıştırır.
pub fn ant_build(args: &[&str]) -> Result<(), String> {
    let mut default_args = vec!["jar"];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_run_ant", args = default_args.join(" ")));
    run_command("ant", &default_args)
}

/// Ant/Java tabanlı kurulum adımını çalıştırır.
pub fn ant_install(dest_dir: &str, args: &[&str]) -> Result<(), String> {
    let destdir_arg = format!("DESTDIR={}", dest_dir);
    let mut default_args = vec!["install", "-D", &destdir_arg];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_run_ant_install", dest = dest_dir));
    run_command("ant", &default_args)
}

/// Npm tabanlı derleme sistemini çalıştırır.
pub fn npm_build(args: &[&str]) -> Result<(), String> {
    let mut default_args = vec!["run", "build"];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_run_npm_build"));
    run_command("npm", &default_args)
}

/// Npm tabanlı kurulum adımını çalıştırır.
pub fn npm_install(dest_dir: &str, args: &[&str]) -> Result<(), String> {
    let prefix_arg = format!("--prefix={}", dest_dir);
    let mut default_args = vec!["install", "-g", &prefix_arg];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_run_npm_install", dest = dest_dir));
    run_command("npm", &default_args)
}

/// Go tabanlı derleme sistemini çalıştırır.
pub fn go_build(args: &[&str]) -> Result<(), String> {
    let mut default_args = vec!["build", "-v"];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_run_go_build"));
    run_command("go", &default_args)
}

/// Go tabanlı kurulum adımını çalıştırır.
pub fn go_install(dest_dir: &str, args: &[&str]) -> Result<(), String> {
    let mut default_args = vec!["install", "-v"];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_run_go_install", dest = dest_dir));
    // Go install için GOPATH ayarlanır
    pisi_core::safe_env::set_var("GOPATH", dest_dir);
    let result = run_command("go", &default_args);
    pisi_core::safe_env::remove_var("GOPATH");
    result
}

/// SCons tabanlı derleme sistemini çalıştırır.
pub fn scons_build(args: &[&str]) -> Result<(), String> {
    let jobs = super::get::make_jobs();
    let mut default_args = vec!["-j", jobs.as_str()];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_run_scons", args = default_args.join(" ")));
    run_command("scons", &default_args)
}

/// SCons tabanlı kurulum adımını çalıştırır.
pub fn scons_install(dest_dir: &str, args: &[&str]) -> Result<(), String> {
    let jobs = super::get::make_jobs();
    let destdir_arg = format!("DESTDIR={}", dest_dir);
    let mut default_args = vec!["-j", jobs.as_str(), "install", &destdir_arg];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_run_scons_install", dest = dest_dir));
    run_command("scons", &default_args)
}

/// Perl modülleri (Makefile.PL) için yapılandırma adımını uygular.
pub fn perl_makefile_configure(args: &[&str]) -> Result<(), String> {
    let cflags = super::get::cflags();
    let mut default_args = vec![
        "Makefile.PL",
        "PREFIX=/usr",
        "INSTALLDIRS=vendor",
        "MAN1DIR=/usr/share/man/man1",
        "MAN3DIR=/usr/share/man/man3",
    ];
    let optimize_arg = format!("OPTIMIZE={}", cflags);
    if !cflags.is_empty() {
        default_args.push(&optimize_arg);
    }
    default_args.extend_from_slice(args);
    println!("{}", t!("api_perl_config"));
    run_command("perl", &default_args)
}

/// Perl modülleri için kurulum adımını uygular.
pub fn perl_makefile_install(dest_dir: &str, args: &[&str]) -> Result<(), String> {
    let destdir_arg = format!("DESTDIR={}", dest_dir);
    let jobs = super::get::make_jobs();
    let mut default_args = vec!["-j", jobs.as_str(), "install", destdir_arg.as_str()];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_perl_install"));
    run_command("make", &default_args)
}

/// Linux Kernel modülleri için derleme adımını uygular.
pub fn kernel_module_build(kernel_ver: &str, args: &[&str]) -> Result<(), String> {
    let kdir = format!("/lib/modules/{}/build", kernel_ver);
    // SYSSRC: kernel module Makefile'larının ihtiyaç duyduğu kaynak dizini
    if std::env::var("SYSSRC").is_err() {
        pisi_core::safe_env::set_var("SYSSRC", &kdir);
    }
    let jobs = super::get::make_jobs();
    let mut default_args = vec![
        "-j",
        jobs.as_str(),
        "-C",
        kdir.as_str(),
        "M=$(pwd)",
        "modules",
    ];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_kernel_build", version = kernel_ver));
    run_command("make", &default_args)
}

/// Linux Kernel modülleri için kurulum adımını uygular.
pub fn kernel_module_install(
    kernel_ver: &str,
    dest_dir: &str,
    args: &[&str],
) -> Result<(), String> {
    let kdir = format!("/lib/modules/{}/build", kernel_ver);
    // SYSSRC: kernel module Makefile'larının ihtiyaç duyduğu kaynak dizini
    if std::env::var("SYSSRC").is_err() {
        pisi_core::safe_env::set_var("SYSSRC", &kdir);
    }
    let inst_dir = format!("{}/lib/modules/{}/extra", dest_dir, kernel_ver);
    let destdir_arg = format!("INSTALL_MOD_PATH={}", dest_dir);
    let jobs = super::get::make_jobs();
    if !Path::new(&inst_dir).exists() {
        fs::create_dir_all(&inst_dir)
            .map_err(|e| t!("api_err_kernel_dir", error = e).to_string())?;
    }
    let mut default_args = vec![
        "-j",
        jobs.as_str(),
        "-C",
        kdir.as_str(),
        "M=$(pwd)",
        destdir_arg.as_str(),
        "modules_install",
    ];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_kernel_install"));
    run_command("make", &default_args)
}

fn ensure_cargo_home() -> Result<String, String> {
    let cargo_home = env::current_dir()
        .map_err(|e| e.to_string())?
        .join(".cargo");
    if !cargo_home.exists() {
        fs::create_dir_all(&cargo_home).map_err(|e| e.to_string())?;
    }
    let home_str = cargo_home.to_string_lossy().to_string();
    set_env("CARGO_HOME", &home_str);
    Ok(home_str)
}

/// cargo fetch --locked
pub fn cargo_fetch(args: &[&str]) -> Result<(), String> {
    ensure_cargo_home()?;
    let mut default_args = vec!["fetch", "--locked"];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_cargo_fetch"));
    run_command("cargo", &default_args)
}

/// cargo build --release --jobs N [extra_args]
pub fn cargo_build(args: &[&str]) -> Result<(), String> {
    ensure_cargo_home()?;
    let jobs = super::get::make_jobs();
    let mut default_args = vec!["build", "--release", "--jobs", jobs.as_str()];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_cargo_build"));
    run_command("cargo", &default_args)
}

/// cargo test --release [extra_args]
pub fn cargo_test(args: &[&str]) -> Result<(), String> {
    ensure_cargo_home()?;
    let jobs = super::get::make_jobs();
    let mut default_args = vec!["test", "--release", "--jobs", jobs.as_str()];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_cargo_test"));
    run_command("cargo", &default_args)
}

/// cargo install --path . --root DEST/usr --jobs N [extra_args]
pub fn cargo_install(dest_dir: &str, args: &[&str]) -> Result<(), String> {
    ensure_cargo_home()?;
    let install_root = Path::new(dest_dir).join("usr");
    let root_str = install_root.to_string_lossy();
    let jobs = super::get::make_jobs();
    let mut default_args = vec![
        "install",
        "--jobs",
        jobs.as_str(),
        "--path",
        ".",
        "--root",
        &root_str,
    ];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_cargo_install", dest = dest_dir));
    run_command("cargo", &default_args)
}

/// Ruby Gem paketlerini yerel dizine kurar.
pub fn ruby_gem_install(dest_dir: &str, gem_file: &str, args: &[&str]) -> Result<(), String> {
    let install_dir = format!("{}/usr/lib/ruby/gems", dest_dir);
    let bin_dir = format!("{}/usr/bin", dest_dir);
    let mut default_args = vec![
        "install",
        "--local",
        "--install-dir",
        &install_dir,
        "--bindir",
        &bin_dir,
        "--no-document",
        gem_file,
    ];
    default_args.extend_from_slice(args);
    println!("{}", t!("api_ruby_gem", gem = gem_file));
    run_command("gem", &default_args)
}

/// Perl .packlist dosyalarını temizler.
pub fn remove_packlist(dest_dir: &str) -> Result<(), String> {
    let search_path = format!("{}/**/.packlist", dest_dir);
    if let Ok(entries) = glob::glob(&search_path) {
        for entry in entries.flatten() {
            let _ = std::fs::remove_file(entry);
        }
    }
    Ok(())
}

/// Perl modülleri için .pod dosyalarını bulup siler.
pub fn remove_podfiles(dest_dir: &str) -> Result<(), String> {
    let search_path = format!("{}/**/*.pod", dest_dir);
    if let Ok(entries) = glob::glob(&search_path) {
        for entry in entries.flatten() {
            let _ = std::fs::remove_file(entry);
        }
    }
    Ok(())
}
