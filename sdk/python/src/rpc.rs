use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

use futures::FutureExt;
use futures::select_biased;
use pyo3::prelude::*;
// for `.fuse()`
use pyo3::types::{PyDict, PyFunction, PyTuple};
use workflow_core::channel::Channel;
use workflow_core::prelude::spawn;

use kaspa_addresses::Address;
use kaspa_consensus_core::network::{NetworkId, NetworkType};
use kaspa_consensus_core::tx::TransactionId;
use kaspa_notify::scope::{BlockAddedScope, FinalityConflictResolvedScope, FinalityConflictScope, NewBlockTemplateScope, PruningPointUtxoSetOverrideScope, Scope, SinkBlueScoreChangedScope, UtxosChangedScope, VirtualChainChangedScope, VirtualDaaScoreChangedScope};
use kaspa_rpc_core::{Notification, RpcAddress, RpcBlock, RpcContextualPeerAddress, RpcExtraData, RpcHash, RpcIpAddress, RpcSubnetworkId};
use kaspa_rpc_core::api::ctl::RpcState;
use kaspa_rpc_core::notify::connection::{ChannelConnection, ChannelType};
use kaspa_wallet_core::account::Account;
use kaspa_wallet_core::api::WalletApi;
use kaspa_wallet_core::prelude::KaspaRpcClient;
use kaspa_wallet_core::rpc::WrpcEncoding;

use crate::rpc_core::RpcCore;
use crate::rpc_types::{py_rpc_block_type, py_rpc_transaction_type};

type ListenerCallback = Arc<Mutex<HashMap<String, Py<PyFunction>>>>;

macro_rules! listen_event {
    ($self:ident, $py:ident, $event:expr, $callback:expr, $scope:expr) => {{
        let client = Arc::new($self.client.clone().unwrap());
        let notification_channel_sender = $self.notification_channel.sender.clone();
        let listeners = Arc::clone(&$self.listeners);

        $py.allow_threads(|| {
            let mut listeners = listeners.lock().unwrap();
            listeners.insert($event.to_string(), $callback);
        });

        pyo3_asyncio::tokio::future_into_py($py, async move {
            let listener_id = client.rpc_api().register_new_listener(ChannelConnection::new(
                "",
                notification_channel_sender,
                ChannelType::Persistent,
            ));

            client.rpc_api().start_notify(listener_id, $scope).await.unwrap();
            Ok(())
        })
    }};
}

#[pyclass]
pub struct RPC {
    url: Option<String>,
    client: Option<KaspaRpcClient>,
    is_connected: AtomicBool,
    notification_channel: Channel<Notification>,
    listeners: ListenerCallback,
}

#[pymethods]
impl RPC {
    #[new]
    fn new(url: Option<String>) -> Self {
        RPC {
            url: url,
            client: None,
            is_connected: AtomicBool::new(false),
            notification_channel: Channel::unbounded(),
            listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn connect<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        // let url = self.url.clone().unwrap_or("grpc://127.0.0.1:13110".to_string());
        let url = self.url.clone().unwrap_or("ws://127.0.0.1:17110".to_string());
        self.client = Some(KaspaRpcClient::new(WrpcEncoding::Borsh, Some(url.as_str().as_ref()), None, Some(NetworkId::new(NetworkType::Mainnet)), None).unwrap());
        let client = self.client.clone().unwrap();
        let notification_channel_receiver = self.notification_channel.receiver.clone();
        let listeners = Arc::clone(&self.listeners);

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let error = client.connect(None).await.unwrap();
            if error.is_some() {
                return Ok(false);
            }

            let rpc_ctl_channel = client.rpc_ctl().multiplexer().channel();

            fn emit_event(event: &str, args: impl IntoPy<Py<PyTuple>>, listeners: &ListenerCallback) {
                let listeners = listeners.lock().unwrap();
                if let Some(callback) = listeners.get(event) {
                    Python::with_gil(|py| {
                        callback.call1(py, args).map_err(|e| e.print(py)).ok();
                    })
                }
            }

            spawn(async move {
                loop {
                    select_biased! {
                    msg = rpc_ctl_channel.receiver.recv().fuse() => {

                        println!("msg: {:?}", msg);
                        match msg {
                            Ok(msg) => {
                                match msg {
                                    RpcState::Connected => {
                                        // self.is_connected.store(true, std::sync::atomic::Ordering::Relaxed); // TODO:
                                        // if let Err(err) = listener.handle_connect().await {
                                        //     log_error!("Error in connect handler: {err}");
                                        // }
                                    },
                                    RpcState::Disconnected => {
                                        // self.is_connected.store(false, std::sync::atomic::Ordering::Relaxed); // TODO:
                                        // if let Err(err) = listener.handle_disconnect().await {
                                        //     log_error!("Error in disconnect handler: {err}");
                                        // }
                                    }
                                }
                            }
                            Err(err) => {
                                // this will never occur if the RpcClient is owned and
                                // properly managed. This can only occur if RpcClient is
                                // deleted while this task is still running.
                                println!("RPC CTL channel error: {err}");
                                // panic!("Unexpected: RPC CTL channel closed, halting...");
                            }
                        }
                    }

                    notification = notification_channel_receiver.recv().fuse() => {
                        match notification {
                            Ok(notification) => {
                                println!("notification: {:?}", notification);

                                match notification {
                                    Notification::BlockAdded(ref payload) => {
                                        emit_event("block-added", (RpcBlock {
                                            header: payload.block.header.clone(),
                                            transactions: payload.block.transactions.clone(),
                                            verbose_data: payload.block.verbose_data.clone(),
                                            },), &listeners);
                                    },
                                    Notification::FinalityConflict(ref payload) => {
                                        emit_event("finality-conflict", (payload.violating_block_hash,), &listeners);
                                    },
                                    Notification::FinalityConflictResolved(ref payload) => {
                                        emit_event("finality-conflict-resolved", (payload.finality_block_hash,), &listeners);
                                    },
                                    Notification::NewBlockTemplate(ref payload) => {
                                        emit_event("new-block-template", (), &listeners);
                                    },
                                    Notification::PruningPointUtxoSetOverride(ref payload) => {
                                        emit_event("pruning-point-utxo-set-override", (), &listeners);
                                    },
                                    Notification::UtxosChanged(ref payload) => {
                                        let added: Vec<_> = payload.added.iter().cloned().collect();
                                        let removed: Vec<_> = payload.removed.iter().cloned().collect();

                                        emit_event("utxos-changed", (added, removed,), &listeners);
                                    },
                                    Notification::VirtualDaaScoreChanged(ref payload) => {
                                        emit_event("virtual-daa-score", (payload.virtual_daa_score,), &listeners);
                                    },
                                    Notification::SinkBlueScoreChanged(ref payload) => {
                                        emit_event("sink-blue-score", (payload.sink_blue_score,), &listeners);
                                    },
                                    Notification::VirtualChainChanged(ref payload) => {
                                        let added_chain_block_hashes: Vec<_> = payload.added_chain_block_hashes.iter().cloned().collect();
                                        let removed_chain_block_hashes: Vec<_> = payload.removed_chain_block_hashes.iter().cloned().collect();
                                        let accepted_transaction_ids: Vec<_> = payload.accepted_transaction_ids.iter().cloned().collect();

                                        emit_event("virtual-chain", (added_chain_block_hashes, removed_chain_block_hashes, accepted_transaction_ids,), &listeners);
                                    },
                                    _ => {}
                                }

                                // if let Err(err) = listener.handle_notification(notification).await {
                                //     log_error!("Error while handling notification: {err}");
                                // }
                            }
                            Err(err) => {
                                // panic!("RPC notification channel error: {err}");
                            }
                        }
                    },
                }
                }
            });

            // let client = Arc::new(
            //     KaspaRpcClient::new(, url.as_deref(), resolver.clone().map(Into::into), network_id, None)
            //         .unwrap_or_else(|err| panic!("{err}")),
            // );
            //
            // let binding = wallet.clone().try_wrpc_client();
            // let wrpc_client = binding.as_ref();
            //
            // let options = ConnectOptions {
            //     block_async_connect: true,
            //     strategy: ConnectStrategy::Fallback,
            //     url: Some(url),
            //     ..Default::default()
            // };
            //
            // wrpc_client.unwrap().connect(Some(options)).await.unwrap();
            //
            // wallet.set_network_id(&NetworkId::new(NetworkType::Mainnet)).unwrap();
            //
            // let _ = wallet.start().await; // TODO: Do we always need it with the python wallet api ?

            Ok(true)
        })
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn on_block_added<'a>(&mut self, py: Python<'a>, callback: Py<PyFunction>) -> PyResult<&'a PyAny> {
        listen_event!(self, py, "block-added".to_string(), callback, Scope::BlockAdded(BlockAddedScope {}))
    }

    pub fn on_finality_conflict<'a>(&mut self, py: Python<'a>, callback: Py<PyFunction>) -> PyResult<&'a PyAny> {
        listen_event!(self, py, "finality-conflict".to_string(), callback, Scope::FinalityConflict(FinalityConflictScope {}))
    }

    pub fn on_finality_conflict_resolved<'a>(&mut self, py: Python<'a>, callback: Py<PyFunction>) -> PyResult<&'a PyAny> {
        listen_event!(self, py, "finality-conflict-resolved".to_string(), callback, Scope::FinalityConflictResolved(FinalityConflictResolvedScope {}))
    }

    pub fn on_new_block_template<'a>(&mut self, py: Python<'a>, callback: Py<PyFunction>) -> PyResult<&'a PyAny> {
        listen_event!(self, py, "new-block-template".to_string(), callback, Scope::NewBlockTemplate(NewBlockTemplateScope {}))
    }

    pub fn on_pruning_point_utxo_set_override<'a>(&mut self, py: Python<'a>, callback: Py<PyFunction>) -> PyResult<&'a PyAny> {
        listen_event!(self, py, "pruning-point-utxo-set-override".to_string(), callback, Scope::PruningPointUtxoSetOverride(PruningPointUtxoSetOverrideScope {}))
    }

    pub fn on_utxos_changed<'a>(&mut self, py: Python<'a>, callback: Py<PyFunction>, addresses: Vec<String>) -> PyResult<&'a PyAny> {
        let addresses = addresses.iter().map(|address| Address::try_from(address.as_str()).expect("Failed to parse address")).collect();
        listen_event!(self, py, "utxos-changed".to_string(), callback, Scope::UtxosChanged(UtxosChangedScope {addresses}))
    }

    pub fn on_virtual_daa_score_changed<'a>(&mut self, py: Python<'a>, callback: Py<PyFunction>) -> PyResult<&'a PyAny> {
        listen_event!(self, py, "virtual-daa-score".to_string(), callback, Scope::VirtualDaaScoreChanged(VirtualDaaScoreChangedScope {}))
    }

    pub fn on_sink_blue_score_changed<'a>(&mut self, py: Python<'a>, callback: Py<PyFunction>) -> PyResult<&'a PyAny> {
        listen_event!(self, py, "sink-blue-score".to_string(), callback, Scope::SinkBlueScoreChanged(SinkBlueScoreChangedScope {}))
    }

    pub fn on_virtual_chain_changed<'a>(&mut self, py: Python<'a>, callback: Py<PyFunction>, include_accepted_transaction_ids: bool) -> PyResult<&'a PyAny> {
        listen_event!(self, py, "virtual-chain".to_string(), callback, Scope::VirtualChainChanged(VirtualChainChangedScope {include_accepted_transaction_ids}))
    }

    pub fn ping<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().ping().await.map_err(PyErr::from)
        })
    }

    pub fn get_metrics<'a>(&mut self, py: Python<'a>, process_metrics: bool, connection_metrics: bool, bandwidth_metrics: bool, consensus_metrics: bool) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_metrics(process_metrics, connection_metrics, bandwidth_metrics, consensus_metrics).await.map_err(PyErr::from)
        })
    }

    pub fn get_server_info<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        let py = py.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_server_info().await.map_err(PyErr::from)
        })
    }

    pub fn get_sync_status<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_sync_status().await.map_err(PyErr::from)
        })
    }

    pub fn get_current_network<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            match client.rpc_api().get_current_network().await {
                Ok(network) => Ok(network.to_string()),
                Err(err) => Err(PyErr::from(err)),
            }
        })
    }

    pub fn submit_block<'a>(&mut self, py: Python<'a>, block_dict: &PyDict, allow_non_daa_blocks: bool) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        println!("block_dict: {:?}", block_dict);
        let header = RpcCore::get_dict_item(block_dict, "header").downcast::<PyDict>().unwrap();
        println!("header: {:?}", header);
        println!("keys: {}", header.keys());
        println!("HASH: {}", header.get_item("hash").unwrap().unwrap());
        // let hash: String = RpcCore::get_dict_item(header, "hash").to_string();
        let block = py_rpc_block_type(block_dict)?;

        pyo3_asyncio::tokio::future_into_py(py, async move {

            // println!("hash: {}", hash);

            let result = client.rpc_api().submit_block(block, allow_non_daa_blocks).await.unwrap().report;

            let result = match result {
                kaspa_rpc_core::SubmitBlockReport::Success => 0,
                kaspa_rpc_core::SubmitBlockReport::Reject(kaspa_rpc_core::SubmitBlockRejectReason::BlockInvalid) => 1,
                kaspa_rpc_core::SubmitBlockReport::Reject(kaspa_rpc_core::SubmitBlockRejectReason::IsInIBD) => 2,
                kaspa_rpc_core::SubmitBlockReport::Reject(kaspa_rpc_core::SubmitBlockRejectReason::RouteIsFull) => 0,
            };

            Ok(result)
            // Ok(result.is_success())
            // Ok(true)
            // Ok(hash)
        })
    }

    pub fn get_block_template<'a>(&mut self, py: Python<'a>, address: String, extra_data: RpcExtraData) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api()
                .get_block_template(RpcAddress::try_from(address).expect("Failed to parse address"), extra_data)
                .await.map_err(PyErr::from)
        })
    }

    pub fn get_peer_addresses<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_peer_addresses().await.map_err(PyErr::from)
        })
    }

    pub fn get_sink<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_sink().await.map_err(PyErr::from)
        })
    }

    pub fn get_mempool_entry<'a>(&mut self, py: Python<'a>, transaction_id: String,
                                 include_orphan_pool: bool, filter_transaction_pool: bool) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_mempool_entry(
                TransactionId::from_str(transaction_id.as_str()).expect("Failed to parse transaction id"),
                include_orphan_pool, filter_transaction_pool,
            ).await.map_err(PyErr::from)
        })
    }

    pub fn get_mempool_entries<'a>(&mut self, py: Python<'a>, include_orphan_pool: bool, filter_transaction_pool: bool) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_mempool_entries(include_orphan_pool, filter_transaction_pool).await.map_err(PyErr::from)
        })
    }

    pub fn get_connected_peer_info<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_connected_peer_info().await.map_err(PyErr::from)
        })
    }

    pub fn add_peer<'a>(&mut self, py: Python<'a>, peer_address: String, is_permanent: bool) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().add_peer(RpcContextualPeerAddress::from_str(peer_address.as_str()).map_err(PyErr::from)?, is_permanent)
                .await.map_err(PyErr::from)
        })
    }

    pub fn submit_transaction<'a>(&mut self, py: Python<'a>, transaction: &PyDict, allow_orphan: bool) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());
        let transaction = py_rpc_transaction_type(transaction)?;

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().submit_transaction(transaction, allow_orphan).await.map_err(PyErr::from)
        })
    }

    pub fn get_block<'a>(&mut self, py: Python<'a>, hash: String, include_transactions: bool) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_block(RpcHash::from_str(hash.as_str()).expect("Failed to parse hash"), include_transactions).await.map_err(PyErr::from)
        })
    }

    pub fn get_subnetwork<'a>(&mut self, py: Python<'a>, subnetwork_id: String) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_subnetwork(RpcSubnetworkId::from_str(subnetwork_id.as_str()).expect("Failed to parse subnetwork id")).await.map_err(PyErr::from)
        })
    }

    pub fn get_virtual_chain_from_block<'a>(&mut self, py: Python<'a>, start_hash: String, include_accepted_transaction_ids: bool) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_virtual_chain_from_block(RpcHash::from_str(start_hash.as_str()).expect("Failed to parse hash"), include_accepted_transaction_ids).await.map_err(PyErr::from)
        })
    }

    pub fn get_blocks<'a>(&mut self, py: Python<'a>, low_hash: Option<String>, include_blocks: Option<bool>, include_transactions: Option<bool>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());
        let low_hash = match low_hash.is_some() {
            true => Some(RpcHash::from_str(low_hash.unwrap().as_str()).expect("Failed to parse hash")),
            false => None,
        };

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_blocks(low_hash, include_blocks.unwrap_or(false), include_transactions.unwrap_or(false)).await.map_err(PyErr::from)
        })
    }

    pub fn get_block_count<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_block_count().await.map_err(PyErr::from)
        })
    }

    pub fn get_block_dag_info<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_block_dag_info().await.map_err(PyErr::from)
        })
    }

    pub fn resolve_finality_conflict<'a>(&mut self, py: Python<'a>, finality_block_hash: String) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());
        let finality_block_hash = RpcHash::from_str(finality_block_hash.as_str()).expect("Failed to parse hash");

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().resolve_finality_conflict(finality_block_hash).await.map_err(PyErr::from)
        })
    }

    pub fn shutdown<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().shutdown().await.map_err(PyErr::from)
        })
    }

    pub fn get_headers<'a>(&mut self, py: Python<'a>, start_hash: String, limit: u64, is_ascending: bool) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());
        let start_hash = RpcHash::from_str(start_hash.as_str()).expect("Failed to parse hash");

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_headers(start_hash, limit, is_ascending).await.map_err(PyErr::from)
        })
    }

    pub fn get_balance_by_address<'a>(&mut self, py: Python<'a>, address: String) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_balance_by_address(Address::try_from(address).expect("Failed to parse address")).await.map_err(PyErr::from)
        })
    }

    pub fn get_balances_by_addresses<'a>(&mut self, py: Python<'a>, addresses: Vec<String>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());
        let addresses = addresses.iter().map(|address| Address::try_from(address.to_string()).expect("Failed to parse address")).collect();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_balances_by_addresses(addresses).await.map_err(PyErr::from)
        })
    }

    pub fn get_utxos_by_addresses<'a>(&mut self, py: Python<'a>, addresses: Vec<String>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());
        let addresses = addresses.iter().map(|address| Address::try_from(address.to_string()).expect("Failed to parse address")).collect();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_utxos_by_addresses(addresses).await.map_err(PyErr::from)
        })
    }

    pub fn get_sink_blue_score<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_sink_blue_score().await.map_err(PyErr::from)
        })
    }

    pub fn ban<'a>(&mut self, py: Python<'a>, ip: String) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());
        let ip = RpcIpAddress::from_str(ip.as_str()).expect("Failed to parse ip");

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().ban(ip).await.map_err(PyErr::from)
        })
    }

    pub fn unban<'a>(&mut self, py: Python<'a>, ip: String) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());
        let ip = RpcIpAddress::from_str(ip.as_str()).expect("Failed to parse ip");

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().unban(ip).await.map_err(PyErr::from)
        })
    }

    pub fn get_info<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_info().await.map_err(PyErr::from)
        })
    }

    pub fn estimate_network_hashes_per_second<'a>(&mut self, py: Python<'a>, window_size: u32, start_hash: Option<String>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());
        let start_hash = match start_hash.is_some() {
            true => Some(RpcHash::from_str(start_hash.unwrap().as_str()).expect("Failed to parse hash")),
            false => None,
        };

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().estimate_network_hashes_per_second(window_size, start_hash).await.map_err(PyErr::from)
        })
    }

    pub fn get_mempool_entries_by_addresses<'a>(&mut self, py: Python<'a>, addresses: Vec<String>, include_orphan_pool: bool, filter_transaction_pool: bool) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());
        let addresses = addresses.iter().map(|address| RpcAddress::try_from(address.as_str()).expect("Failed to parse address")).collect();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_mempool_entries_by_addresses(addresses, include_orphan_pool, filter_transaction_pool).await.map_err(PyErr::from)
        })
    }

    pub fn get_coin_supply<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_coin_supply().await.map_err(PyErr::from)
        })
    }

    pub fn get_daa_score_timestamp_estimate<'a>(&mut self, py: Python<'a>, daa_scores: Vec<u64>) -> PyResult<&'a PyAny> {
        let client = Arc::new(self.client.clone().unwrap());

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.rpc_api().get_daa_score_timestamp_estimate(daa_scores).await.map_err(PyErr::from)
        })
    }
}