use std::str::FromStr;

use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;
use pyo3::pyclass;
use pyo3::types::{PyDict, PyList};

use kaspa_consensus_core::{BlueWorkType, Hash};
use kaspa_consensus_core::header::Header;
use kaspa_consensus_core::tx::{TransactionId};
use kaspa_muhash::Blake2Hash as Blake2Hash;
use kaspa_rpc_core::{RpcBlock, RpcScriptPublicKey, RpcSubnetworkId, RpcTransaction, RpcTransactionInput, RpcTransactionOutpoint, RpcTransactionOutput};

use crate::rpc_core::RpcCore;

macro_rules! get_dict_item {
    ($dict:expr, $key:expr, $parent:expr) => {{
        let full_key = if let Some(parent) = $parent {
            format!("{}.{}", parent, $key)
        } else {
            $key.to_string()
        };

        $dict
            .get_item($key)?
            .ok_or(PyErr::new::<PyKeyError, _>(format!("Missing key: {}", full_key)))?
    }};
}

macro_rules! get_dict_item_as_string {
    ($dict:expr, $key:expr, $parent:expr, $func:expr, $error:expr) => {{
        let item = get_dict_item!($dict, $key, $parent);
        let item_str = item.to_string();
        let item_str = item_str.as_str();

        $func(item_str).map_err(|_| PyErr::new::<PyKeyError, _>($error))
    }};
}

macro_rules! get_dict_item_as_hash {
    ($dict:expr, $key:expr, $parent:expr) => {
        get_dict_item_as_string!($dict, $key, $parent, Hash::from_str, "Invalid hash")
    };
}

macro_rules! get_dict_item_as_blake2_hash {
    ($dict:expr, $key:expr, $parent:expr) => {
        get_dict_item_as_string!($dict, $key, $parent, Blake2Hash::from_str, "Invalid Blake2 hash")
    };
}

macro_rules! get_dict_item_as_vec_vec_hash {
    ($dict:expr, $key:expr, $parent:expr) => {{
        let item = get_dict_item!($dict, $key, $parent);
        let item_list = item.downcast::<PyList>()?;

        let mut vec_of_vecs: Vec<Vec<Hash>> = Vec::new();
        for sublist in item_list.iter() {
            let sublist = sublist.downcast::<PyList>()?;
            let mut vec: Vec<Hash> = Vec::new();
            for hash in sublist.iter() {
                let hash_str = hash.to_string();
                let hash = Hash::from_str(hash_str.as_str())
                    .map_err(|_| PyErr::new::<PyKeyError, _>("Invalid hash in nested list"))?;
                vec.push(hash);
            }
            vec_of_vecs.push(vec);
        }
        vec_of_vecs
    }};
}

macro_rules! get_dict_item_as_vec_u8 {
    ($dict:expr, $key:expr, $parent:expr) => {{
        let item = get_dict_item!($dict, $key, $parent);
        let item_list = item.downcast::<PyList>()?;

        let mut vec: Vec<u8> = Vec::new();
        for byte in item_list.iter() {
            let byte = byte.extract::<u8>()
                .map_err(|_| PyErr::new::<PyKeyError, _>("Invalid byte in list"))?;
            vec.push(byte);
        }
        vec
    }};
}

pub fn py_rpc_transaction_type(transaction: &PyAny) -> PyResult<RpcTransaction> {
    let transaction_dict = transaction.downcast::<PyDict>()?;
    let parent = Some("transactions");
    let inputs = RpcCore::get_dict_item(transaction_dict, "inputs").downcast::<PyList>()?;
    let inputs = inputs.iter().map(|input| {
        let parent = Some("transactions.inputs");
        let input = input.downcast::<PyDict>()?;
        let previous_outpoint = get_dict_item!(input, "previous_outpoint", Some("transactions.inputs.previous_outpoint")).downcast::<PyDict>()?;
        let transaction_id = TransactionId::from_str(
            get_dict_item!(previous_outpoint, "transaction_id", parent).to_string().as_str()
        ).map_err(|_| PyErr::new::<PyKeyError, _>("Invalid transactions.inputs.previous_outpoint.transaction_id"))?;

        Ok(RpcTransactionInput {
            previous_outpoint: RpcTransactionOutpoint {
                transaction_id,
                index: get_dict_item!(previous_outpoint, "index", Some("transactions.inputs.previous_outpoint")).extract::<u32>()?,
            },
            signature_script: get_dict_item_as_vec_u8!(input, "signature_script", parent),
            sequence: get_dict_item!(input, "sequence", parent).extract::<u64>()?,
            sig_op_count: get_dict_item!(input, "sig_op_count", parent).extract::<u8>()?,
            verbose_data: None, // TODO: tx.get_item("verbose_data")?,
        })
    }).collect::<PyResult<Vec<RpcTransactionInput>>>()?;

    let outputs = RpcCore::get_dict_item(transaction_dict, "outputs").downcast::<PyList>()?;
    let outputs = outputs.iter().map(|output| {
        let parent = Some("transactions.outputs");
        let output = output.downcast::<PyDict>()?;
        let script_public_key = get_dict_item!(output, "script_public_key", parent).downcast::<PyDict>()?;
        let script_public_key_parent = Some("transactions.outputs.script_public_key");

        Ok(
            RpcTransactionOutput {
                value: get_dict_item!(output, "value", parent).extract::<u64>()?,
                script_public_key: RpcScriptPublicKey::from_vec(
                    get_dict_item!(script_public_key, "version", script_public_key_parent).extract::<u16>()?,
                    get_dict_item_as_vec_u8!(script_public_key, "script", script_public_key_parent)
                ),
                verbose_data: None, // TODO: tx.get_item("verbose_data")?,
            }
        )
    }).collect::<PyResult<Vec<RpcTransactionOutput>>>()?;

    Ok(
        RpcTransaction {
            version: get_dict_item!(transaction_dict, "version", parent).extract::<u16>()?,
            inputs,
            outputs,
            lock_time: get_dict_item!(transaction_dict, "lock_time", parent).extract::<u64>()?,
            subnetwork_id: RpcSubnetworkId::from_str(get_dict_item!(transaction_dict, "subnetwork_id", parent).to_string().as_str())
                .map_err(|_| PyErr::new::<PyKeyError, _>(format!("Invalid {}.subnetwork_id", parent.unwrap())))?,
            gas: get_dict_item!(transaction_dict, "gas", parent).extract::<u64>()?,
            payload: get_dict_item_as_vec_u8!(transaction_dict, "payload", parent),
            mass: get_dict_item!(transaction_dict, "mass", parent).extract::<u64>()?,
            verbose_data: None, // TODO: transaction_dict.get_item("verbose_data")?,
        }
    )
}

// fn get_dict_item_as_hash<'a>(dict: &'a PyDict, key: &str, parent: Option<&str>) -> PyResult<Hash> {
//     let full_key = if let Some(parent) = parent {
//         format!("{}.{}", parent, key)
//     } else {
//         key.to_string()
//     };
//
//     Ok(Hash::from_str(dict.get_item(key)?.ok_or(PyErr::new::<PyKeyError, _>(format!("Missing key: {}", full_key)))?.to_string().as_str())
//         .map_err(|_| PyErr::new::<PyKeyError, _>(format!("Invalid hash")))?)
// }

pub fn py_rpc_block_type(block_dict : &PyDict) -> PyResult<RpcBlock> {
    let header = RpcCore::get_dict_item(block_dict, "header").downcast::<PyDict>()?;
    let transactions = RpcCore::get_dict_item(block_dict, "transactions").downcast::<PyList>()?;
    let parent = Some("header");

    println!("asd: {:?}", RpcCore::get_dict_item(block_dict, "transactions").downcast::<PyList>()?);

    Ok(
        RpcBlock {
            header: Header {
                hash: get_dict_item_as_hash!(header, "hash", parent)?,
                version: get_dict_item!(header, "version", parent).extract::<u16>()?,
                parents_by_level: get_dict_item_as_vec_vec_hash!(header, "parents_by_level", parent),
                hash_merkle_root: get_dict_item_as_hash!(header, "hash_merkle_root", parent)?,
                accepted_id_merkle_root: get_dict_item_as_hash!(header, "accepted_id_merkle_root", parent)?,
                utxo_commitment: get_dict_item_as_blake2_hash!(header, "utxo_commitment", parent)?,
                timestamp: get_dict_item!(header, "timestamp", parent).extract::<u64>()?,
                bits: get_dict_item!(header, "bits", parent).extract::<u32>()?,
                nonce: get_dict_item!(header, "nonce", parent).extract::<u64>()?,
                daa_score: get_dict_item!(header, "daa_score", parent).extract::<u64>()?,
                blue_work: BlueWorkType::from_u64(get_dict_item!(header, "blue_work", parent).extract::<u64>()?),
                blue_score: get_dict_item!(header, "blue_score", parent).extract::<u64>()?,
                pruning_point: get_dict_item_as_hash!(header, "pruning_point", parent)?,
            },
            transactions: transactions.iter().map(|tx| py_rpc_transaction_type(tx)).collect::<PyResult<Vec<RpcTransaction>>>()?,
            // verbose_data: block_dict.get_item("verbose_data")??,
            verbose_data: None,
        }
    )
}