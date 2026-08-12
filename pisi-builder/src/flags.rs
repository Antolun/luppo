use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct Flags {
    pub evars: Vec<String>,
}

#[pymethods]
impl Flags {
    #[new]
    #[pyo3(signature = (*evars))]
    pub fn new(evars: Vec<String>) -> Self {
        Flags { evars }
    }

    #[pyo3(signature = (*flags))]
    pub fn add(&self, flags: Vec<String>) -> PyResult<()> {
        for evar in &self.evars {
            let current = std::env::var(evar).unwrap_or_default();
            let mut parts: Vec<String> =
                current.split_whitespace().map(|s| s.to_string()).collect();
            for flag in &flags {
                parts.push(flag.trim().to_string());
            }
            pisi_core::safe_env::set_var(evar, parts.join(" "));
        }
        Ok(())
    }

    #[pyo3(signature = (*flags))]
    pub fn remove(&self, flags: Vec<String>) -> PyResult<()> {
        for evar in &self.evars {
            let current = std::env::var(evar).unwrap_or_default();
            let flags_set: std::collections::HashSet<_> = flags.iter().map(|f| f.trim()).collect();
            let parts: Vec<_> = current
                .split_whitespace()
                .filter(|v| !flags_set.contains(v))
                .collect();
            pisi_core::safe_env::set_var(evar, parts.join(" "));
        }
        Ok(())
    }

    pub fn replace(&self, old_val: String, new_val: String) -> PyResult<()> {
        for evar in &self.evars {
            let current = std::env::var(evar).unwrap_or_default();
            let parts: Vec<String> = current
                .split_whitespace()
                .map(|v| {
                    if v == old_val {
                        new_val.clone()
                    } else {
                        v.to_string()
                    }
                })
                .collect();
            pisi_core::safe_env::set_var(evar, parts.join(" "));
        }
        Ok(())
    }

    pub fn sub(&self, pattern: String, repl: String) -> PyResult<()> {
        let re = regex::Regex::new(&pattern)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        for evar in &self.evars {
            let current = std::env::var(evar).unwrap_or_default();
            let new_val = re.replace_all(&current, &repl as &str).to_string();
            pisi_core::safe_env::set_var(evar, new_val);
        }
        Ok(())
    }

    pub fn reset(&self) -> PyResult<()> {
        for evar in &self.evars {
            pisi_core::safe_env::set_var(evar, "");
        }
        Ok(())
    }
}
