use crate::{RpcTransactionOutpoint, RpcUtxoEntry};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
#[cfg(not(target_family = "wasm"))]
use pyo3::prelude::*;
#[cfg(not(target_family = "wasm"))]
use pyo3::pyclass;

pub type RpcAddress = kaspa_addresses::Address;

/// Represents a UTXO entry of an address returned by the `GetUtxosByAddresses` RPC.
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(not(target_family = "wasm"))]
#[pyclass]
pub struct RpcUtxosByAddressesEntry {
    #[pyo3(get)]
    pub address: Option<RpcAddress>,
    #[pyo3(get)]
    pub outpoint: RpcTransactionOutpoint,
    #[pyo3(get)]
    pub utxo_entry: RpcUtxoEntry,
}

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(target_family = "wasm")]
pub struct RpcUtxosByAddressesEntry {
    pub address: Option<RpcAddress>,
    pub outpoint: RpcTransactionOutpoint,
    pub utxo_entry: RpcUtxoEntry,
}

/// Represents a balance of an address returned by the `GetBalancesByAddresses` RPC.
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(not(target_family = "wasm"))]
#[pyclass]
pub struct RpcBalancesByAddressesEntry {
    #[pyo3(get)]
    pub address: RpcAddress,

    #[pyo3(get)]
    /// Balance of `address` if available
    pub balance: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(target_family = "wasm")]
pub struct RpcBalancesByAddressesEntry {
    pub address: RpcAddress,
    /// Balance of `address` if available
    pub balance: Option<u64>,
}
