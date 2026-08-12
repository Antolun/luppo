#![allow(non_snake_case)]
use pyo3::prelude::*;
use crate::actionsapi;
use crate::flags::Flags;

pub fn init_module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Flags>()?;
    m.add(
        "cflags",
        Flags {
            evars: vec!["CFLAGS".to_string()],
        },
    )?;
    m.add(
        "ldflags",
        Flags {
            evars: vec!["LDFLAGS".to_string()],
        },
    )?;
    m.add(
        "cxxflags",
        Flags {
            evars: vec!["CXXFLAGS".to_string()],
        },
    )?;
    m.add(
        "flags",
        Flags {
            evars: vec!["CFLAGS".to_string(), "CXXFLAGS".to_string()],
        },
    )?;

    // INSTALL_DIR her fonksiyonda çevre değişkeninden dinamik olarak okunur
    fn idir() -> String {
        actionsapi::get::install_dir()
    }

    #[pyfunction]
    fn dobin(path: String) -> PyResult<()> {
        actionsapi::dobin(&idir(), path).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn dosbin(path: String) -> PyResult<()> {
        actionsapi::dosbin(&idir(), path).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[pyo3(signature = (path, dest_dir = None))]
    fn dolib(path: String, dest_dir: Option<String>) -> PyResult<()> {
        let idir = idir();

        let dest_base = if let Some(ref dir) = dest_dir {
            std::path::PathBuf::from(&idir).join(dir.trim_start_matches('/'))
        } else {
            let lib_subdir = if std::env::var("PISI_BUILD_TYPE").as_deref() == Ok("emul32") {
                "usr/lib32"
            } else {
                "usr/lib"
            };
            std::path::PathBuf::from(&idir).join(lib_subdir)
        };

        std::fs::create_dir_all(&dest_base)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;

        let src = std::path::Path::new(&path);
        let has_glob = path.contains('*') || path.contains('?') || path.contains('[');
        if has_glob {
            let pattern = src.parent().unwrap_or(std::path::Path::new(".")).join(
                src.file_name().ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Geçersiz dosya adı"))?,
            );
            let entries: Vec<_> = glob::glob(&pattern.to_string_lossy())
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Glob hatası: {}", e)))?
                .filter_map(|r| r.ok())
                .collect();
            if entries.is_empty() {
                return Err(pyo3::exceptions::PyRuntimeError::new_err(format!("Eşleşen dosya yok: {}", path)));
            }
            for entry in &entries {
                actionsapi::install(entry, &dest_base)
                    .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
            }
        } else {
            actionsapi::install(src, &dest_base)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(
                    format!("Kopyalama hatası: {} -> {}: {}", path, dest_base.display(), e)
                ))?;
        }
        Ok(())
    }
    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn doman(args: Vec<String>) -> PyResult<()> {
        let idir = idir();
        for path in args {
            actionsapi::doman(&idir, path).map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
        }
        Ok(())
    }
    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn doinfo(args: Vec<String>) -> PyResult<()> {
        let idir = idir();
        for path in args {
            actionsapi::doinfo(&idir, path).map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
        }
        Ok(())
    }
    #[pyfunction]
    #[allow(non_snake_case)]
    #[pyo3(signature = (*args, destDir = None))]
    fn dodoc(args: Vec<String>, destDir: Option<String>) -> PyResult<()> {
        let idir = idir();
        let pkg_name = actionsapi::get::src_name();
        let doc_base = std::path::Path::new(&idir).join("usr/share/doc").join(&pkg_name);
        for path in args {
            if let Some(ref subdir) = destDir {
                let doc_dir = doc_base.join(subdir);
                std::fs::create_dir_all(&doc_dir)
                    .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
                actionsapi::install(std::path::Path::new(&path), &doc_dir)
                    .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
            } else {
                actionsapi::dodoc(&idir, &pkg_name, path)
                    .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
            }
        }
        Ok(())
    }
    #[pyfunction]
    #[allow(non_snake_case)]
    #[pyo3(signature = (path, search, replace = "", deleteLine = false))]
    fn dosed(path: &str, search: &str, replace: &str, deleteLine: bool) -> PyResult<()> {
        actionsapi::dosed(path, search, replace, deleteLine)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[allow(non_snake_case)]
    fn installHeaders(source: String) -> PyResult<()> {
        actionsapi::install_headers(&idir(), source)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn install(source: String, destination: String) -> PyResult<()> {
        if destination.starts_with('/') {
            let idir = idir();
            let src_path = std::path::Path::new(&source);
            let file_name = src_path
                .file_name()
                .ok_or_else(|| {
                    pyo3::exceptions::PyRuntimeError::new_err("Geçersiz kaynak dosya adı")
                })?
                .to_string_lossy();
            let dest_dir = std::path::Path::new(&idir)
                .join(destination.trim_start_matches('/').trim_end_matches('/'));
            let full_dest = dest_dir.join(&*file_name);
            actionsapi::install(source, full_dest.to_string_lossy().to_string())
                .map_err(pyo3::exceptions::PyRuntimeError::new_err)
        } else {
            actionsapi::install(source, destination)
                .map_err(pyo3::exceptions::PyRuntimeError::new_err)
        }
    }
    #[pyfunction]
    fn rename(source: String, destination: String) -> PyResult<()> {
        let src_full = format!("{}/{}", idir(), source.trim_start_matches('/'));
        let parent = std::path::Path::new(&source)
            .parent()
            .map(|p| p.to_str().unwrap_or(""))
            .unwrap_or("");
        let dst_full = format!(
            "{}/{}/{}",
            idir(),
            parent.trim_start_matches('/'),
            destination
        );
        actionsapi::move_path(src_full, dst_full).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn remove(path: String) -> PyResult<()> {
        let full_path = format!("{}/{}", idir(), path.trim_start_matches('/'));
        actionsapi::remove_path(full_path).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn dodir(dir: String) -> PyResult<()> {
        actionsapi::dodir(&idir(), &dir).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn dosym(source: String, destination: String) -> PyResult<()> {
        actionsapi::dosym(&idir(), &source, &destination)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn domove(source: String, destination: String) -> PyResult<()> {
        actionsapi::domove(&idir(), &source, &destination)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[pyo3(signature = (dest_dir, source, new_name = None))]
    fn insinto(dest_dir: String, source: String, new_name: Option<String>) -> PyResult<()> {
        actionsapi::insinto(&idir(), &dest_dir, &source)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
        if let Some(name) = new_name {
            let src_path = std::path::Path::new(&source);
            let filename = src_path.file_name().ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("Geçersiz kaynak dosya adı")
            })?;
            let old = std::path::PathBuf::from(&idir())
                .join(dest_dir.trim_start_matches('/'))
                .join(filename);
            let new = std::path::PathBuf::from(&idir())
                .join(dest_dir.trim_start_matches('/'))
                .join(&name);
            if old != new {
                std::fs::rename(&old, &new)
                    .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
            }
        }
        Ok(())
    }
    #[pyfunction]
    fn doexe(source: String, dest_dir: String) -> PyResult<()> {
        actionsapi::doexe(&idir(), source, &dest_dir)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn dolib_a(source: String, dest_dir: String) -> PyResult<()> {
        actionsapi::dolib_a(&idir(), source, &dest_dir)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn dolib_so(source: String, dest_dir: String) -> PyResult<()> {
        actionsapi::dolib_so(&idir(), source, &dest_dir)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[allow(non_snake_case)]
    fn removeDir(dir: String) -> PyResult<()> {
        actionsapi::remove_dir(&idir(), &dir).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn dopixmaps(source: String) -> PyResult<()> {
        actionsapi::dopixmaps(&idir(), source).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[allow(non_snake_case)]
    #[pyo3(signature = (*args, destDir = None))]
    fn dohtml(args: Vec<String>, destDir: Option<&str>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::dohtml(&idir(), &refs, destDir)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn domo(source: String, locale: String, destination_file: String) -> PyResult<()> {
        actionsapi::domo(&idir(), &source, &locale, &destination_file)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn newdoc(source: String, destination: String) -> PyResult<()> {
        actionsapi::newdoc(&idir(), &source, &destination)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn newman(source: String, destination: String) -> PyResult<()> {
        actionsapi::newman(&idir(), &source, &destination)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    m.add_function(wrap_pyfunction!(dobin, m)?)?;
    m.add_function(wrap_pyfunction!(dosbin, m)?)?;
    m.add_function(wrap_pyfunction!(dolib, m)?)?;
    m.add_function(wrap_pyfunction!(doman, m)?)?;
    m.add_function(wrap_pyfunction!(doinfo, m)?)?;
    m.add_function(wrap_pyfunction!(dodoc, m)?)?;
    m.add_function(wrap_pyfunction!(dosed, m)?)?;
    m.add_function(wrap_pyfunction!(installHeaders, m)?)?;
    m.add_function(wrap_pyfunction!(install, m)?)?;
    m.add_function(wrap_pyfunction!(rename, m)?)?;
    m.add_function(wrap_pyfunction!(remove, m)?)?;
    m.add_function(wrap_pyfunction!(dodir, m)?)?;
    m.add_function(wrap_pyfunction!(dosym, m)?)?;
    m.add_function(wrap_pyfunction!(domove, m)?)?;
    m.add_function(wrap_pyfunction!(insinto, m)?)?;
    m.add_function(wrap_pyfunction!(doexe, m)?)?;
    m.add_function(wrap_pyfunction!(dolib_a, m)?)?;
    m.add_function(wrap_pyfunction!(dolib_so, m)?)?;
    m.add_function(wrap_pyfunction!(removeDir, m)?)?;
    m.add_function(wrap_pyfunction!(dopixmaps, m)?)?;
    m.add_function(wrap_pyfunction!(dohtml, m)?)?;
    m.add_function(wrap_pyfunction!(domo, m)?)?;
    m.add_function(wrap_pyfunction!(newdoc, m)?)?;
    m.add_function(wrap_pyfunction!(newman, m)?)?;
    Ok(())
}
