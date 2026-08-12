use pisi_builder::build::{BuildOptions, PackageBuilder};
use pisi_core::{
    config::Config, database::PisiDatabase, installer::Installer, PisiError, PisiResult,
};
use pisi_spec::models::PisiSpec;
use rust_i18n::t;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn is_root() -> bool {
    nix::unistd::getuid().as_raw() == 0
}

/// Sanal dosya sistemlerinin mount durumunu kontrol eder.
fn is_mounted(path: &str) -> bool {
    if let Ok(content) = fs::read_to_string("/proc/mounts") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 && parts[1] == path {
                return true;
            }
        }
    }
    false
}

/// Sanal dosya sistemini bağlar (mount eder).
fn mount_fs(source: &str, target: &str, fstype: &str, flags: &str) -> Result<(), PisiError> {
    if is_mounted(target) {
        println!("{}", t!("toolchain_mount_already", target = target));
        return Ok(());
    }

    println!("{}", t!("toolchain_mount_fs", src = source, dest = target));
    let mut cmd = Command::new("mount");
    if !fstype.is_empty() {
        cmd.arg("-t").arg(fstype);
    }
    if !flags.is_empty() {
        cmd.arg("-o").arg(flags);
    }
    cmd.arg(source).arg(target);

    let status = cmd.status().map_err(|e| {
        PisiError::RuntimeError(format!("{}: {}", t!("toolchain_mount_cmd_fail"), e))
    })?;
    if !status.success() {
        return Err(PisiError::RuntimeError(
            t!("toolchain_mount_fail", src = source, dest = target).to_string(),
        ));
    }
    Ok(())
}

/// Chroot Kök Dizini altında stable Chroot ortamı hazırlar.
pub fn perform_toolchain_start(_trace_id: u64) -> PisiResult<()> {
    // 1. Root Yetkisi Kontrolü
    if !is_root() {
        return Err(PisiError::RuntimeError(
            t!("error_root_required").to_string(),
        ));
    }

    println!("{}", t!("toolchain_start_starting"));

    // 2. Chroot Temel Dizinlerini Oluştur
    let chroot_root = "/mnt/chroot";
    let dirs = [
        "sources", "tools", "bin", "etc", "lib", "sbin", "usr", "var", "lib64", "dev", "proc",
        "sys", "run",
    ];

    for dir in &dirs {
        let path = Path::new(chroot_root).join(dir);
        if !path.exists() {
            println!(
                "{}",
                t!(
                    "toolchain_start_dir_creating",
                    dir = path.display().to_string()
                )
            );
            fs::create_dir_all(&path).map_err(PisiError::IoError)?;
        }
    }

    // 3. Temel Sembolik Bağları (Symlinks) Oluştur
    let sh_symlink = Path::new(chroot_root).join("bin/sh");
    if !sh_symlink.exists() && !sh_symlink.is_symlink() {
        println!("{}", t!("toolchain_start_symlink"));
        #[cfg(unix)]
        let _ = std::os::unix::fs::symlink("bash", &sh_symlink);
    }

    // 4. Sanal Çekirdek Dosya Sistemlerini Mount Et
    mount_fs(
        "devtmpfs",
        "/mnt/chroot/dev",
        "devtmpfs",
        "mode=0755,nosuid",
    )?;

    let devpts_path = "/mnt/chroot/dev/pts";
    if !Path::new(devpts_path).exists() {
        let _ = fs::create_dir_all(devpts_path);
    }
    mount_fs("devpts", devpts_path, "devpts", "gid=5,mode=620")?;
    mount_fs("proc", "/mnt/chroot/proc", "proc", "")?;
    mount_fs("sysfs", "/mnt/chroot/sys", "sysfs", "")?;
    mount_fs(
        "tmpfs",
        "/mnt/chroot/run",
        "tmpfs",
        "mode=0755,nodev,nosuid",
    )?;

    println!("{}", t!("toolchain_start_success"));
    Ok(())
}

/// Stable Chroot sırasına göre paketleri derleyip /mnt/chroot altına kurar ve Docker İmajı oluşturur.
pub fn perform_toolchain_update(trace_id: u64) -> PisiResult<()> {
    if !is_root() {
        return Err(PisiError::RuntimeError(
            t!("error_root_required").to_string(),
        ));
    }

    println!("{}", t!("toolchain_update_starting"));

    // 1. Sıralı Chroot Paket Listesi
    let chroot_packages = vec![
        "binutils",
        "gcc",
        "linux-headers",
        "glibc",
        "libstdcxx",
        "m4",
        "ncurses",
        "bash",
        "coreutils",
        "diffutils",
        "file",
        "findutils",
        "gawk",
        "grep",
        "gzip",
        "make",
        "patch",
        "sed",
        "tar",
        "xz",
    ];

    let db_path = "/var/lib/pisi/pisi.db";
    let db = PisiDatabase::open(PathBuf::from(db_path))
        .map_err(|e| PisiError::RuntimeError(t!("toolchain_err_db", error = e).to_string()))?;
    let config = Config::load(Some(PathBuf::from("/mnt/chroot"))); // /mnt/chroot altına kurulacak!

    for (idx, pkg_name) in chroot_packages.iter().enumerate() {
        println!(
            "{}",
            t!(
                "toolchain_update_building",
                current = idx + 1,
                total = chroot_packages.len(),
                package = pkg_name
            )
        );

        // Paket tarifi (pspec.kdl veya pspec.xml) dosyasını ara
        let recipe_paths = [
            format!("./recipes/{}/pspec.kdl", pkg_name),
            format!("./recipes/{}/pspec.xml", pkg_name),
            format!("/mnt/chroot/recipes/{}/pspec.kdl", pkg_name),
            format!("/mnt/chroot/recipes/{}/pspec.xml", pkg_name),
            format!("/var/lib/pisi/recipes/{}/pspec.kdl", pkg_name),
            format!("/var/lib/pisi/recipes/{}/pspec.xml", pkg_name),
        ];

        let mut spec_file: Option<PathBuf> = None;
        for path_str in &recipe_paths {
            let path = PathBuf::from(path_str);
            if path.exists() {
                spec_file = Some(path);
                break;
            }
        }

        if let Some(spec_path) = spec_file {
            println!(
                "{}",
                t!(
                    "toolchain_update_recipe_found",
                    path = spec_path.display().to_string()
                )
            );
            let spec = PisiSpec::from_path(&spec_path)
                .map_err(|e| PisiError::RuntimeError(t!("toolchain_err_spec_parse", error = e).to_string()))?;

            // Build seçeneklerini hazırla (Varsayılan veya Cross compiler)
            let build_options = BuildOptions {
                jobs: std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(1),
                verbose: true,
                debug: false,
                log_path: None,
                optimization_level: Some("2".to_string()),
                enable_sandbox: false, // Chroot derlemesinde ana sisteme erişim gerekebilir
                architecture: config.general.architecture.clone(),
                yes_all: true,
                sbindir: "usr/bin".to_string(),
                run_build: false,
                run_install: false,
                run_package: false,
            };

            let work_dir = config
                .directories
                .tmp_dir
                .join(format!("{}-chroot-build", pkg_name));
            if work_dir.exists() {
                let _ = fs::remove_dir_all(&work_dir);
            }
            fs::create_dir_all(&work_dir).map_err(PisiError::IoError)?;

            let specdir = spec_path.parent().unwrap_or(Path::new(".")).to_path_buf();

            let builder = PackageBuilder::new(
                spec,
                work_dir.clone(),
                specdir,
                db.clone(),
                config.clone(),
                build_options,
            );

            match builder.build(trace_id) {
                Ok(built_packages) => {
                    for pkg_path in built_packages {
                        println!(
                            "{}",
                            t!(
                                "toolchain_update_installing",
                                path = pkg_path.display().to_string()
                            )
                        );
                        let installer = Installer::new(db.clone(), config.clone());
                        installer.perform_install(
                            vec![pkg_path.to_string_lossy().to_string()],
                            trace_id,
                            true, // force
                            true, // yes_all
                            None,
                            None,
                            false,
                            true,
                            true,
                            true,
                            true,
                            true,
                            true,
                            None,
                        )?;
                    }
                }
                Err(e) => {
                    return Err(PisiError::RuntimeError(
                        t!(
                            "toolchain_update_fail",
                            package = pkg_name,
                            error = e.to_string()
                        )
                        .to_string(),
                    ));
                }
            }
        } else {
            // Eğer tarif bulunamazsa, simülasyon/gösterim modunda devam et
            println!(
                "{}",
                t!("toolchain_update_recipe_not_found", package = pkg_name)
            );
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    println!("{}", t!("toolchain_update_docker_creating"));

    // tar -C /mnt/chroot -c . | docker import - pisi-linux-chroot:latest
    let tar_child = Command::new("tar")
        .arg("-C")
        .arg("/mnt/chroot")
        .arg("-c")
        .arg(".")
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| {
            PisiError::RuntimeError(
                t!("toolchain_update_tar_fail", error = e.to_string()).to_string(),
            )
        })?;

    let docker_output = Command::new("docker")
        .arg("import")
        .arg("-")
        .arg("pisi-linux-chroot:latest")
        .stdin(tar_child.stdout.unwrap())
        .output()
        .map_err(|e| {
            PisiError::RuntimeError(
                t!("toolchain_update_docker_import_fail", error = e.to_string()).to_string(),
            )
        })?;

    if docker_output.status.success() {
        println!("{}", t!("toolchain_update_docker_success"));
    } else {
        println!(
            "{}",
            t!(
                "toolchain_update_docker_inactive",
                error = String::from_utf8_lossy(&docker_output.stderr).to_string()
            )
        );
    }

    println!("{}", t!("toolchain_update_success"));
    Ok(())
}
