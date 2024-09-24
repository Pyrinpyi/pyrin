use std::sync::Arc;

use serde::{Deserialize, Serialize};

use kaspa_consensus_core::subnets::SubnetworkId;
use kaspa_consensus_core::tx::{ScriptPublicKey, Transaction, TransactionInput, TransactionOutput};
use kaspa_utils::mem_size::MemSizeEstimator;
use kaspa_utils::serde_bytes;

// consensus/src/model/stores/block_transactions.rs
#[derive(Clone, Serialize, Deserialize)]
pub struct BlockBody(pub Arc<Vec<Transaction>>);

impl MemSizeEstimator for BlockBody {
    fn estimate_mem_bytes(&self) -> usize {
        const NORMAL_SIG_SIZE: usize = 66;
        let (inputs, outputs) = self.0.iter().fold((0, 0), |(ins, outs), tx| (ins + tx.inputs.len(), outs + tx.outputs.len()));
        inputs * (size_of::<TransactionInput>() + NORMAL_SIG_SIZE)
            + outputs * size_of::<TransactionOutput>()
            + self.0.len() * size_of::<Transaction>()
            + size_of::<Vec<Transaction>>()
            + size_of::<Self>()
    }
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AddressTransactionOutput {
    pub value: u64,
    pub script_public_key: ScriptPublicKey,
    pub address: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressTransaction {
    pub version: u16,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<AddressTransactionOutput>,
    pub lock_time: u64,
    pub subnetwork_id: SubnetworkId,
    pub gas: u64,
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,
    pub mass: u64,
    pub id: String,
}