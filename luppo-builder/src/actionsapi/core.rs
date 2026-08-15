use rust_i18n::t;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

/// Belirtilen komutu çalıştırır ve çıktıyı yakalar.
/// Başarılı olursa Ok(()), başarısız olursa anlamlı bir hata mesajı döndürür.
pub fn run_command(name: &str, args: &[&str]) -> Result<(), String> {
    let log_file_path = env::var("LUPPO_BUILD_LOG").ok();

    // Shell-quote each argument individually so make receives
    // CFLAGS=... as a single arg, not split on spaces.
    fn shell_quote(s: &str) -> String {
        if s.is_empty() || s.contains(' ') || s.contains('\t') || s.contains('"') || s.contains('\'')
            || s.contains('$') || s.contains('\\') || s.contains('`') || s.contains('!')
        {
            // Single-quote and escape inner single quotes
            format!("'{}'", s.replace('\'', "'\\''"))
        } else {
            s.to_string()
        }
    }

    let quoted_args: Vec<String> = args.iter().map(|a| shell_quote(a)).collect();
    let full_command = if args.is_empty() {
        name.to_string()
    } else {
        format!("{} {}", name, quoted_args.join(" "))
    };

    let log_file = log_file_path
        .and_then(|path| OpenOptions::new().create(true).append(true).open(path).ok())
        .map(|f| Arc::new(Mutex::new(f)));

    if let Some(ref f) = log_file {
        let mut f = f.lock().unwrap();
        writeln!(f, "\n[RUN]: {}", full_command).ok();
    }

    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&full_command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("TERM", "xterm-256color")
        .env("CLICOLOR_FORCE", "1")
        .env("MESON_COLOR", "always")
        .env("CMAKE_COLOR_DIAGNOSTICS", "ON")
        .env("CMAKE_COLOR_MAKEFILE", "ON")
        .env("GCC_COLORS", "1")
        .env("CARGO_TERM_COLOR", "always")
        .spawn()
        .map_err(|e| t!("api_err_spawn", name = name, error = e).to_string())?;

    let stdout_pipe = child.stdout.take().ok_or("Stdout pipe alınamadı")?;
    let stderr_pipe = child.stderr.take().ok_or("Stderr pipe alınamadı")?;

    let is_verbose = env::var("VERBOSE").is_ok() || env::var("LUPPO_DEBUG").is_ok();
    let log_file_stdout = log_file.clone();
    let log_file_stderr = log_file.clone();

    let stdout_thread = thread::spawn(move || {
        let reader = BufReader::new(stdout_pipe);
        let mut output = String::new();
        for l in reader.lines().map_while(Result::ok) {
            if is_verbose {
                println!("{}", l);
            }
            if let Some(ref f) = log_file_stdout {
                let mut f = f.lock().unwrap();
                writeln!(f, "{}", l).ok();
            }
            output.push_str(&l);
            output.push('\n');
        }
        output
    });

    let stderr_thread = thread::spawn(move || {
        let reader = BufReader::new(stderr_pipe);
        let mut output = String::new();
        for l in reader.lines().map_while(Result::ok) {
            eprintln!("{}", l);
            if let Some(ref f) = log_file_stderr {
                let mut f = f.lock().unwrap();
                writeln!(f, "{}", l).ok();
            }
            output.push_str(&l);
            output.push('\n');
        }
        output
    });

    let stdout_str = stdout_thread.join().unwrap_or_default();
    let stderr_str = stderr_thread.join().unwrap_or_default();

    let status = child.wait().map_err(|e| e.to_string())?;
    if !status.success() {
        let error_msg = if !stderr_str.is_empty() {
            stderr_str.to_string()
        } else {
            stdout_str.to_string()
        };
        return Err(format!(
            "Command '{}' failed (Exit Code: {}). Error: {}",
            full_command,
            status.code().unwrap_or(-1),
            error_msg.trim()
        ));
    }

    Ok(())
}

/// Mevcut çalışma dizinini değiştirir (shelltools.cd muadili).
pub fn cd<P: AsRef<std::path::Path>>(path: P) -> Result<(), String> {
    let p = path.as_ref();
    println!("{}", t!("api_cd", path = p.display()));
    env::set_current_dir(p).map_err(|e| t!("api_err_cd", path = p.display(), error = e).to_string())
}

/// COMAR betikleri için gerekli sistem çevre değişkenlerini hazırlar.
pub fn setup_comar_env(pkg_name: &str, dest_root: &str) {
    set_env("LUPPO_PACKAGE_NAME", pkg_name);
    set_env("LUPPO_DESTDIR", dest_root);
    if let Ok(path) = env::var("PYTHONPATH") {
        set_env(
            "PYTHONPATH",
            &format!("{}:/usr/lib/python3/site-packages", path),
        );
    } else {
        set_env("PYTHONPATH", "/usr/lib/python3/site-packages");
    }
}

/// Bir çevre değişkenini ayarlar (variable.py ve Flags muadili).
pub fn set_env(key: &str, value: &str) {
    println!("{}", t!("api_set_env", key = key, value = value));
    luppo_core::safe_env::set_var(key, value);
}

/// İnşa süreci için kritik olan 'make' ve 'patch' komutlarının sistemde olup olmadığını denetler.
pub fn check_required_tools() -> Result<(), String> {
    for tool in &["make", "patch"] {
        if Command::new(tool).arg("--version").output().is_err() {
            return Err(t!("api_err_tool_not_found", tool = tool).to_string());
        }
    }
    println!("{}", t!("api_tools_verified"));
    Ok(())
}

/// Çalışma dizinini değiştirir (shelltools.cd muadili) - deprecated alias.
pub fn chdir<P: AsRef<std::path::Path>>(path: P) -> Result<(), String> {
    cd(path)
}

/// Kaynak dosyadan hedef dizine dosya veya dizin kopyalar/taşır.
pub fn install<P: AsRef<Path>, D: AsRef<Path>>(source: P, destination: D) -> Result<(), String> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    println!(
        "{}",
        t!(
            "api_install",
            src = source.display(),
            dest = destination.display()
        )
    );
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    if source.is_dir() {
        run_command(
            "cp",
            &[
                "-pPR",
                source.to_str().unwrap(),
                destination.to_str().unwrap(),
            ],
        )?;
    } else {
        let final_dest = if destination.is_dir() {
            destination.join(source.file_name().ok_or("Geçersiz kaynak dosya")?)
        } else {
            destination.to_path_buf()
        };

        if let Some(parent) = final_dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::copy(source, &final_dest)
            .map_err(|e| t!("api_err_copy", src = source.display(), error = e).to_string())?;
    }

    Ok(())
}

/// CFLAGS, CXXFLAGS ve LDFLAGS ortam değişkenlerini ayarlar.
pub fn export_flags() {
    set_env("CFLAGS", &super::get::cflags());
    set_env("CXXFLAGS", &super::get::cxxflags());
    set_env("LDFLAGS", &super::get::ldflags());
}

/// Sembolik bağ (symlink) oluşturur.
pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(source: P, link_name: Q) -> Result<(), String> {
    let src = source.as_ref();
    let dst = link_name.as_ref();
    println!(
        "{}",
        t!("api_symlink", src = src.display(), dest = dst.display())
    );

    #[cfg(unix)]
    return std::os::unix::fs::symlink(src, dst)
        .map_err(|e| t!("api_err_symlink", error = e).to_string());

    #[cfg(not(unix))]
    Err(t!("api_err_symlink_unix").to_string())
}

/// Dosya veya dizin izinlerini (chmod) ayarlar.
pub fn set_perms<P: AsRef<Path>>(path: P, mode: u32) -> Result<(), String> {
    let p = path.as_ref();
    println!(
        "{}",
        t!(
            "api_perms",
            path = p.display(),
            mode = format!("{:o}", mode)
        )
    );

    let path_str = p.to_string_lossy();
    if path_str.contains('*') || path_str.contains('?') {
        let mut matched_any = false;
        if let Ok(entries) = glob::glob(&path_str) {
            for entry_path in entries.flatten() {
                matched_any = true;
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&entry_path, fs::Permissions::from_mode(mode)).map_err(
                    |e| t!("api_err_perms", path = entry_path.display(), error = e).to_string(),
                )?;
            }
        }
        if !matched_any {
            return Err(t!(
                "api_err_perms",
                path = p.display(),
                error = "No matching files found for glob"
            )
            .to_string());
        }
        Ok(())
    } else {
        use std::os::unix::fs::PermissionsExt;

        if p.is_symlink() && !p.exists() {
            return Ok(());
        }

        fs::set_permissions(p, fs::Permissions::from_mode(mode))
            .map_err(|e| t!("api_err_perms", path = p.display(), error = e).to_string())
    }
}

/// Bir dosyayı veya dizini yeni bir konuma taşır veya adını değiştirir.
fn rename_or_copy(src: &Path, dst: &Path) -> Result<(), String> {
    if let Err(e) = fs::rename(src, dst) {
        let errno = e.raw_os_error().unwrap_or(0);
        if errno == 16 || errno == 18 || errno == 39 {
            // EBUSY (16) or EXDEV (18) – cross-device/busy, fall back to copy+remove
            if src.is_dir() {
                copy_dir(src, dst)?;
                fs::remove_dir_all(src).map_err(|e| {
                    format!("Remove after copy error: {}: {}", src.display(), e)
                })?;
            } else {
                fs::copy(src, dst).map_err(|e| {
                    format!("Copy error: {} -> {}: {}", src.display(), dst.display(), e)
                })?;
                fs::remove_file(src).map_err(|e| {
                    format!("Remove after copy error: {}: {}", src.display(), e)
                })?;
            }
        } else {
            return Err(t!(
                "api_err_move",
                src = src.display(),
                dest = dst.display(),
                error = e
            )
            .to_string());
        }
    }
    Ok(())
}

fn copy_dir(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("mkdir error: {}: {}", dst.display(), e))?;
    for entry in src.read_dir().map_err(|e| format!("readdir error: {}: {}", src.display(), e))? {
        let entry = entry.map_err(|e| format!("entry error: {}", e))?;
        let entry_src = entry.path();
        let entry_dst = dst.join(entry.file_name().as_os_str());
        if entry_src.is_dir() {
            copy_dir(&entry_src, &entry_dst)?;
        } else {
            fs::copy(&entry_src, &entry_dst).map_err(|e| {
                format!("Copy error: {} -> {}: {}", entry_src.display(), entry_dst.display(), e)
            })?;
        }
    }
    Ok(())
}

pub fn move_path<P: AsRef<Path>, Q: AsRef<Path>>(source: P, destination: Q) -> Result<(), String> {
    let src = source.as_ref();
    let dst = destination.as_ref();
    let src_str = src.to_string_lossy();

    if src_str.contains('*') || src_str.contains('?') {
        let opts = glob::MatchOptions {
            require_literal_leading_dot: true,
            ..Default::default()
        };
        if let Ok(entries) = glob::glob_with(&src_str, opts) {
            for entry_path in entries.flatten() {
                let dest_path = dst.join(
                    entry_path.file_name().ok_or_else(|| t!("api_err_move_invalid"))?,
                );
                rename_or_copy(&entry_path, &dest_path)?;
            }
        }
        return Ok(());
    }

    let actual_dest = if dst.is_dir() {
        dst.join(src.file_name().ok_or_else(|| t!("api_err_move_invalid"))?)
    } else {
        dst.to_path_buf()
    };

    println!(
        "{}",
        t!("api_move", src = src.display(), dest = actual_dest.display())
    );

    rename_or_copy(src, &actual_dest)
}

/// Bir dosyayı veya dizini (içeriğiyle birlikte) siler.
pub fn remove_path<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let p = path.as_ref();
    println!("{}", t!("api_remove", path = p.display()));

    let path_str = p.to_string_lossy();
    if path_str.contains('*') || path_str.contains('?') {
        if let Ok(entries) = glob::glob(&path_str) {
            for entry_path in entries.flatten() {
                if entry_path.is_dir() {
                    let _ = fs::remove_dir_all(&entry_path);
                } else {
                    let _ = fs::remove_file(&entry_path);
                }
            }
        }
        Ok(())
    } else {
        if !p.exists() && !p.is_symlink() {
            return Ok(());
        }

        let result = if p.is_dir() {
            fs::remove_dir_all(p)
        } else {
            fs::remove_file(p)
        };

        result.map_err(|e| {
            t!(
                "api_err_remove",
                path = p.display(),
                error = if cfg!(unix) {
                    use std::os::unix::fs::PermissionsExt;
                    format!(
                        "{} (perms: {:o})",
                        e,
                        fs::metadata(p).map(|m| m.permissions().mode()).unwrap_or(0)
                    )
                } else {
                    e.to_string()
                }
            )
            .to_string()
        })
    }
}
