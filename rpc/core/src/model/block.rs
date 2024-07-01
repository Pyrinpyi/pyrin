use borsh::{BorshDeserialize, BorshSerialize};
use pyo3::pyclass;
use serde::{Deserialize, Serialize};

use crate::prelude::{RpcHash, RpcHeader, RpcTransaction};

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct RpcBlock {
    #[pyo3(get)]
    pub header: RpcHeader,
    #[pyo3(get)]
    pub transactions: Vec<RpcTransaction>,
    #[pyo3(get)]
    pub verbose_data: Option<RpcBlockVerboseData>,
}

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct RpcBlockVerboseData {
    #[pyo3(get)]
    pub hash: RpcHash,
    #[pyo3(get)]
    pub difficulty: f64,
    #[pyo3(get)]
    pub selected_parent_hash: RpcHash,
    #[pyo3(get)]
    pub transaction_ids: Vec<RpcHash>,
    #[pyo3(get)]
    pub is_header_only: bool,
    #[pyo3(get)]
    pub blue_score: u64,
    #[pyo3(get)]
    pub children_hashes: Vec<RpcHash>,
    #[pyo3(get)]
    pub merge_set_blues_hashes: Vec<RpcHash>,
    #[pyo3(get)]
    pub merge_set_reds_hashes: Vec<RpcHash>,
    #[pyo3(get)]
    pub is_chain_block: bool,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "wasm32-sdk")] {
        use wasm_bindgen::prelude::*;

        #[wasm_bindgen(typescript_custom_section)]
        const TS_BLOCK: &'static str = r#"
        /**
         * Interface defining the structure of a block.
         * 
         * @category Consensus
         */
        export interface IBlock {
            header: IHeader;
            transactions: ITransaction[];
            verboseData?: IBlockVerboseData;
        }

        /**
         * Interface defining the structure of a block verbose data.
         * 
         * @category Node RPC
         */
        export interface IBlockVerboseData {
            hash: HexString;
            difficulty: number;
            selectedParentHash: HexString;
            transactionIds: HexString[];
            isHeaderOnly: boolean;
            blueScore: number;
            childrenHashes: HexString[];
            mergeSetBluesHashes: HexString[];
            mergeSetRedsHashes: HexString[];
            isChainBlock: boolean;
        }
        "#;
    }
}
