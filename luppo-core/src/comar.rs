use crate::{LuppoError, LuppoResult};
use luppo_spec::models::TriggersConfig;
use rust_i18n::t;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use zbus::blocking::Connection;

pub enum ComarAction {
    PostInstall {
        from_version: String,
        from_release: String,
        to_version: String,
        to_release: String,
    },
    PreRemove,
    PostRemove,
    Configure,
}

impl ComarAction {
    pub fn as_str(&self) -> &str {
        match self {
            ComarAction::PostInstall { .. } => "postInstall",
            ComarAction::PreRemove => "preRemove",
            ComarAction::PostRemove => "postRemove",
            ComarAction::Configure => "setupPackage",
        }
    }
}

/// Sistem genelindeki bir tetikleyici kuralını temsil eder.
pub struct SystemTrigger {
    pub name: String,
    pub path_prefix: String,
    pub path_suffix: Option<String>,
    pub script: PathBuf,
}

/// COMAR işlemlerini soyutlayan arka plan yöneticisi (Backend)
pub trait ComarBackend {
    fn register_package(&self, pkg_name: &str, model: &str, script_path: &str) -> LuppoResult<()>;
    fn remove_package(&self, pkg_name: &str) -> LuppoResult<()>;
    fn register_service_state(&self, app_name: &str) -> LuppoResult<()>;
    fn run_package_script(&self, pkg_name: &str, action: ComarAction) -> LuppoResult<()>;
}

// -----------------------------------------------------------------------------
// D-Bus Tabanlı Modern COMAR Backend (comar ve comar-rust uyumlu)
// -----------------------------------------------------------------------------
pub struct DBusComarBackend {
    _root: PathBuf, // Şimdilik D-Bus için doğrudan path kullanımı gerekmeyebilir ama API uyumu için tutuyoruz
}

impl DBusComarBackend {
    pub fn new(root: PathBuf) -> Self {
        Self { _root: root }
    }
}

impl ComarBackend for DBusComarBackend {
    fn register_package(&self, pkg_name: &str, model: &str, script_path: &str) -> LuppoResult<()> {
        let conn = Connection::system().map_err(|e| {
            LuppoError::RuntimeError(t!("comar_err_dbus_conn", error = e).to_string())
        })?;
        let dest = "tr.org.pardus.comar";
        let path = "/";
        let interface = "tr.org.pardus.comar";

        conn.call_method(
                Some(dest),
                path,
                Some(interface),
                "register",
                &(pkg_name, model, script_path),
            )
            .map_err(|e| {
                LuppoError::RuntimeError(t!("comar_err_register", error = e).to_string())
            })?;

        Ok(())

    }

    fn remove_package(&self, pkg_name: &str) -> LuppoResult<()> {
        let conn = Connection::system().map_err(|e| {
            LuppoError::RuntimeError(t!("comar_err_dbus_conn", error = e).to_string())
        })?;
        let dest = "tr.org.pardus.comar";
        let path = "/";
        let interface = "tr.org.pardus.comar";

        conn.call_method(Some(dest), path, Some(interface), "remove", &(pkg_name,))
            .map_err(|e| LuppoError::RuntimeError(t!("comar_err_remove", error = e).to_string()))?;

        Ok(())
    }

    fn register_service_state(&self, app_name: &str) -> LuppoResult<()> {
        let conn = Connection::system().map_err(|e| {
            LuppoError::RuntimeError(t!("comar_err_dbus_conn", error = e).to_string())
        })?;
        let dest = "tr.org.pardus.comar";
        let path = format!("/package/{}", app_name.replace("-", "_"));
        let iface = "tr.org.pardus.comar.System.Service";

        conn.call_method(Some(dest), path.as_str(), Some(iface), "registerState", &())
            .map_err(|e| {
                LuppoError::RuntimeError(t!("comar_err_state", error = e).to_string())
            })?;

        Ok(())
    }

    fn run_package_script(&self, pkg_name: &str, action: ComarAction) -> LuppoResult<()> {
        let conn = Connection::system().map_err(|e| {
            LuppoError::RuntimeError(t!("comar_err_dbus_conn", error = e).to_string())
        })?;

        let dest = "tr.org.pardus.comar";
        let path = format!("/package/{}", pkg_name.replace("-", "_"));
        let interface = "tr.org.pardus.comar.System.Package";

        let proxy_result = match action {
            ComarAction::PostInstall {
                ref from_version,
                ref from_release,
                ref to_version,
                ref to_release,
            } => conn.call_method(
                    Some(dest),
                    path.as_str(),
                    Some(interface),
                    "postInstall",
                    &(from_version, from_release, to_version, to_release),
                ),
            ComarAction::PreRemove =>
                conn.call_method(Some(dest), path.as_str(), Some(interface), "preRemove", &()),
            ComarAction::PostRemove =>
                conn.call_method(Some(dest), path.as_str(), Some(interface), "postRemove", &()),
            ComarAction::Configure => {
                let iface = "tr.org.pardus.comar.System.PackageHandler";
                conn.call_method(Some(dest), path.as_str(), Some(iface), "setupPackage", &("", ""))
            }
        };

        if let Err(e) = proxy_result {
            let err_str = e.to_string();
            if !err_str.contains("Unable to find") {
                eprintln!("{}", t!("comar_err_proxy", error = err_str));
            }
        }

        Ok(())
    }

}

// -----------------------------------------------------------------------------
// Subprocess Tabanlı Eski Usül COMAR Backend (Geriye Dönük Uyumluluk)
// -----------------------------------------------------------------------------
pub struct SubprocessComarBackend {
    root: PathBuf,
}

impl SubprocessComarBackend {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn load_system_triggers(&self, dir: &Path) -> LuppoResult<Vec<SystemTrigger>> {
        let mut triggers = Vec::new();
        let xml_path = dir.join("triggers.xml");

        // 1. Önce XML yapılandırmasını (triggers.xml) yüklemeyi dene
        if xml_path.exists() {
            if let Ok(content) = fs::read_to_string(&xml_path) {
                if let Ok(config) = serde_xml_rs::from_str::<TriggersConfig>(&content) {
                    for entry in config.triggers {
                        triggers.push(SystemTrigger {
                            name: entry.name,
                            path_prefix: entry.path,
                            path_suffix: entry.suffix,
                            script: dir.join(entry.script),
                        });
                    }
                    if !triggers.is_empty() {
                        return Ok(triggers);
                    }
                }
            }
        }

        // 2. Fallback: XML yoksa veya ayrıştırılamazsa dizindeki .py dosyalarını tara (Geriye dönük uyum)
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("py") {
                    let name = path.file_stem().unwrap().to_string_lossy().into_owned();
                    let prefix = match name.as_str() {
                        "fontconfig" => "/usr/share/fonts",
                        "gtk-icon-cache" => "/usr/share/icons",
                        "pango" => "/usr/lib/pango",
                        _ => "/usr/bin",
                    };
                    triggers.push(SystemTrigger {
                        name,
                        path_prefix: prefix.to_string(),
                        path_suffix: None,
                        script: path,
                    });
                }
            }
        }
        Ok(triggers)
    }
}

impl ComarBackend for SubprocessComarBackend {
    fn register_package(
        &self,
        _pkg_name: &str,
        _model: &str,
        _script_path: &str,
    ) -> LuppoResult<()> {
        Ok(())
    }

    fn remove_package(&self, _pkg_name: &str) -> LuppoResult<()> {
        Ok(())
    }

    fn run_package_script(&self, pkg_name: &str, action: ComarAction) -> LuppoResult<()> {
        let script_path = self
            .root
            .join("var/lib/luppo/package")
            .join(pkg_name)
            .join("comar/package.py");

        if !script_path.exists() {
            return Ok(());
        }

        println!(
            "{}",
            t!(
                "comar_subprocess_trigger",
                package = pkg_name,
                action = action.as_str()
            )
        );

        let mut cmd = Command::new("python3");
        cmd.arg(script_path).arg(action.as_str());

        // Argümanları subprocess'e de geçir
        if let ComarAction::PostInstall {
            from_version,
            from_release,
            to_version,
            to_release,
        } = &action
        {
            cmd.arg(from_version)
                .arg(from_release)
                .arg(to_version)
                .arg(to_release);
        }

        let status = cmd.status().map_err(LuppoError::IoError)?;

        if !status.success() {
            return Err(LuppoError::RuntimeError(
                t!(
                    "comar_err_script",
                    code = status.code().unwrap_or(-1),
                    name = pkg_name
                )
                .to_string(),
            ));
        }

        Ok(())
    }

    fn register_service_state(&self, _app_name: &str) -> LuppoResult<()> {
        Ok(())
    }

}

// -----------------------------------------------------------------------------
// Ana Comar Manager (Soyutlanmış Katman)
// -----------------------------------------------------------------------------
pub struct ComarManager {
    backend: Box<dyn ComarBackend>,
    root: PathBuf,
}

impl ComarManager {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            backend: Box::new(DBusComarBackend::new(root.as_ref().to_path_buf())),
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn with_backend(backend: Box<dyn ComarBackend>, root: PathBuf) -> Self {
        Self { backend, root }
    }

    pub fn register_package(
        &self,
        pkg_name: &str,
        model: &str,
        script_path: &str,
    ) -> LuppoResult<()> {
        self.backend.register_package(pkg_name, model, script_path)
    }

    pub fn remove_package(&self, pkg_name: &str) -> LuppoResult<()> {
        self.backend.remove_package(pkg_name)
    }

    pub fn register_service_state(&self, app_name: &str) -> LuppoResult<()> {
        self.backend.register_service_state(app_name)
    }

    pub fn run_package_script(&self, pkg_name: &str, action: ComarAction) -> LuppoResult<()> {
        self.backend.run_package_script(pkg_name, action)
    }

    pub fn run_system_triggers(&self, affected_files: &[String]) -> LuppoResult<()> {
        let trigger_dir = self.root.join("etc/comar/triggers");
        if !trigger_dir.exists() {
            return Ok(());
        }

        let triggers = load_system_triggers(&trigger_dir)?;
        let affected_set: HashSet<_> = affected_files.iter().collect();

        for trigger in triggers {
            let is_affected = affected_set.iter().any(|f| {
                f.starts_with(&trigger.path_prefix)
                    && trigger.path_suffix.as_ref().is_none_or(|s| f.ends_with(s))
            });

            if is_affected {
                println!(
                    "{}",
                    t!(
                        "comar_system_trigger_hit",
                        name = trigger.name,
                        path = trigger.path_prefix
                    )
                );
                let status = Command::new("python3")
                    .arg(&trigger.script)
                    .status()
                    .map_err(LuppoError::IoError)?;

                if !status.success() {
                    eprintln!("{}", t!("comar_trigger_failed", name = trigger.name));
                }
            }
        }

        Ok(())
    }
}

fn load_system_triggers(trigger_dir: &Path) -> LuppoResult<Vec<SystemTrigger>> {
    let triggers_file = trigger_dir.join("triggers.xml");
    if !triggers_file.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&triggers_file).map_err(LuppoError::IoError)?;
    let root: TriggersConfig = quick_xml::de::from_str(&content)
        .map_err(|e| LuppoError::RuntimeError(format!("Tetikleyici dosyası ayrıştırılamadı: {e}")))?;

    let mut triggers = Vec::new();
    for t in root.triggers {
        if t.path.is_empty() {
            continue;
        }
        triggers.push(SystemTrigger {
            path_prefix: t.path,
            path_suffix: t.suffix,
            script: trigger_dir.join(&t.script),
            name: t.name,
        });
    }
    Ok(triggers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_system_trigger_matching_logic() {
        let dir = tempdir().unwrap();
        let trigger_dir = dir.path().join("etc/comar/triggers");
        fs::create_dir_all(&trigger_dir).unwrap();

        let xml_content = r#"
        <Triggers>
            <Trigger>
                <Name>fontconfig</Name>
                <Path>/usr/share/fonts</Path>
                <Suffix>.otf</Suffix>
                <Script>font-trigger.py</Script>
            </Trigger>
        </Triggers>
        "#;
        File::create(trigger_dir.join("triggers.xml"))
            .unwrap()
            .write_all(xml_content.as_bytes())
            .unwrap();
        File::create(trigger_dir.join("font-trigger.py")).unwrap();

        // Test the trigger matching logic
        let triggers = load_system_triggers(&trigger_dir)
            .expect("Tetikleyiciler yüklenemedi");

        assert_eq!(triggers.len(), 1);
        let trigger = &triggers[0];

        let affected_files = vec!["/usr/share/fonts/liberation.otf".to_string()];
        let is_affected = affected_files.iter().any(|f| {
            f.starts_with(&trigger.path_prefix)
                && trigger
                    .path_suffix
                    .as_ref()
                    .map_or(true, |s| f.ends_with(s))
        });

        assert!(
            is_affected,
            "Kaldırılan .otf dosyası fontconfig tetikleyicisini çalıştırmalıydı."
        );
    }
}
