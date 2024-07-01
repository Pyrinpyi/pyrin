mod wallet;
mod bip32;
mod account;
mod rpc;
mod rpc_types;
mod rpc_core;

use pyo3::prelude::*;
use pyo3::types::PyFunction;
use account::{PyAccount, PyBalance};
use kaspa_wallet_core::utils::{kaspa_to_sompi, sompi_to_kaspa};

#[pyfunction]
fn to_leor(value: f64) -> PyResult<u64> {
    Ok(kaspa_to_sompi(value))
}

#[pyfunction]
fn from_leor(value: u64) -> PyResult<f64> {
    Ok(sompi_to_kaspa(value))
}

#[pyfunction]
fn call_with_callback(callback: Py<PyFunction>, data: &PyAny) -> PyResult<()> {
    Python::with_gil(|py| {
        let callback = callback.as_ref(py);
        callback.call1((data,)).map_err(|e| e.print(py)).ok();
        Ok(())
    })
}

#[pymodule]
fn pyrin(_py: Python, m: &PyModule) -> PyResult<()> {

    m.add_function(wrap_pyfunction!(to_leor, m)?)?;
    m.add_function(wrap_pyfunction!(from_leor, m)?)?;

    m.add_class::<wallet::Wallet>()?;
    m.add_class::<PyAccount>()?;
    m.add_class::<PyBalance>()?;
    m.add_class::<bip32::Bip32>()?;
    m.add_class::<rpc::RPC>()?;

    m.add_function(wrap_pyfunction!(call_with_callback, m)?)?;

    Ok(())
}
