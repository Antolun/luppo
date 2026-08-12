#![allow(non_snake_case)]
use pyo3::prelude::*;
use crate::actionsapi;

pub fn init_module(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfunction]
    #[allow(non_snake_case)]
    #[pyo3(signature = (*args, sourceDir = ".."))]
    fn configure(args: Vec<String>, sourceDir: &str) -> PyResult<()> {
        let mut all_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        if !all_args.iter().any(|a| a.starts_with("..") || a.starts_with('/')) {
            all_args.push(sourceDir);
        }
        actionsapi::cmake_configure_skip_build_dir(&all_args).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn make(args: Vec<String>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::autotools_make(&refs).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn install(args: Vec<String>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::autotools_install(&actionsapi::get::install_dir(), &refs)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    #[allow(non_snake_case)]
    #[pyo3(signature = (*args, argument = "install"))]
    fn rawInstall(args: Vec<String>, argument: &str) -> PyResult<()> {
        let mut all_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        all_args.push(argument);
        actionsapi::autotools_make(&all_args).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    m.add_function(wrap_pyfunction!(configure, m)?)?;
    m.add_function(wrap_pyfunction!(make, m)?)?;
    m.add_function(wrap_pyfunction!(install, m)?)?;
    m.add_function(wrap_pyfunction!(rawInstall, m)?)?;
    Ok(())
}
