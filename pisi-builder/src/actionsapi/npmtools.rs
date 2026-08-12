use pyo3::prelude::*;
use crate::actionsapi;

pub fn init_module(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn build(args: Vec<String>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::npm_build(&refs).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    #[pyfunction]
    #[pyo3(signature = (*args))]
    fn install(args: Vec<String>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        actionsapi::npm_install(&actionsapi::get::install_dir(), &refs)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    m.add_function(wrap_pyfunction!(build, m)?)?;
    m.add_function(wrap_pyfunction!(install, m)?)?;
    Ok(())
}
