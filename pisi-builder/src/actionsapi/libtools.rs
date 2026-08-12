use pyo3::prelude::*;
use crate::actionsapi;
use std::fs;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;

pub fn init_module(_py: Python, m: &PyModule) -> PyResult<()> {

    #[pyfunction]
    #[pyo3(signature = (source_directory = "/usr/lib".to_string()))]
    fn preplib(source_directory: String) -> PyResult<()> {
        let install_dir = actionsapi::get::install_dir();
        let dir = Path::new(&install_dir)
            .join(source_directory.trim_start_matches('/'));
        actionsapi::run_command("ldconfig", &["-n", "-N", dir.to_str().unwrap()])
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    fn gnuconfig_update() -> PyResult<()> {
        actionsapi::gnuconfig_update().map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn libtoolize(args: Vec<String>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::libtoolize(&refs).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    fn gen_usr_ldscript(dynamic_lib: String) -> PyResult<()> {
        let install_dir = actionsapi::get::install_dir();
        let lib_dir = Path::new(&install_dir).join("usr/lib");
        fs::create_dir_all(&lib_dir)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;

        let path = lib_dir.join(&dynamic_lib);
        let content = format!(
            "/* GNU ld script\n\
             Since Pardus has critical dynamic libraries\n\
             in /lib, and the static versions in /usr/lib,\n\
             we need to have a \"fake\" dynamic lib in /usr/lib,\n\
             otherwise we run into linking problems.\n\
             */\n\
             GROUP ( /lib/{} )\n",
            dynamic_lib
        );
        fs::write(&path, &content)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
        Ok(())
    }

    m.add_function(wrap_pyfunction!(preplib, m)?)?;
    m.add_function(wrap_pyfunction!(gnuconfig_update, m)?)?;
    m.add_function(wrap_pyfunction!(libtoolize, m)?)?;
    m.add_function(wrap_pyfunction!(gen_usr_ldscript, m)?)?;
    Ok(())
}
