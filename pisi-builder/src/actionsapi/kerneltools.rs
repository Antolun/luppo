#![allow(non_snake_case)]
use pyo3::prelude::*;
use crate::actionsapi;

/// __getExtraVersion() muadili: Kaynak sürümden sadece "ekstra" kısmı çıkarır.
/// Örn: "6.12.89" → "", "6.12.89.1" → ".1", "6.12.89_rc8" → "_rc8"
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

pub fn init_module(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfunction]
    #[pyo3(name = "__getSuffix")]
    fn get_suffix() -> PyResult<String> {
        Ok(actionsapi::get::src_version())
    }

    #[pyfunction]
    #[pyo3(name = "getKernelVersion")]
    fn get_kernel_version(flavour: Option<String>) -> PyResult<String> {
        let f = flavour.unwrap_or_else(|| "kernel".to_string());
        let path = format!("/etc/kernel/{}", f);
        std::fs::read_to_string(&path)
            .map(|s| s.trim().to_string())
            .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err(
                format!("Cannot read kernel version from {}", path)
            ))
    }


    #[pyfunction]
    fn configure() -> PyResult<()> {
        use std::fs;
        let raw_arch = actionsapi::get::arch().replace("i686", "i386");
        let kernel_arch = raw_arch.replace("x86_64", "x86");
        let config_src = format!("configs/kernel-{}-config", raw_arch);

        fs::copy(&config_src, ".config")
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(
                format!("Cannot copy {}: {}", config_src, e)
            ))?;

        let version = actionsapi::get::src_version();
        let extra = kernel_extra_version(&version);
        actionsapi::install_tools::dosed("Makefile", "EXTRAVERSION =.*", &format!("EXTRAVERSION = {}", extra), false)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;

        let make_arch = format!("ARCH={}", kernel_arch);
        actionsapi::autotools_make(&[&make_arch, "oldconfig"])
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;

        let _ = actionsapi::autotools_make(&[&make_arch, "listnewconfig"]);

        Ok(())
    }

    #[pyfunction]
    #[allow(non_snake_case)]
    fn build(debugSymbols: Option<bool>) -> PyResult<()> {
        let raw_arch = actionsapi::get::arch().replace("i686", "i386");
        let kernel_arch = raw_arch.replace("x86_64", "x86");
        let make_arch = format!("ARCH={}", kernel_arch);
        if debugSymbols.unwrap_or(false) {
            actionsapi::autotools_make(&[&make_arch, "CONFIG_DEBUG_INFO=y"])
                .map_err(pyo3::exceptions::PyRuntimeError::new_err)
        } else {
            actionsapi::autotools_make(&[&make_arch])
                .map_err(pyo3::exceptions::PyRuntimeError::new_err)
        }
    }

    #[pyfunction]
    fn install() -> PyResult<()> {
        use std::fs;
        let suffix = actionsapi::get::src_version();
        let idir = actionsapi::get::install_dir();

        let kernel_dir = format!("{}/etc/kernel", idir);
        fs::create_dir_all(&kernel_dir).map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
        fs::write(format!("{}/kernel", kernel_dir), &suffix)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;

        actionsapi::filesystem::insinto(&idir, "/boot/", "arch/x86/boot/bzImage")
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
        let kimg_dest = format!("{}/boot/kernel-{}", idir, suffix);
        fs::rename(format!("{}/boot/bzImage", idir), &kimg_dest)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;

        actionsapi::raw_install(&[
            &format!("INSTALL_MOD_PATH={}/", idir),
            "DEPMOD=/bin/true",
            "modules_install",
            "mod-fw=",
        ]).map_err(pyo3::exceptions::PyRuntimeError::new_err)?;

        let _ = fs::remove_file(format!("{}/lib/modules/{}/source", idir, suffix));
        let _ = fs::remove_file(format!("{}/lib/modules/{}/build", idir, suffix));

        for f in &["Module.symvers", "System.map"] {
            fs::copy(f, format!("{}/lib/modules/{}/{}", idir, suffix, f))
                .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
        }

        for d in &["extra", "updates"] {
            actionsapi::filesystem::dodir(&idir, &format!("/lib/modules/{}/{}", suffix, d))
                .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
        }

        Ok(())
    }

    #[pyfunction]
    #[allow(non_snake_case)]
    fn installHeaders() -> PyResult<()> {
        let suffix = actionsapi::get::src_version();
        let idir = actionsapi::get::install_dir();
        let hdir = format!("usr/src/linux-headers-{}", suffix);
        let dest = format!("{}/{}", idir, hdir);

        actionsapi::core::run_command("mkdir", &["-p", &dest])
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;

        let find_cmd = format!(
            "find . -path './include/*' -prune -o -path './scripts/*' -prune -o -path './Documentation/*' -prune -o \
             -type f \\( -name 'Makefile*' -o -name 'Kconfig*' -o -name 'Kbuild*' -o -name '*.sh' -o -name '*.pl' -o -name '*.lds' \\) \
             -print | cpio -pVd --preserve-modification-time {}",
            dest
        );
        actionsapi::core::run_command(&find_cmd, &[])
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;

        actionsapi::core::run_command("cp", &["-a", "include", "scripts", "Documentation", &dest])
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;

        let _ = actionsapi::core::run_command(&format!("rm -rf {}/scripts/*.o", dest), &[]);
        let _ = actionsapi::core::run_command(&format!("rm -rf {}/scripts/*/*.o", dest), &[]);
        let _ = actionsapi::core::run_command(&format!("rm -rf {}/Documentation/DocBook", dest), &[]);

        actionsapi::core::run_command(&format!(
            "(find arch -name include -type d -print | xargs -n1 -i: find : -type f) | \
             cpio -pd --preserve-modification-time {}", dest
        ), &[]).map_err(pyo3::exceptions::PyRuntimeError::new_err)?;

        for f in &["Module.symvers", "System.map", ".config"] {
            let _ = std::fs::copy(f, format!("{}/{}", dest, f));
        }

        actionsapi::filesystem::dosym(&idir, &format!("/{}", hdir), &format!("/lib/modules/{}/build", suffix))
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
        actionsapi::filesystem::dosym(&idir, "build", &format!("/lib/modules/{}/source", suffix))
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;

        Ok(())
    }

    #[pyfunction]
    #[pyo3(signature = (excludes=None))]
    #[allow(non_snake_case)]
    fn installLibcHeaders(excludes: Option<Vec<String>>) -> PyResult<()> {
        let idir = actionsapi::get::install_dir();
        let htmp = format!("{}/tmp-headers", idir);
        let hdir = format!("{}/usr/include", idir);

        let _ = actionsapi::core::run_command("rm", &["-rf", &htmp]);
        actionsapi::core::run_command("mkdir", &["-p", &htmp, &hdir])
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;

        // Kernel zaten build edildi — mevcut .config ile doğrudan headers_install yap,
        // defconfig/mrproper gerekmez (ağaç kirli olsa bile headers_install çalışır)
        let raw_arch = actionsapi::get::arch().replace("i686", "i386");
        let kernel_arch = raw_arch.replace("x86_64", "x86");
        let o_arg = format!("O={}", htmp);
        let arch_arg = format!("ARCH={}", kernel_arch);
        let hdr_arg = format!("INSTALL_HDR_PATH={}/install", htmp);

        let work_dir = actionsapi::get::work_dir();
        let _ = actionsapi::core::run_command(&format!(
            "cp -Rv {}/linux-*/arch/x86/include/generated {}/arch/x86/include/",
            work_dir, htmp
        ), &[]);

        actionsapi::raw_install(&[&o_arg, &arch_arg, &hdr_arg, "headers_install"])
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;

        actionsapi::core::run_command(&format!(
            "cd {}/install/include && find . -name '.' -o -name '.*' -prune -o -print | \
             cpio -pVd --preserve-modification-time {}", htmp, hdir
        ), &[]).map_err(pyo3::exceptions::PyRuntimeError::new_err)?;

        let _ = actionsapi::core::run_command("rm", &["-rf", &format!("{}/sound", hdir)]);

        if let Some(exc) = excludes {
            for e in &exc {
                let _ = actionsapi::core::run_command("rm", &["-rf", &format!("{}/{}", hdir, e.trim_start_matches('/'))]);
            }
        }

        let _ = actionsapi::core::run_command("rm", &["-rf", &htmp]);

        Ok(())
    }

    m.add_function(wrap_pyfunction!(get_suffix, m)?)?;
    m.add_function(wrap_pyfunction!(get_kernel_version, m)?)?;
    m.add_function(wrap_pyfunction!(configure, m)?)?;
    m.add_function(wrap_pyfunction!(build, m)?)?;
    m.add_function(wrap_pyfunction!(install, m)?)?;
    m.add_function(wrap_pyfunction!(installHeaders, m)?)?;
    m.add_function(wrap_pyfunction!(installLibcHeaders, m)?)?;
    Ok(())
}
