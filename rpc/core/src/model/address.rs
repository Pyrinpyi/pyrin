use crate::{RpcTransactionOutpoint, RpcUtxoEntry};
use borsh::{BorshDeserialize, BorshSerialize};
use pyo3::prelude::*;
use pyo3::pyclass;
use serde::{Deserialize, Serialize};

pub type RpcAddress = kaspa_addresses::Address;

/// Represents a UTXO entry of an address returned by the `GetUtxosByAddresses` RPC.
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct RpcUtxosByAddressesEntry {
    #[pyo3(get)]
    pub address: Option<RpcAddress>,
    #[pyo3(get)]
    pub outpoint: RpcTransactionOutpoint,
    #[pyo3(get)]
    pub utxo_entry: RpcUtxoEntry,
}

/// Represents a balance of an address returned by the `GetBalancesByAddresses` RPC.
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct RpcBalancesByAddressesEntry {
    #[pyo3(get)]
    pub address: RpcAddress,

    #[pyo3(get)]
    /// Balance of `address` if available
    pub balance: Option<u64>,
}
