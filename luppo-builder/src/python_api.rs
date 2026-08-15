#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::actionsapi;
use pyo3::prelude::*;



// ROOT MODÜL — luppo.actionsapi olarak Python'a açılır
#[pymodule]
#[pyo3(name = "actionsapi")]
pub fn init_actionsapi_module(py: Python, m: &PyModule) -> PyResult<()> {
    let get_mod = PyModule::new(py, "get")?;
    actionsapi::get_py::init_module(py, get_mod)?;
    m.add_submodule(get_mod)?;

    let shelltools_mod = PyModule::new(py, "shelltools")?;
    actionsapi::shelltools::init_module(py, shelltools_mod)?;
    m.add_submodule(shelltools_mod)?;

    let luppotools_mod = PyModule::new(py, "luppotools")?;
    actionsapi::luppotools::init_module(py, luppotools_mod)?;
    m.add_submodule(luppotools_mod)?;

    let autotools_mod = PyModule::new(py, "autotools")?;
    actionsapi::autotools::init_module(py, autotools_mod)?;
    m.add_submodule(autotools_mod)?;

    let cmaketools_mod = PyModule::new(py, "cmaketools")?;
    actionsapi::cmaketools::init_module(py, cmaketools_mod)?;
    m.add_submodule(cmaketools_mod)?;

    let mesontools_mod = PyModule::new(py, "mesontools")?;
    actionsapi::mesontools::init_module(py, mesontools_mod)?;
    m.add_submodule(mesontools_mod)?;

    let pythonmodules_mod = PyModule::new(py, "pythonmodules")?;
    actionsapi::pythonmodules::init_module(py, pythonmodules_mod)?;
    m.add_submodule(pythonmodules_mod)?;

    let python3modules_mod = PyModule::new(py, "python3modules")?;
    actionsapi::python3modules::init_module(py, python3modules_mod)?;
    m.add_submodule(python3modules_mod)?;

    let qt5_mod = PyModule::new(py, "qt5")?;
    actionsapi::qt5::init_module(py, qt5_mod)?;
    m.add_submodule(qt5_mod)?;

    let qt6_mod = PyModule::new(py, "qt6")?;
    actionsapi::qt6::init_module(py, qt6_mod)?;
    m.add_submodule(qt6_mod)?;

    let kde6_mod = PyModule::new(py, "kde6")?;
    actionsapi::kde6::init_module(py, kde6_mod)?;
    m.add_submodule(kde6_mod)?;

    let sconstools_mod = PyModule::new(py, "sconstools")?;
    actionsapi::sconstools::init_module(py, sconstools_mod)?;
    m.add_submodule(sconstools_mod)?;

    let cargotools_mod = PyModule::new(py, "cargotools")?;
    actionsapi::cargotools::init_module(py, cargotools_mod)?;
    m.add_submodule(cargotools_mod)?;

    let perlmodules_mod = PyModule::new(py, "perlmodules")?;
    actionsapi::perlmodules::init_module(py, perlmodules_mod)?;
    m.add_submodule(perlmodules_mod)?;

    let kerneltools_mod = PyModule::new(py, "kerneltools")?;
    actionsapi::kerneltools::init_module(py, kerneltools_mod)?;
    m.add_submodule(kerneltools_mod)?;

    let waftools_mod = PyModule::new(py, "waftools")?;
    actionsapi::waftools::init_module(py, waftools_mod)?;
    m.add_submodule(waftools_mod)?;

    let antools_mod = PyModule::new(py, "anttools")?;
    actionsapi::anttools::init_module(py, antools_mod)?;
    m.add_submodule(antools_mod)?;

    let npmtools_mod = PyModule::new(py, "npmtools")?;
    actionsapi::npmtools::init_module(py, npmtools_mod)?;
    m.add_submodule(npmtools_mod)?;

    let gotools_mod = PyModule::new(py, "gotools")?;
    actionsapi::gotools::init_module(py, gotools_mod)?;
    m.add_submodule(gotools_mod)?;

    let libtools_mod = PyModule::new(py, "libtools")?;
    actionsapi::libtools::init_module(py, libtools_mod)?;
    m.add_submodule(libtools_mod)?;

    Ok(())
}
