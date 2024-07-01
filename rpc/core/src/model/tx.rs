use borsh::{BorshDeserialize, BorshSerialize};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use kaspa_addresses::Address;
use kaspa_consensus_core::tx::{
    ScriptPublicKey, ScriptVec, TransactionId, TransactionInput, TransactionOutpoint, TransactionOutput, UtxoEntry,
};

use crate::prelude::{RpcHash, RpcScriptClass, RpcSubnetworkId};

/// Represents the ID of a Kaspa transaction
pub type RpcTransactionId = TransactionId;

pub type RpcScriptVec = ScriptVec;
pub type RpcScriptPublicKey = ScriptPublicKey;
pub type RpcUtxoEntry = UtxoEntry;

/// Represents a Kaspa transaction outpoint
pub type RpcTransactionOutpoint = TransactionOutpoint;

/// Represents a Kaspa transaction input
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct RpcTransactionInput {
    #[pyo3(get)]
    pub previous_outpoint: RpcTransactionOutpoint,
    #[serde(with = "hex::serde")]
    #[pyo3(get)]
    pub signature_script: Vec<u8>,
    #[pyo3(get)]
    pub sequence: u64,
    #[pyo3(get)]
    pub sig_op_count: u8,
    #[pyo3(get)]
    pub verbose_data: Option<RpcTransactionInputVerboseData>,
}

impl From<TransactionInput> for RpcTransactionInput {
    fn from(input: TransactionInput) -> Self {
        Self {
            previous_outpoint: input.previous_outpoint,
            signature_script: input.signature_script,
            sequence: input.sequence,
            sig_op_count: input.sig_op_count,
            verbose_data: None,
        }
    }
}

impl RpcTransactionInput {
    pub fn from_transaction_inputs(other: Vec<TransactionInput>) -> Vec<Self> {
        other.into_iter().map(Self::from).collect()
    }
}

/// Represent Kaspa transaction input verbose data
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct RpcTransactionInputVerboseData {}

/// Represents a Kaspad transaction output
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct RpcTransactionOutput {
    #[pyo3(get)]
    pub value: u64,
    #[pyo3(get)]
    pub script_public_key: RpcScriptPublicKey,
    #[pyo3(get)]
    pub verbose_data: Option<RpcTransactionOutputVerboseData>,
}

impl RpcTransactionOutput {
    pub fn from_transaction_outputs(other: Vec<TransactionOutput>) -> Vec<Self> {
        other.into_iter().map(Self::from).collect()
    }
}

impl From<TransactionOutput> for RpcTransactionOutput {
    fn from(output: TransactionOutput) -> Self {
        Self { value: output.value, script_public_key: output.script_public_key, verbose_data: None }
    }
}

/// Represent Kaspa transaction output verbose data
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct RpcTransactionOutputVerboseData {
    #[pyo3(get)]
    pub script_public_key_type: RpcScriptClass,
    #[pyo3(get)]
    pub script_public_key_address: Address,
}

/// Represents a Kaspa transaction
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct RpcTransaction {
    #[pyo3(get)]
    pub version: u16,
    #[pyo3(get)]
    pub inputs: Vec<RpcTransactionInput>,
    #[pyo3(get)]
    pub outputs: Vec<RpcTransactionOutput>,
    #[pyo3(get)]
    pub lock_time: u64,
    #[pyo3(get)]
    pub subnetwork_id: RpcSubnetworkId,
    #[pyo3(get)]
    pub gas: u64,
    #[serde(with = "hex::serde")]
    #[pyo3(get)]
    pub payload: Vec<u8>,
    #[pyo3(get)]
    pub mass: u64,
    #[pyo3(get)]
    pub verbose_data: Option<RpcTransactionVerboseData>,
}

/// Represent Kaspa transaction verbose data
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct RpcTransactionVerboseData {
    #[pyo3(get)]
    pub transaction_id: RpcTransactionId,
    #[pyo3(get)]
    pub hash: RpcHash,
    #[pyo3(get)]
    pub mass: u64,
    #[pyo3(get)]
    pub block_hash: RpcHash,
    #[pyo3(get)]
    pub block_time: u64,
}

/// Represents accepted transaction ids
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct RpcAcceptedTransactionIds {
    #[pyo3(get)]
    pub accepting_block_hash: RpcHash,
    #[pyo3(get)]
    pub accepted_transaction_ids: Vec<RpcTransactionId>,
}
