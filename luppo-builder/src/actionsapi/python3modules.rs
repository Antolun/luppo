#![allow(non_snake_case)]
use pyo3::prelude::*;
use crate::actionsapi;

fn resolve_pyver(py: Python, pyver: Option<PyObject>) -> i32 {
    match pyver {
        Some(ref v) => {
            if let Ok(n) = v.extract::<i32>(py) {
                n
            } else if let Ok(s) = v.extract::<String>(py) {
                s.parse::<i32>().unwrap_or(3)
            } else {
                3
            }
        }
        None => 3,
    }
}

pub fn init_module(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfunction]
    #[pyo3(signature = (*args, pyVer=None))]
    fn build(py: Python, args: Vec<String>, pyVer: Option<PyObject>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        if resolve_pyver(py, pyVer) == 2 {
            actionsapi::python2_setup_build(&refs)
        } else {
            actionsapi::python3_setup_build(&refs)
        }
        .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    #[pyo3(signature = (*args, pyVer=None))]
    fn install(py: Python, args: Vec<String>, pyVer: Option<PyObject>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        if resolve_pyver(py, pyVer) == 2 {
            actionsapi::python2_setup_install(&actionsapi::get::install_dir(), &refs)
        } else {
            actionsapi::python3_setup_install(&actionsapi::get::install_dir(), &refs)
        }
        .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    #[pyo3(signature = (*args, pyVer=None))]
    fn compile(py: Python, args: Vec<String>, pyVer: Option<PyObject>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        if resolve_pyver(py, pyVer) == 2 {
            actionsapi::python2_setup_build(&refs)
        } else {
            actionsapi::python3_setup_build(&refs)
        }
        .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    #[pyo3(signature = (*args, pyVer=None))]
    fn configure(py: Python, args: Vec<String>, pyVer: Option<PyObject>) -> PyResult<()> {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        if resolve_pyver(py, pyVer) == 2 {
            actionsapi::python2_setup_configure(&refs)
        } else {
            actionsapi::python3_setup_configure(&refs)
        }
        .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    #[pyo3(signature = (parameters="", pyVer=None))]
    fn run(py: Python, parameters: &str, pyVer: Option<PyObject>) -> PyResult<()> {
        let py_cmd = if resolve_pyver(py, pyVer) == 2 { "python" } else { "python3" };
        actionsapi::core::run_command(py_cmd, &[parameters])
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    #[pyo3(signature = (look_into=None))]
    fn fix_compiled_py(look_into: Option<String>) -> PyResult<()> {
        actionsapi::python_fix_compiled_py(look_into)
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    m.add_function(wrap_pyfunction!(build, m)?)?;
    m.add_function(wrap_pyfunction!(install, m)?)?;
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(configure, m)?)?;
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(fix_compiled_py, m)?)?;
    Ok(())
}
