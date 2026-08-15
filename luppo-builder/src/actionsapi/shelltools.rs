#![allow(non_snake_case)]
use pyo3::prelude::*;
use crate::actionsapi;

pub fn init_module(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfunction]
    fn cd(path: String) -> PyResult<()> {
        actionsapi::cd(path).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn symlink(source: String, link_name: String) -> PyResult<()> {
        actionsapi::symlink(source, link_name).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn sym(source: String, destination: String) -> PyResult<()> {
        actionsapi::symlink(source, destination).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn move_(source: String, destination: String) -> PyResult<()> {
        actionsapi::move_path(source, destination)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn copy(source: String, destination: String) -> PyResult<()> {
        actionsapi::install(source, destination).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn copytree(source: String, destination: String) -> PyResult<()> {
        actionsapi::install(source, destination).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn makedirs(path: String) -> PyResult<()> {
        std::fs::create_dir_all(&path).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn unlink(path: String) -> PyResult<()> {
        actionsapi::remove_path(path).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[pyo3(signature = (path, mode = 0o755))]
    fn chmod(path: String, mode: u32) -> PyResult<()> {
        actionsapi::set_perms(path, mode).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[allow(non_snake_case)]
    fn unlinkDir(path: String) -> PyResult<()> {
        actionsapi::remove_path(path).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn export(key: String, value: String) -> PyResult<()> {
        luppo_core::safe_env::set_var(&key, &value);
        Ok(())
    }
    #[pyfunction]
    #[allow(non_snake_case)]
    fn exportFlags() -> PyResult<()> {
        actionsapi::export_flags();
        Ok(())
    }
    #[pyfunction]
    fn echo(fname: String, content: String) -> PyResult<()> {
        println!("-> echo: {} <- {}", fname, content);
        if let Some(parent) = std::path::Path::new(&fname).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("echo error: {}: {}", fname, e)))?;
        }
        std::fs::write(&fname, &content)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("echo error: {}: {}", fname, e)))
    }
    #[pyfunction]
    fn system(cmd: String) -> PyResult<i32> {
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .stdin(std::process::Stdio::null())
            .status()
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
        Ok(status.code().unwrap_or(-1))
    }
    #[pyfunction]
    fn can_access_file(path: String) -> PyResult<bool> {
        Ok(std::path::Path::new(&path).exists())
    }
    #[pyfunction]
    fn ls(path: String) -> PyResult<Vec<String>> {
        let mut result = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                result.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
        Ok(result)
    }
    #[pyfunction]
    fn touch(path: String) -> PyResult<()> {
        std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)
            .map(|_| ())
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn cat(path: String) -> PyResult<String> {
        std::fs::read_to_string(&path).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn write(path: String, content: String) -> PyResult<()> {
        std::fs::write(&path, content).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn is_file(path: String) -> PyResult<bool> {
        Ok(std::path::Path::new(&path).is_file())
    }
    #[pyfunction]
    fn is_dir(path: String) -> PyResult<bool> {
        Ok(std::path::Path::new(&path).is_dir())
    }
    #[pyfunction]
    fn is_link(path: String) -> PyResult<bool> {
        Ok(std::path::Path::new(&path).is_symlink())
    }
    #[pyfunction]
    fn get_size(path: String) -> PyResult<u64> {
        std::fs::metadata(&path)
            .map(|m| m.len())
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    #[pyo3(signature = (path, uid = None, gid = None))]
    fn chown(path: String, uid: Option<String>, gid: Option<String>) -> PyResult<()> {
        use std::os::unix::fs::chown as unix_chown;

        let resolved_uid = if let Some(ref u) = uid {
            if let Ok(id) = u.parse::<u32>() {
                Some(id)
            } else {
                std::fs::read_to_string("/etc/passwd")
                    .ok()
                    .and_then(|pw| {
                        pw.lines().find_map(|line| {
                            let parts: Vec<&str> = line.split(':').collect();
                            if parts.len() > 2 && parts[0] == u.as_str() {
                                parts[2].parse::<u32>().ok()
                            } else {
                                None
                            }
                        })
                    })
            }
        } else {
            None
        };

        let resolved_gid = if let Some(ref g) = gid {
            if let Ok(id) = g.parse::<u32>() {
                Some(id)
            } else {
                std::fs::read_to_string("/etc/group")
                    .ok()
                    .and_then(|gr| {
                        gr.lines().find_map(|line| {
                            let parts: Vec<&str> = line.split(':').collect();
                            if parts.len() > 2 && parts[0] == g.as_str() {
                                parts[2].parse::<u32>().ok()
                            } else {
                                None
                            }
                        })
                    })
            }
        } else {
            None
        };

        unix_chown(&path, resolved_uid, resolved_gid).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    m.add_function(wrap_pyfunction!(cd, m)?)?;
    m.add_function(wrap_pyfunction!(symlink, m)?)?;
    m.add_function(wrap_pyfunction!(sym, m)?)?;
    m.add_function(wrap_pyfunction!(move_, m)?)?;
    m.add_function(wrap_pyfunction!(copy, m)?)?;
    m.add_function(wrap_pyfunction!(copytree, m)?)?;
    m.add_function(wrap_pyfunction!(makedirs, m)?)?;
    m.add_function(wrap_pyfunction!(unlink, m)?)?;
    m.add_function(wrap_pyfunction!(unlinkDir, m)?)?;
    m.add_function(wrap_pyfunction!(chmod, m)?)?;
    m.add_function(wrap_pyfunction!(chown, m)?)?;
    m.add_function(wrap_pyfunction!(export, m)?)?;
    m.add_function(wrap_pyfunction!(exportFlags, m)?)?;
    m.add_function(wrap_pyfunction!(echo, m)?)?;
    m.add_function(wrap_pyfunction!(system, m)?)?;
    m.add_function(wrap_pyfunction!(can_access_file, m)?)?;
    m.add_function(wrap_pyfunction!(ls, m)?)?;
    m.add_function(wrap_pyfunction!(touch, m)?)?;
    m.add_function(wrap_pyfunction!(cat, m)?)?;
    m.add_function(wrap_pyfunction!(write, m)?)?;
    m.add_function(wrap_pyfunction!(is_file, m)?)?;
    m.add_function(wrap_pyfunction!(is_dir, m)?)?;
    m.add_function(wrap_pyfunction!(is_link, m)?)?;
    m.add_function(wrap_pyfunction!(get_size, m)?)?;
    m.setattr("move", m.getattr("move_")?)?;
    Ok(())
}
