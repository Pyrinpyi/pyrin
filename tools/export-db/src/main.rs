use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};

use kaspa_addresses::Prefix;
use kaspa_consensus::model::stores::ghostdag::GhostdagData;
use kaspa_consensus::model::stores::headers::HeaderWithBlockLevel;
use kaspa_consensus_core::{BlockHasher, Hash};
use kaspa_consensus_core::acceptance_data::AcceptanceData;
use kaspa_consensus_core::header::Header;
use kaspa_consensus_core::tx::Transaction;
use kaspa_database::prelude::{CachedDbAccess, CachePolicy};
use kaspa_database::registry::DatabaseStorePrefixes;
use kaspa_txscript::extract_script_pub_key_address;
use kaspa_utils::hex::ToHex;
use kaspa_utils::mem_size::MemSizeEstimator;
use types::*;

mod types;

const CONSENSUS_DB: &str = "consensus";
const UTXOINDEX_DB: &str = "utxoindex";
const META_DB: &str = "meta";

fn main() {
    // Currently only JSON supported

    let db_dir = r"D:\pyrin\pyrin-mainnet\datadir".to_string();
    let db_dir = Path::new(db_dir.as_str());

    let consensus_db_dir = db_dir.join(CONSENSUS_DB);
    let utxoindex_db_dir = db_dir.join(UTXOINDEX_DB);
    let meta_db_dir = db_dir.join(META_DB);

    let style = ProgressStyle::default_bar();

    let output_dir = PathBuf::from(r"D:\pyrin\".to_string());
    let to_address = true; // TODO: Arg
    let prefix = Prefix::Mainnet; // TODO: Allow to select (as --testnet etc, if to_address specified require it)

    let keys = vec![
        // "acceptance_data".to_string(),
        // "blocks_transactions".to_string(),
        // "headers".to_string(),
        "ghostdag".to_string(),
    ];

    for key in keys {
        let output_file = format!("{}.json", key);
        let file = File::create(output_dir.join(output_file)).unwrap();
        let mut writer = BufWriter::new(file);

        let mut acceptance_data: HashMap<String, AcceptanceData> = HashMap::new();
        let mut blocks: HashMap<String, Vec<Transaction>> = HashMap::new();
        let mut blocks_with_addresses: HashMap<String, Vec<AddressTransaction>> = HashMap::new();
        let mut headers: HashMap<String, Arc<Header>> = HashMap::new();
        let mut ghostdag: HashMap<String, Arc<GhostdagData>> = HashMap::new();

        // Go over all the consensus folders
        for entry in std::fs::read_dir(consensus_db_dir.clone()).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap().to_str().unwrap();

                let db = kaspa_database::prelude::ConnBuilder::default()
                    .with_db_path(consensus_db_dir.clone().join(dir_name))
                    .with_files_limit(1)
                    .build()
                    .unwrap();

                let bar = ProgressBar::new(0);

                match key.as_str() {
                    "acceptance_data" => {
                        let access = CachedDbAccess::<Hash, AcceptanceData>::new(db, CachePolicy::Empty, DatabaseStorePrefixes::AcceptanceData.into());
                        bar.set_length(access.iterator().count() as u64);

                        access.iterator().for_each(|entity| {
                            match entity {
                                Ok((hash, data)) => {
                                    let hash = Hash::from_slice(&hash).to_hex();
                                    acceptance_data.insert(hash, data);
                                }
                                Err(e) => {}
                            }

                            bar.inc(1);
                        });

                        bar.finish();
                    }

                    "blocks_transactions" => {
                        let access = CachedDbAccess::<Hash, BlockBody, BlockHasher>::new(db, CachePolicy::Empty, DatabaseStorePrefixes::BlockTransactions.into());
                        bar.set_length(access.iterator().count() as u64);

                        access.iterator().for_each(|entity| {
                            match entity {
                                Ok((hash, transactions)) => {
                                    // println!("[{}] {}", Hash::from_slice(&hash).to_hex(), transactions.estimate_size(MemMode::Bytes));

                                    let block_hash = Hash::from_slice(&hash).to_hex();
                                    let transactions = transactions.0.to_vec();

                                    if to_address {
                                        let mut to_address_transactions = Vec::<AddressTransaction>::new();
                                        transactions.iter().for_each(|t| {
                                            to_address_transactions.push(
                                                AddressTransaction {
                                                    version: t.clone().version,
                                                    inputs: t.clone().inputs,
                                                    outputs: t.clone().outputs.iter().map(|output| {
                                                        AddressTransactionOutput {
                                                            value: output.clone().value,
                                                            script_public_key: output.clone().script_public_key,
                                                            address: extract_script_pub_key_address(&output.script_public_key, prefix).unwrap().to_string(),
                                                        }
                                                    }).collect(),
                                                    lock_time: t.clone().lock_time,
                                                    subnetwork_id: t.clone().subnetwork_id,
                                                    gas: t.clone().gas,
                                                    payload: t.clone().payload,
                                                    mass: t.clone().mass(),
                                                    id: t.clone().id().to_hex(),
                                                }
                                            )
                                        });
                                        blocks_with_addresses.insert(block_hash, to_address_transactions);
                                    } else {
                                        blocks.insert(block_hash, transactions);
                                    }
                                }
                                Err(e) => {}
                            }

                            bar.inc(1);
                        });

                        bar.finish();
                    }

                    "headers" => {
                        let access = CachedDbAccess::<Hash, HeaderWithBlockLevel, BlockHasher>::new(db, CachePolicy::Empty, DatabaseStorePrefixes::Headers.into());
                        bar.set_length(access.iterator().count() as u64);

                        access.iterator().for_each(|entity| {
                            match entity {
                                Ok((hash, data)) => {
                                    let hash = Hash::from_slice(&hash).to_hex();
                                    headers.insert(hash, data.header);
                                }
                                Err(e) => {}
                            }

                            bar.inc(1);
                        });

                        bar.finish();
                    }

                    "ghostdag" => {
                        let access = CachedDbAccess::<Hash, Arc<GhostdagData>, BlockHasher>::new(db, CachePolicy::Empty, DatabaseStorePrefixes::Ghostdag.into());
                        bar.set_length(access.iterator().count() as u64);

                        access.iterator().for_each(|entity| {
                            match entity {
                                Ok((hash, data)) => {
                                    let prefix = hash.get(0).unwrap();
                                    let hash = Hash::from_slice(&hash[1..]).to_hex();
                                    ghostdag.insert(hash, data);
                                }
                                Err(e) => {}
                            }

                            bar.inc(1);
                        });

                        bar.finish();
                    }

                    _ => {
                        panic!("Unimplemented key: {}", key);
                    }
                }

                println!("JSON export completed successfully.");
            }
        }

        match key.as_str() {
            "acceptance_data" => {
                serde_json::to_writer(&mut writer, &acceptance_data).unwrap();
            }

            "blocks_transactions" => {
                if to_address {
                    serde_json::to_writer(&mut writer, &blocks_with_addresses).unwrap();
                } else {
                    serde_json::to_writer(&mut writer, &blocks).unwrap();
                }
            }

            "headers" => {
                serde_json::to_writer(&mut writer, &headers).unwrap();
            }

            "ghostdag" => {
                serde_json::to_writer(&mut writer, &ghostdag).unwrap();
            }

            _ => {
                panic!("Unimplemented key: {}", key);
            }
        }
    }

}
