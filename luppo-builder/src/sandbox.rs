// luppo-builder/src/sandbox.rs

use nix::mount::{MntFlags, MsFlags, mount, umount2};
use nix::sched::{CloneFlags, unshare};
use nix::unistd::{Gid, Uid, chroot, setgid, sethostname, setuid};
use rust_i18n::t;
use std::path::{Path, PathBuf};

rust_i18n::i18n!("../../locales", fallback = "tr");

// Düşük yetkili derleme kullanıcısı ve grubu
const BUILD_UID: Uid = Uid::from_raw(999);
const BUILD_GID: Gid = Gid::from_raw(999);
// Ancak nix'in kendisi sağlamalı.

pub struct SandboxContext {
    root_path: PathBuf,
}

impl SandboxContext {
    pub fn new(root_path: PathBuf) -> Self {
        SandboxContext { root_path }
    }

    /// Sandbox ortamını ayarlar: Namespaces yaratır, kök dizini değiştirir ve yetki düşürür.
    pub fn setup_and_run<F>(&self, build_fn: F) -> Result<(), String>
    where
        F: FnOnce() -> Result<(), String>,
    {
        // 1. Namespaces Oluşturma (PID, UTS, MOUNT)
        // Yeni bir process ID (PID), hostname (UTS) ve mount (MOUNT) alanı yaratılır.
        println!("{}", t!("sandbox_namespaces"));
        unshare(CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWNS)
            .map_err(|e| {
                if e == nix::errno::Errno::EPERM {
                    format!(
                        "{}: {}. (Tip: If you are in Docker/CI, try --no-sandbox flag)",
                        t!("sandbox_namespaces"),
                        e
                    )
                } else {
                    format!("{}: {}", t!("sandbox_namespaces"), e)
                }
            })?;

        // Yeni bir hostname ata
        sethostname("luppo-builder-box")
            .map_err(|e| t!("sandbox_hostname_err", error = e).to_string())?;

        // 2. Kök Dosya Sistemi Ayarları
        self.setup_root_filesystem()?;

        // 3. Yetki Düşürme
        // Kötü amaçlı derleme betiklerinin ana sisteme zarar vermesini engellemek için.
        println!("{}", t!("sandbox_set_user"));
        setgid(BUILD_GID).map_err(|e| t!("sandbox_gid_err", error = e).to_string())?;
        setuid(BUILD_UID).map_err(|e| t!("sandbox_uid_err", error = e).to_string())?;

        // 4. Derleme Fonksiyonunu Çalıştırma
        println!("{}", t!("sandbox_started"));
        build_fn()?;

        Ok(())
    }

    /// Chroot ve temel bind mount işlemlerini gerçekleştirir.
    fn setup_root_filesystem(&self) -> Result<(), String> {
        // chroot için önce yeni kök dizine git
        std::env::set_current_dir(&self.root_path)
            .map_err(|e| t!("sandbox_cd_err", error = e).to_string())?;

        // chroot işlemini yap
        chroot(".").map_err(|e| t!("sandbox_chroot_err", error = e).to_string())?;

        // Kök dizini yeniden mount et (Private Mounts için)
        mount(
            Some(Path::new("none")),
            Path::new("/"),
            None::<&Path>,
            MsFlags::MS_REC | MsFlags::MS_PRIVATE,
            None::<&Path>,
        )
        .map_err(|e| t!("sandbox_mount_root_err", error = e).to_string())?;

        // Gerekli sistem dizinlerini (proc, sys) bind mount etme simülasyonu
        // Gerçekte, bu dizinler farklı bayraklarla bind mount veya tmpfs olarak bağlanır.

        // Mount /proc (Procfs olmadan derleme betikleri çöker)
        mount(
            Some(Path::new("proc")),
            Path::new("/proc"),
            Some(Path::new("proc")),
            MsFlags::MS_NODEV | MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID,
            None::<&Path>,
        )
        .map_err(|e| t!("sandbox_mount_proc_err", error = e).to_string())?;

        println!("{}", t!("sandbox_proc_mounted"));

        Ok(())
    }
}

/// Sandbox'tan çıkışta /proc gibi özel mount'ları temizler.
impl Drop for SandboxContext {
    fn drop(&mut self) {
        // Ters sırada mountları ayır

        // MNT_DETACH = 0x00000002. Tipi MsFlags değil, MntFlags olmalı.
        // Ham değeri MntFlags olarak tanımlıyoruz.
        const MNT_DETACH_RAW: MntFlags = MntFlags::from_bits_truncate(0x00000002); // <<< Tip düzeltildi

        // umount2 çağrısı artık MntFlags beklediği için tip uyuşmazlığı çözüldü.
        if let Err(e) = umount2(Path::new("/proc"), MNT_DETACH_RAW) {
            eprintln!("{}", t!("sandbox_proc_unmount_err", error = e));
        }
        // Gerçekte burada chroot'tan çıkış da yapılmalıdır.
        println!("{}", t!("sandbox_cleaned"));
    }
}
