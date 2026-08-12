use pyo3::prelude::*;
use crate::actionsapi;

pub fn init_module(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn configure(args: Vec<String>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::qt6_configure(&refs).map_err(pyo3::exceptions::PyRuntimeError::new_err)
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
    m.add_function(wrap_pyfunction!(configure, m)?)?;
    m.add_function(wrap_pyfunction!(make, m)?)?;
    m.add_function(wrap_pyfunction!(install, m)?)?;
    Ok(())
}
