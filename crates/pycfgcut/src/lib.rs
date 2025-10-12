use std::path::PathBuf;

use cfgcut::{
    Anonymization, CfgcutError, CommentHandling, OutputMode, RunRequest, TokenDestination,
    TokenRecord, run,
};
use pyo3::Bound;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};

#[pyfunction]
#[pyo3(signature = (matches, inputs, with_comments = false, quiet = false, anonymize = false, tokens = false, tokens_out = None))]
#[allow(clippy::too_many_arguments)]
fn run_cfg(
    py: Python<'_>,
    matches: Vec<String>,
    inputs: Vec<String>,
    with_comments: bool,
    quiet: bool,
    anonymize: bool,
    tokens: bool,
    tokens_out: Option<String>,
) -> PyResult<PyObject> {
    if matches.is_empty() {
        return Err(PyRuntimeError::new_err(
            "at least one match expression is required",
        ));
    }
    if inputs.is_empty() {
        return Err(PyRuntimeError::new_err(
            "at least one input path is required",
        ));
    }

    let paths = inputs.into_iter().map(PathBuf::from).collect::<Vec<_>>();
    let token_output = tokens_out
        .map(PathBuf::from)
        .map(TokenDestination::File)
        .or_else(|| tokens.then_some(TokenDestination::Stdout));

    let request = RunRequest::builder()
        .matches(matches)
        .comment_handling(if with_comments {
            CommentHandling::Include
        } else {
            CommentHandling::Exclude
        })
        .output_mode(if quiet {
            OutputMode::Quiet
        } else {
            OutputMode::Normal
        })
        .anonymization(if anonymize {
            Anonymization::Enabled
        } else {
            Anonymization::Disabled
        })
        .inputs(paths)
        .token_output(token_output)
        .build();

    match run(&request) {
        Ok(result) => {
            if let Some(TokenDestination::File(path)) = request.token_output()
                && let Err(err) = write_tokens_to_file(path, &result.tokens)
            {
                return Err(PyRuntimeError::new_err(err.to_string()));
            }

            let dict = PyDict::new(py);
            dict.set_item("stdout", result.stdout)?;
            dict.set_item("matched", result.matched)?;
            dict.set_item("tokens", tokens_to_py(py, &result.tokens)?)?;
            #[allow(deprecated)]
            {
                Ok(dict.into_py(py))
            }
        }
        Err(err) => Err(PyRuntimeError::new_err(err.to_string())),
    }
}

fn tokens_to_py(py: Python<'_>, tokens: &[TokenRecord]) -> PyResult<Vec<PyObject>> {
    tokens
        .iter()
        .map(|record| {
            let dict = PyDict::new(py);
            dict.set_item("dialect", format!("{:?}", record.dialect))?;
            dict.set_item("path", record.path.clone())?;
            dict.set_item("kind", record.kind.as_str())?;
            dict.set_item("original", record.original.clone())?;
            dict.set_item("anonymized", record.anonymized.clone())?;
            dict.set_item("line", record.line)?;
            #[allow(deprecated)]
            {
                Ok(dict.into_py(py))
            }
        })
        .collect()
}

fn write_tokens_to_file(path: &PathBuf, tokens: &[TokenRecord]) -> Result<(), CfgcutError> {
    if tokens.is_empty() {
        return Ok(());
    }
    let mut file = std::fs::File::create(path).map_err(|source| CfgcutError::Io {
        path: path.clone(),
        source,
    })?;
    for record in tokens {
        let line = serde_json::to_string(record)
            .map_err(|err| CfgcutError::InvalidArguments(err.to_string()))?;
        use std::io::Write;
        file.write_all(line.as_bytes())
            .map_err(|source| CfgcutError::Io {
                path: path.clone(),
                source,
            })?;
        file.write_all(b"\n").map_err(|source| CfgcutError::Io {
            path: path.clone(),
            source,
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::PyDict;

    fn fixture_path(rel: &str) -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(rel)
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn run_cfg_smoke_test() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let result = run_cfg(
                py,
                vec!["interfaces|>>|".to_string()],
                vec![fixture_path("juniper_junos/sample.conf")],
                false,
                false,
                false,
                false,
                None,
            )
            .expect("run_cfg succeeds");

            let dict = result
                .bind(py)
                .downcast::<PyDict>()
                .expect("result converts to dictionary");
            let matched = dict
                .get_item("matched")
                .expect("matched lookup succeeded")
                .expect("matched key exists")
                .extract::<bool>()
                .expect("matched is boolean");
            assert!(matched);
        });
    }
}

#[pymodule]
fn pycfgcut(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_cfg, m)?)?;
    let version = env!("CARGO_PKG_VERSION");
    m.add("__version__", version)?;
    m.add("__all__", vec!["run_cfg"])?;
    Ok(())
}
