#![allow(non_snake_case)]
use pyo3::prelude::*;
use crate::actionsapi;

pub fn init_module(_py: Python, m: &PyModule) -> PyResult<()> {
    fn idir() -> String {
        actionsapi::get::install_dir()
    }

    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn configure(args: Vec<String>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::autotools_configure(&refs).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[allow(non_snake_case)]
    #[pyo3(signature = (*args))]
    fn rawConfigure(args: Vec<String>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::raw_configure(&refs).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn make(args: Vec<String>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::autotools_make(&refs).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[allow(non_snake_case)]
    #[pyo3(signature = (parameters=String::new(), argument=String::from("install")))]
    fn rawInstall(parameters: String, argument: String) -> PyResult<()> {
        let args = if parameters.is_empty() {
            vec![argument]
        } else {
            vec![parameters, argument]
        };
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::raw_install(&refs).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn install(args: Vec<String>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::autotools_install(&idir(), &refs)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn aclocal(args: Vec<String>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::aclocal(&refs).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn autoconf(args: Vec<String>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::autoconf(&refs).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn libtoolize(args: Vec<String>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::libtoolize(&refs).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn autoreconf(args: Vec<String>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::autoreconf(&refs).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn automake(args: Vec<String>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::automake(&refs).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn autoheader(args: Vec<String>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::autoheader(&refs).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[allow(non_snake_case)]
    fn fixInfoDir() -> PyResult<()> {
        actionsapi::fix_info_dir(&idir()).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    fn gnuconfig_update() -> PyResult<()> {
        actionsapi::gnuconfig_update().map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    m.add_function(wrap_pyfunction!(configure, m)?)?;
    m.add_function(wrap_pyfunction!(rawConfigure, m)?)?;
    m.add_function(wrap_pyfunction!(make, m)?)?;
    m.add_function(wrap_pyfunction!(rawInstall, m)?)?;
    m.add_function(wrap_pyfunction!(install, m)?)?;
    m.add_function(wrap_pyfunction!(aclocal, m)?)?;
    m.add_function(wrap_pyfunction!(autoconf, m)?)?;
    m.add_function(wrap_pyfunction!(libtoolize, m)?)?;
    m.add_function(wrap_pyfunction!(autoreconf, m)?)?;
    m.add_function(wrap_pyfunction!(automake, m)?)?;
    m.add_function(wrap_pyfunction!(autoheader, m)?)?;
    m.add_function(wrap_pyfunction!(fixInfoDir, m)?)?;
    m.add_function(wrap_pyfunction!(gnuconfig_update, m)?)?;
    Ok(())
}
