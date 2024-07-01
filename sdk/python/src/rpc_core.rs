use pyo3::{PyAny, PyErr};
use pyo3::types::PyDict;

pub struct RpcCore {

}

impl RpcCore {
    pub fn get_dict_item<'a>(dict: &'a PyDict, key: &str) -> &'a PyAny {
        dict.get_item(key).unwrap()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!("Missing key: {}", key)))
            .unwrap()
    }
}