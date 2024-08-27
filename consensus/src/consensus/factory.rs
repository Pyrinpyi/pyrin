use std::{collections::HashMap, error::Error, fs, path::PathBuf, sync::Arc};
use std::io::{Cursor, Read};

use itertools::Itertools;
use parking_lot::RwLock;
use rocksdb::WriteBatch;
use serde::{Deserialize, Serialize};

use kaspa_consensus_core::config::Config;
use kaspa_consensus_core::tx::{ScriptPublicKey, TransactionOutpoint, UtxoEntry};
use kaspa_consensus_core::utxo::utxo_collection::UtxoCollection;
use kaspa_consensus_notify::root::ConsensusNotificationRoot;
use kaspa_consensusmanager::{ConsensusFactory, ConsensusInstance, DynConsensusCtl, SessionLock};
use kaspa_core::{debug, error, info, time::unix_now, warn};
use kaspa_database::{
    prelude::{
        BatchDbWriter, CachedDbAccess, CachedDbItem, CachePolicy, DB, DirectDbWriter, StoreError, StoreResult, StoreResultExtensions,
    },
    registry::DatabaseStorePrefixes,
};
use kaspa_txscript::caches::TxScriptCacheCounters;
use kaspa_utils::mem_size::MemSizeEstimator;
use zip::ZipArchive;

use crate::{model::stores::U64Key, pipeline::ProcessingCounters};

use super::{Consensus, ctl::Ctl};
#[cfg(feature = "devnet-prealloc")]
use super::utxo_set_override::{set_genesis_utxo_commitment_from_config};

use super::utxo_set_override::{set_initial_utxo_set};

#[derive(Serialize, Deserialize, Clone)]
pub struct ConsensusEntry {
    key: u64,
    directory_name: String,
    creation_timestamp: u64,
}

impl MemSizeEstimator for ConsensusEntry {}

impl ConsensusEntry {
    pub fn new(key: u64, directory_name: String, creation_timestamp: u64) -> Self {
        Self { key, directory_name, creation_timestamp }
    }

    pub fn from_key(key: u64) -> Self {
        Self { key, directory_name: format!("consensus-{:0>3}", key), creation_timestamp: unix_now() }
    }
}

pub enum ConsensusEntryType {
    Existing(ConsensusEntry),
    New(ConsensusEntry),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MultiConsensusMetadata {
    current_consensus_key: Option<u64>,
    staging_consensus_key: Option<u64>,
    /// Max key used for a consensus entry
    max_key_used: u64,
    /// Memorizes whether this node was recently an archive node
    is_archival_node: bool,
    /// General serialized properties to be used cross DB versions
    props: HashMap<Vec<u8>, Vec<u8>>,
    /// The DB scheme version
    version: u32,
}

const LATEST_DB_VERSION: u32 = 3;
impl Default for MultiConsensusMetadata {
    fn default() -> Self {
        Self {
            current_consensus_key: Default::default(),
            staging_consensus_key: Default::default(),
            max_key_used: Default::default(),
            is_archival_node: Default::default(),
            props: Default::default(),
            version: LATEST_DB_VERSION,
        }
    }
}

#[derive(Clone)]
pub struct MultiConsensusManagementStore {
    db: Arc<DB>,
    entries: CachedDbAccess<U64Key, ConsensusEntry>,
    metadata: CachedDbItem<MultiConsensusMetadata>,
}

impl MultiConsensusManagementStore {
    pub fn new(db: Arc<DB>) -> Self {
        let mut store = Self {
            db: db.clone(),
            entries: CachedDbAccess::new(db.clone(), CachePolicy::Count(16), DatabaseStorePrefixes::ConsensusEntries.into()),
            metadata: CachedDbItem::new(db, DatabaseStorePrefixes::MultiConsensusMetadata.into()),
        };
        store.init();
        store
    }

    fn init(&mut self) {
        if self.metadata.read().unwrap_option().is_none() {
            let mut batch = WriteBatch::default();
            let metadata = MultiConsensusMetadata::default();
            self.metadata.write(BatchDbWriter::new(&mut batch), &metadata).unwrap();
            self.db.write(batch).unwrap();
        }
    }

    /// The directory name of the active consensus, if one exists. None otherwise
    pub fn active_consensus_dir_name(&self) -> StoreResult<Option<String>> {
        let metadata = self.metadata.read()?;
        match metadata.current_consensus_key {
            Some(key) => Ok(Some(self.entries.read(key.into()).unwrap().directory_name)),
            None => Ok(None),
        }
    }

    /// The entry type signifies whether the returned entry is an existing/new consensus
    pub fn active_consensus_entry(&mut self) -> StoreResult<ConsensusEntryType> {
        let mut metadata = self.metadata.read()?;
        match metadata.current_consensus_key {
            Some(key) => Ok(ConsensusEntryType::Existing(self.entries.read(key.into())?)),
            None => {
                metadata.max_key_used += 1; // Capture the slot
                let key = metadata.max_key_used;
                self.metadata.write(DirectDbWriter::new(&self.db), &metadata)?;
                Ok(ConsensusEntryType::New(ConsensusEntry::from_key(key)))
            }
        }
    }

    // This function assumes metadata is already set
    pub fn staging_consensus_entry(&mut self) -> Option<ConsensusEntry> {
        let metadata = self.metadata.read().unwrap();
        match metadata.staging_consensus_key {
            Some(key) => Some(self.entries.read(key.into()).unwrap()),
            None => None,
        }
    }

    pub fn save_new_active_consensus(&mut self, entry: ConsensusEntry) -> StoreResult<()> {
        let key = entry.key;
        if self.entries.has(key.into())? {
            return Err(StoreError::KeyAlreadyExists(format!("{key}")));
        }
        let mut batch = WriteBatch::default();
        self.entries.write(BatchDbWriter::new(&mut batch), key.into(), entry)?;
        self.metadata.update(BatchDbWriter::new(&mut batch), |mut data| {
            data.current_consensus_key = Some(key);
            data
        })?;
        self.db.write(batch)?;
        Ok(())
    }

    pub fn new_staging_consensus_entry(&mut self) -> StoreResult<ConsensusEntry> {
        let mut metadata = self.metadata.read()?;

        metadata.max_key_used += 1;
        let new_key = metadata.max_key_used;
        metadata.staging_consensus_key = Some(new_key);
        let new_entry = ConsensusEntry::from_key(new_key);

        let mut batch = WriteBatch::default();
        self.metadata.write(BatchDbWriter::new(&mut batch), &metadata)?;
        self.entries.write(BatchDbWriter::new(&mut batch), new_key.into(), new_entry.clone())?;
        self.db.write(batch)?;

        Ok(new_entry)
    }

    pub fn commit_staging_consensus(&mut self) -> StoreResult<()> {
        self.metadata.update(DirectDbWriter::new(&self.db), |mut data| {
            assert!(data.staging_consensus_key.is_some());
            data.current_consensus_key = data.staging_consensus_key.take();
            data
        })?;
        Ok(())
    }

    pub fn cancel_staging_consensus(&mut self) -> StoreResult<()> {
        self.metadata.update(DirectDbWriter::new(&self.db), |mut data| {
            data.staging_consensus_key = None;
            data
        })?;
        Ok(())
    }

    fn iterator(&self) -> impl Iterator<Item = Result<ConsensusEntry, Box<dyn Error>>> + '_ {
        self.entries.iterator().map(|iter_result| match iter_result {
            Ok((_, entry)) => Ok(entry),
            Err(e) => Err(e),
        })
    }

    fn iterate_inactive_entries(&self) -> impl Iterator<Item = Result<ConsensusEntry, Box<dyn Error>>> + '_ {
        let current_consensus_key = self.metadata.read().unwrap().current_consensus_key;
        self.iterator().filter(move |entry_result| {
            if let Ok(entry) = entry_result {
                return Some(entry.key) != current_consensus_key;
            }

            true
        })
    }

    fn delete_entry(&mut self, entry: ConsensusEntry) -> StoreResult<()> {
        self.entries.delete(DirectDbWriter::new(&self.db), entry.key.into())
    }

    pub fn is_archival_node(&self) -> StoreResult<bool> {
        match self.metadata.read() {
            Ok(data) => Ok(data.is_archival_node),
            Err(StoreError::KeyNotFound(_)) => Ok(false),
            Err(err) => Err(err),
        }
    }

    pub fn set_is_archival_node(&mut self, is_archival_node: bool) {
        let mut metadata = self.metadata.read().unwrap();
        if metadata.is_archival_node != is_archival_node {
            metadata.is_archival_node = is_archival_node;
            let mut batch = WriteBatch::default();
            self.metadata.write(BatchDbWriter::new(&mut batch), &metadata).unwrap();
        }
    }

    pub fn should_upgrade(&self) -> StoreResult<bool> {
        match self.metadata.read() {
            Ok(data) => Ok(data.version != LATEST_DB_VERSION),
            Err(StoreError::KeyNotFound(_)) => Ok(false),
            Err(err) => Err(err),
        }
    }
}

pub struct Factory {
    management_store: Arc<RwLock<MultiConsensusManagementStore>>,
    config: Config,
    db_root_dir: PathBuf,
    db_parallelism: usize,
    notification_root: Arc<ConsensusNotificationRoot>,
    counters: Arc<ProcessingCounters>,
    tx_script_cache_counters: Arc<TxScriptCacheCounters>,
    fd_budget: i32,
}

impl Factory {
    pub fn new(
        management_db: Arc<DB>,
        config: &Config,
        db_root_dir: PathBuf,
        db_parallelism: usize,
        notification_root: Arc<ConsensusNotificationRoot>,
        counters: Arc<ProcessingCounters>,
        tx_script_cache_counters: Arc<TxScriptCacheCounters>,
        fd_budget: i32,
    ) -> Self {
        assert!(fd_budget > 0, "fd_budget has to be positive");
        let mut config = config.clone();
        #[cfg(feature = "devnet-prealloc")]
        set_genesis_utxo_commitment_from_config(&mut config);
        config.process_genesis = false;
        let management_store = Arc::new(RwLock::new(MultiConsensusManagementStore::new(management_db)));
        management_store.write().set_is_archival_node(config.is_archival);
        let factory = Self {
            management_store,
            config,
            db_root_dir,
            db_parallelism,
            notification_root,
            counters,
            tx_script_cache_counters,
            fd_budget,
        };
        factory.delete_inactive_consensus_entries();
        factory
    }
}

impl ConsensusFactory for Factory {
    fn new_active_consensus(&self) -> (ConsensusInstance, DynConsensusCtl) {
        assert!(!self.notification_root.is_closed());

        let mut config = self.config.clone();
        let mut is_new_consensus = false;
        let entry = match self.management_store.write().active_consensus_entry().unwrap() {
            ConsensusEntryType::Existing(entry) => {
                config.process_genesis = false;
                entry
            }
            ConsensusEntryType::New(entry) => {
                // Configure to process genesis only if this is a brand new consensus
                config.process_genesis = true;
                is_new_consensus = true;
                entry
            }
        };

        let dir = self.db_root_dir.join(entry.directory_name.clone());
        let db = kaspa_database::prelude::ConnBuilder::default()
            .with_db_path(dir)
            .with_parallelism(self.db_parallelism)
            .with_files_limit(self.fd_budget / 2) // active and staging consensuses should have equal budgets
            .build()
            .unwrap();

        let session_lock = SessionLock::new();
        let consensus = Arc::new(Consensus::new(
            db.clone(),
            Arc::new(config),
            session_lock.clone(),
            self.notification_root.clone(),
            self.counters.clone(),
            self.tx_script_cache_counters.clone(),
            entry.creation_timestamp,
        ));

        // We write the new active entry only once the instance was created successfully.
        // This way we can safely avoid processing genesis in future process runs
        if is_new_consensus {
            #[cfg(feature = "devnet-prealloc")]
            set_initial_utxo_set(&self.config.initial_utxo_set, consensus.clone(), self.config.params.genesis.hash);

            // HF Relaunch: Load the UTXO dump with a commitment of e9b3ab4ccc51b1925de45f1af019b4dae00e34331e7db8dc2c35b0fd48e75438 (13,969,182 UTXOs)
            let utxo_set: UtxoCollection = load_utxo_dump();
            set_initial_utxo_set(&utxo_set, consensus.clone(), self.config.params.genesis.hash);

            self.management_store.write().save_new_active_consensus(entry).unwrap();
        }

        (ConsensusInstance::new(session_lock, consensus.clone()), Arc::new(Ctl::new(self.management_store.clone(), db, consensus)))
    }

    fn new_staging_consensus(&self) -> (ConsensusInstance, DynConsensusCtl) {
        assert!(!self.notification_root.is_closed());

        let entry = self.management_store.write().new_staging_consensus_entry().unwrap();
        let dir = self.db_root_dir.join(entry.directory_name);
        let db = kaspa_database::prelude::ConnBuilder::default()
            .with_db_path(dir)
            .with_parallelism(self.db_parallelism)
            .with_files_limit(self.fd_budget / 2) // active and staging consensuses should have equal budgets
            .build()
            .unwrap();

        let session_lock = SessionLock::new();
        let consensus = Arc::new(Consensus::new(
            db.clone(),
            Arc::new(self.config.to_builder().skip_adding_genesis().build()),
            session_lock.clone(),
            self.notification_root.clone(),
            self.counters.clone(),
            self.tx_script_cache_counters.clone(),
            entry.creation_timestamp,
        ));

        (ConsensusInstance::new(session_lock, consensus.clone()), Arc::new(Ctl::new(self.management_store.clone(), db, consensus)))
    }

    fn close(&self) {
        debug!("Consensus factory: closing");
        self.notification_root.close();
    }

    fn delete_inactive_consensus_entries(&self) {
        // Staging entry is deleted also by archival nodes since it represents non-final data
        self.delete_staging_entry();

        if self.config.is_archival {
            return;
        }

        let mut write_guard = self.management_store.write();
        let entries_to_delete = write_guard
            .iterate_inactive_entries()
            .filter_map(|entry_result| {
                let entry = entry_result.unwrap();
                let dir = self.db_root_dir.join(entry.directory_name.clone());
                if dir.exists() {
                    match fs::remove_dir_all(dir) {
                        Ok(_) => Some(entry),
                        Err(e) => {
                            warn!("Error deleting consensus entry {}: {}", entry.key, e);
                            None
                        }
                    }
                } else {
                    Some(entry)
                }
            })
            .collect_vec();

        for entry in entries_to_delete {
            write_guard.delete_entry(entry).unwrap();
        }
    }

    fn delete_staging_entry(&self) {
        let mut write_guard = self.management_store.write();
        if let Some(entry) = write_guard.staging_consensus_entry() {
            let dir = self.db_root_dir.join(entry.directory_name.clone());
            match fs::remove_dir_all(dir) {
                Ok(_) => {
                    write_guard.delete_entry(entry).unwrap();
                }
                Err(e) => {
                    warn!("Error deleting staging consensus entry {}: {}", entry.key, e);
                }
            };
            write_guard.cancel_staging_consensus().unwrap();
        }
    }
}


static UTXO_DUMP_1: &[u8] = include_bytes!("utxo_data.zip.001");
static UTXO_DUMP_2: &[u8] = include_bytes!("utxo_data.zip.002");
static UTXO_DUMP_3: &[u8] = include_bytes!("utxo_data.zip.003");
static UTXO_DUMP_4: &[u8] = include_bytes!("utxo_data.zip.004");
static UTXO_DUMP_5: &[u8] = include_bytes!("utxo_data.zip.005");
static UTXO_DUMP_6: &[u8] = include_bytes!("utxo_data.zip.006");


fn load_utxo_dump() -> UtxoCollection {
    info!("Recovering the UTXO set of the pruning point");

    #[derive(Serialize, Deserialize, Debug)]
    struct UtxoData {
        version: u16,
        script_len: u64,
        script: Vec<u8>,
        transaction_id: [u8; 32],
        index: u32,
        amount: u64,
        block_daa_score: u64,
        is_coinbase: bool,
    }

    fn decode_utxo_data_list(binary_data: &[u8]) -> Result<Vec<UtxoData>, Box<dyn std::error::Error>> {
        let mut utxo_data_list = Vec::new();
        let mut cursor = Cursor::new(binary_data);

        while cursor.position() < binary_data.len() as u64 {
            match bincode::deserialize_from::<_, UtxoData>(&mut cursor) {
                Ok(utxo_data) => utxo_data_list.push(utxo_data),
                Err(e) if e.to_string().contains("Unexpected end of file") => break,
                Err(e) => return Err(Box::new(e)),
            }
        }

        Ok(utxo_data_list)
    }


    fn read_binary_from_zip_chunks() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let chunks = [
            UTXO_DUMP_1, UTXO_DUMP_2, UTXO_DUMP_3,
            UTXO_DUMP_4, UTXO_DUMP_5, UTXO_DUMP_6
        ];

        let total_size: usize = chunks.iter().map(|chunk| chunk.len()).sum();
        let mut combined_data = Vec::with_capacity(total_size);

        for chunk in chunks.iter() {
            combined_data.extend_from_slice(chunk);
        }

        let mut archive = ZipArchive::new(Cursor::new(combined_data))?;
        let file_name = "utxo_data.bin";
        let mut zip_file = archive.by_name(file_name)?;
        let mut binary_data = Vec::new();
        zip_file.read_to_end(&mut binary_data)?;

        Ok(binary_data)
    }

    match read_binary_from_zip_chunks() {
        Ok(binary_data) => {
            let utxo_data_list = decode_utxo_data_list(&binary_data).unwrap();

            let mut utxos_loaded: usize = 0;
            let utxos: UtxoCollection = utxo_data_list.iter().map(|d| {
                if utxos_loaded % 400_000 == 0 || utxos_loaded >= (utxo_data_list.len() - 1) {
                    info!("Processing UTXO dump ({:.2}%)", (utxos_loaded as f64 / utxo_data_list.len() as f64) * 100.0);
                }
                utxos_loaded += 1;
                (
                    TransactionOutpoint { transaction_id: d.transaction_id.into(), index: d.index },
                    UtxoEntry { amount: d.amount, script_public_key: ScriptPublicKey::from_vec(
                        d.version,
                        d.script.clone()
                    ), block_daa_score: d.block_daa_score, is_coinbase: d.is_coinbase },
                )
            })
                .collect();

            info!("{} UTXOs loaded", utxos.clone().len());

            utxos
        },
        Err(e) => {
            error!("Error reading from zip file: {}", e);
            UtxoCollection::new()
        },
    }
}