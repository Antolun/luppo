#![allow(non_snake_case)]
use pyo3::prelude::*;
use crate::actionsapi;

pub fn init_module(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfunction]
    fn make_jobs() -> PyResult<String> {
        Ok(actionsapi::get::make_jobs())
    }
    #[pyfunction]
    fn cflags() -> PyResult<String> {
        Ok(actionsapi::get::cflags())
    }
    #[pyfunction]
    fn cxxflags() -> PyResult<String> {
        Ok(actionsapi::get::cxxflags())
    }
    #[pyfunction]
    fn ldflags() -> PyResult<String> {
        Ok(actionsapi::get::ldflags())
    }
    #[pyfunction]
    fn arch() -> PyResult<String> {
        Ok(actionsapi::get::arch())
    }
    #[pyfunction]
    fn host() -> PyResult<String> {
        Ok(actionsapi::get::host())
    }
    #[pyfunction]
    fn cc() -> PyResult<String> {
        Ok(actionsapi::get::cc())
    }
    #[pyfunction]
    fn cxx() -> PyResult<String> {
        Ok(actionsapi::get::cxx())
    }
    #[pyfunction]
    fn ar() -> PyResult<String> {
        Ok(actionsapi::get::ar())
    }
    #[pyfunction]
    fn ld() -> PyResult<String> {
        Ok(actionsapi::get::ld())
    }
    #[pyfunction]
    fn ranlib() -> PyResult<String> {
        Ok(actionsapi::get::ranlib())
    }
    #[pyfunction]
    fn nm() -> PyResult<String> {
        Ok(actionsapi::get::nm())
    }
    #[pyfunction]
    fn src_name() -> PyResult<String> {
        Ok(actionsapi::get::src_name())
    }
    #[pyfunction]
    fn src_version() -> PyResult<String> {
        Ok(actionsapi::get::src_version())
    }
    #[pyfunction]
    fn src_release() -> PyResult<String> {
        Ok(actionsapi::get::src_release())
    }
    #[pyfunction]
    fn src_tag() -> PyResult<String> {
        Ok(actionsapi::get::src_tag())
    }
    #[pyfunction]
    fn src_dir() -> PyResult<String> {
        Ok(actionsapi::get::src_dir())
    }
    #[pyfunction]
    fn install_dir() -> PyResult<String> {
        Ok(actionsapi::get::install_dir())
    }
    #[pyfunction]
    fn work_dir() -> PyResult<String> {
        Ok(actionsapi::get::work_dir())
    }
    #[pyfunction]
    fn pkg_dir() -> PyResult<String> {
        Ok(actionsapi::get::pkg_dir())
    }
    #[pyfunction]
    fn cur_dir() -> PyResult<String> {
        Ok(actionsapi::get::cur_dir())
    }
    #[pyfunction]
    fn cur_kernel() -> PyResult<String> {
        Ok(actionsapi::get::cur_kernel())
    }
    #[pyfunction]
    fn cur_python() -> PyResult<String> {
        Ok(actionsapi::get::cur_python())
    }
    #[pyfunction]
    fn cur_perl() -> PyResult<String> {
        Ok(actionsapi::get::cur_perl())
    }
    #[pyfunction]
    fn doc_dir() -> PyResult<String> {
        Ok(actionsapi::get::doc_dir())
    }
    #[pyfunction]
    fn man_dir() -> PyResult<String> {
        Ok(actionsapi::get::man_dir())
    }
    #[pyfunction]
    fn info_dir() -> PyResult<String> {
        Ok(actionsapi::get::info_dir())
    }
    #[pyfunction]
    fn data_dir() -> PyResult<String> {
        Ok(actionsapi::get::data_dir())
    }
    #[pyfunction]
    fn conf_dir() -> PyResult<String> {
        Ok(actionsapi::get::conf_dir())
    }
    #[pyfunction]
    fn libexec_dir() -> PyResult<String> {
        Ok(actionsapi::get::libexec_dir())
    }
    #[pyfunction]
    fn kde_dir() -> PyResult<String> {
        Ok(actionsapi::get::kde_dir())
    }
    #[pyfunction]
    fn qt_dir() -> PyResult<String> {
        Ok(actionsapi::get::qt_dir())
    }
    #[pyfunction]
    fn build_type() -> PyResult<String> {
        Ok(actionsapi::get::build_type())
    }
    #[pyfunction]
    fn buildTYPE() -> PyResult<String> {
        Ok(actionsapi::get::build_type())
    }
    #[pyfunction]
    fn emul32_prefix_dir() -> PyResult<String> {
        Ok(actionsapi::get::emul32_prefix_dir())
    }
    #[pyfunction]
    fn emul32_prefixDIR() -> PyResult<String> {
        Ok(actionsapi::get::emul32_prefix_dir())
    }
    #[pyfunction]
    fn installDIR() -> PyResult<String> {
        Ok(actionsapi::get::install_dir())
    }
    #[pyfunction]
    fn makeJOBS() -> PyResult<String> {
        Ok(actionsapi::get::make_jobs())
    }
    #[pyfunction]
    fn srcNAME() -> PyResult<String> {
        Ok(actionsapi::get::src_name())
    }
    #[pyfunction]
    fn srcVERSION() -> PyResult<String> {
        Ok(actionsapi::get::src_version())
    }
    #[pyfunction]
    fn srcRELEASE() -> PyResult<String> {
        Ok(actionsapi::get::src_release())
    }
    #[pyfunction]
    fn workDIR() -> PyResult<String> {
        Ok(actionsapi::get::work_dir())
    }
    #[pyfunction]
    fn env_var(key: String) -> PyResult<Option<String>> {
        Ok(actionsapi::get::env_var(&key))
    }
    #[pyfunction]
    fn ENV(key: String) -> PyResult<Option<String>> {
        Ok(actionsapi::get::env_var(&key))
    }
    #[pyfunction]
    fn exist_binary(name: String) -> PyResult<bool> {
        Ok(actionsapi::get::exist_binary(&name))
    }

    #[pyfunction]
    fn CFLAGS() -> PyResult<String> {
        Ok(actionsapi::get::cflags())
    }
    #[pyfunction]
    fn CXXFLAGS() -> PyResult<String> {
        Ok(actionsapi::get::cxxflags())
    }
    #[pyfunction]
    fn LDFLAGS() -> PyResult<String> {
        Ok(actionsapi::get::ldflags())
    }
    #[pyfunction]
    fn ARCH() -> PyResult<String> {
        Ok(actionsapi::get::arch())
    }
    #[pyfunction]
    fn HOST() -> PyResult<String> {
        Ok(actionsapi::get::host())
    }
    #[pyfunction]
    fn CHOST() -> PyResult<String> {
        Ok(actionsapi::get::host())
    }
    #[pyfunction]
    fn CC() -> PyResult<String> {
        Ok(actionsapi::get::cc())
    }
    #[pyfunction]
    fn CXX() -> PyResult<String> {
        Ok(actionsapi::get::cxx())
    }
    #[pyfunction]
    fn AR() -> PyResult<String> {
        Ok(actionsapi::get::ar())
    }
    #[pyfunction]
    fn AS() -> PyResult<String> {
        Ok(actionsapi::get::ar())
    }
    #[pyfunction]
    fn LD() -> PyResult<String> {
        Ok(actionsapi::get::ld())
    }
    #[pyfunction]
    fn RANLIB() -> PyResult<String> {
        Ok(actionsapi::get::ranlib())
    }
    #[pyfunction]
    fn NM() -> PyResult<String> {
        Ok(actionsapi::get::nm())
    }

    #[pyfunction]
    fn curDIR() -> PyResult<String> {
        Ok(actionsapi::get::cur_dir())
    }
    #[pyfunction]
    fn curKERNEL() -> PyResult<String> {
        Ok(actionsapi::get::cur_kernel())
    }
    #[pyfunction]
    fn curPYTHON() -> PyResult<String> {
        Ok(actionsapi::get::cur_python())
    }
    #[pyfunction]
    fn curPERL() -> PyResult<String> {
        Ok(actionsapi::get::cur_perl())
    }
    #[pyfunction]
    fn pkgDIR() -> PyResult<String> {
        Ok(actionsapi::get::pkg_dir())
    }
    #[pyfunction]
    fn srcTAG() -> PyResult<String> {
        Ok(actionsapi::get::src_tag())
    }
    #[pyfunction]
    fn srcDIR() -> PyResult<String> {
        Ok(actionsapi::get::src_dir())
    }
    #[pyfunction]
    fn docDIR() -> PyResult<String> {
        Ok(actionsapi::get::doc_dir())
    }
    #[pyfunction]
    fn sbinDIR() -> PyResult<String> {
        Ok(actionsapi::get::sbin_dir())
    }
    #[pyfunction]
    fn infoDIR() -> PyResult<String> {
        Ok(actionsapi::get::info_dir())
    }
    #[pyfunction]
    fn manDIR() -> PyResult<String> {
        Ok(actionsapi::get::man_dir())
    }
    #[pyfunction]
    fn dataDIR() -> PyResult<String> {
        Ok(actionsapi::get::data_dir())
    }
    #[pyfunction]
    fn confDIR() -> PyResult<String> {
        Ok(actionsapi::get::conf_dir())
    }
    #[pyfunction]
    fn libexecDIR() -> PyResult<String> {
        Ok(actionsapi::get::libexec_dir())
    }
    #[pyfunction]
    fn kdeDIR() -> PyResult<String> {
        Ok(actionsapi::get::kde_dir())
    }
    #[pyfunction]
    fn qtDIR() -> PyResult<String> {
        Ok(actionsapi::get::qt_dir())
    }

    m.add_function(wrap_pyfunction!(make_jobs, m)?)?;
    m.add_function(wrap_pyfunction!(cflags, m)?)?;
    m.add_function(wrap_pyfunction!(cxxflags, m)?)?;
    m.add_function(wrap_pyfunction!(ldflags, m)?)?;
    m.add_function(wrap_pyfunction!(arch, m)?)?;
    m.add_function(wrap_pyfunction!(host, m)?)?;
    m.add_function(wrap_pyfunction!(cc, m)?)?;
    m.add_function(wrap_pyfunction!(cxx, m)?)?;
    m.add_function(wrap_pyfunction!(ar, m)?)?;
    m.add_function(wrap_pyfunction!(ld, m)?)?;
    m.add_function(wrap_pyfunction!(ranlib, m)?)?;
    m.add_function(wrap_pyfunction!(nm, m)?)?;
    m.add_function(wrap_pyfunction!(src_name, m)?)?;
    m.add_function(wrap_pyfunction!(src_version, m)?)?;
    m.add_function(wrap_pyfunction!(src_release, m)?)?;
    m.add_function(wrap_pyfunction!(src_tag, m)?)?;
    m.add_function(wrap_pyfunction!(src_dir, m)?)?;
    m.add_function(wrap_pyfunction!(install_dir, m)?)?;
    m.add_function(wrap_pyfunction!(work_dir, m)?)?;
    m.add_function(wrap_pyfunction!(pkg_dir, m)?)?;
    m.add_function(wrap_pyfunction!(cur_dir, m)?)?;
    m.add_function(wrap_pyfunction!(cur_kernel, m)?)?;
    m.add_function(wrap_pyfunction!(cur_python, m)?)?;
    m.add_function(wrap_pyfunction!(cur_perl, m)?)?;
    m.add_function(wrap_pyfunction!(doc_dir, m)?)?;
    m.add_function(wrap_pyfunction!(man_dir, m)?)?;
    m.add_function(wrap_pyfunction!(info_dir, m)?)?;
    m.add_function(wrap_pyfunction!(data_dir, m)?)?;
    m.add_function(wrap_pyfunction!(conf_dir, m)?)?;
    m.add_function(wrap_pyfunction!(libexec_dir, m)?)?;
    m.add_function(wrap_pyfunction!(kde_dir, m)?)?;
    m.add_function(wrap_pyfunction!(qt_dir, m)?)?;
    m.add_function(wrap_pyfunction!(build_type, m)?)?;
    m.add_function(wrap_pyfunction!(buildTYPE, m)?)?;
    m.add_function(wrap_pyfunction!(emul32_prefix_dir, m)?)?;
    m.add_function(wrap_pyfunction!(emul32_prefixDIR, m)?)?;
    m.add_function(wrap_pyfunction!(installDIR, m)?)?;
    m.add_function(wrap_pyfunction!(makeJOBS, m)?)?;
    m.add_function(wrap_pyfunction!(srcNAME, m)?)?;
    m.add_function(wrap_pyfunction!(srcVERSION, m)?)?;
    m.add_function(wrap_pyfunction!(srcRELEASE, m)?)?;
    m.add_function(wrap_pyfunction!(workDIR, m)?)?;
    m.add_function(wrap_pyfunction!(env_var, m)?)?;
    m.add_function(wrap_pyfunction!(exist_binary, m)?)?;
    m.add_function(wrap_pyfunction!(ENV, m)?)?;

    m.add_function(wrap_pyfunction!(CFLAGS, m)?)?;
    m.add_function(wrap_pyfunction!(CXXFLAGS, m)?)?;
    m.add_function(wrap_pyfunction!(LDFLAGS, m)?)?;
    m.add_function(wrap_pyfunction!(ARCH, m)?)?;
    m.add_function(wrap_pyfunction!(HOST, m)?)?;
    m.add_function(wrap_pyfunction!(CHOST, m)?)?;
    m.add_function(wrap_pyfunction!(CC, m)?)?;
    m.add_function(wrap_pyfunction!(CXX, m)?)?;
    m.add_function(wrap_pyfunction!(AR, m)?)?;
    m.add_function(wrap_pyfunction!(AS, m)?)?;
    m.add_function(wrap_pyfunction!(LD, m)?)?;
    m.add_function(wrap_pyfunction!(RANLIB, m)?)?;
    m.add_function(wrap_pyfunction!(NM, m)?)?;

    m.add_function(wrap_pyfunction!(curDIR, m)?)?;
    m.add_function(wrap_pyfunction!(curKERNEL, m)?)?;
    m.add_function(wrap_pyfunction!(curPYTHON, m)?)?;
    m.add_function(wrap_pyfunction!(curPERL, m)?)?;
    m.add_function(wrap_pyfunction!(pkgDIR, m)?)?;
    m.add_function(wrap_pyfunction!(srcTAG, m)?)?;
    m.add_function(wrap_pyfunction!(srcDIR, m)?)?;
    m.add_function(wrap_pyfunction!(docDIR, m)?)?;
    m.add_function(wrap_pyfunction!(sbinDIR, m)?)?;
    m.add_function(wrap_pyfunction!(infoDIR, m)?)?;
    m.add_function(wrap_pyfunction!(manDIR, m)?)?;
    m.add_function(wrap_pyfunction!(dataDIR, m)?)?;
    m.add_function(wrap_pyfunction!(confDIR, m)?)?;
    m.add_function(wrap_pyfunction!(libexecDIR, m)?)?;
    m.add_function(wrap_pyfunction!(kdeDIR, m)?)?;
    m.add_function(wrap_pyfunction!(qtDIR, m)?)?;
    Ok(())
}
