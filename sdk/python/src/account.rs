use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures::{FutureExt};
use futures::channel::oneshot;
use pyo3::prelude::*;
use pyo3::types::{PyFunction, PyTuple};
use tokio::select;
use workflow_core::abortable::Abortable;
use workflow_core::channel::MultiplexerChannel;

use kaspa_addresses::Address;
use kaspa_wallet_core::account::Account;
use kaspa_wallet_core::events::Events;
use kaspa_wallet_core::prelude::{PaymentDestination, PaymentOutputs, Secret};
use kaspa_wallet_core::utils::kaspa_to_sompi;

type ListenerCallback = Arc<Mutex<HashMap<String, Py<PyFunction>>>>;

#[pyclass]
pub struct PyBalance {
    #[pyo3(get)]
    pub mature: u64,
    #[pyo3(get)]
    pub pending: u64,
    #[pyo3(get)]
    pub outgoing: u64,
    #[pyo3(get)]
    pub mature_utxo_count: usize,
    #[pyo3(get)]
    pub pending_utxo_count: usize,
    #[pyo3(get)]
    pub stasis_utxo_count: usize,
}

impl PyBalance {
    pub fn new(
        mature: u64,
        pending: u64,
        outgoing: u64,
        mature_utxo_count: usize,
        pending_utxo_count: usize,
        stasis_utxo_count: usize,
    ) -> Self {
        PyBalance {
            mature,
            pending,
            outgoing,
            mature_utxo_count,
            pending_utxo_count,
            stasis_utxo_count,
        }
    }
}

#[pyclass]
pub struct PySendSummary {
    #[pyo3(get)]
    pub aggregated_utxos: usize,
    #[pyo3(get)]
    pub aggregated_fees: u64,
    #[pyo3(get)]
    pub number_of_generated_transactions: usize,
    #[pyo3(get)]
    pub final_transaction_amount: u64,
    #[pyo3(get)]
    pub final_transaction_id: String,
}

impl PySendSummary {
    pub fn new(
        aggregated_utxos: usize,
        aggregated_fees: u64,
        number_of_generated_transactions: usize,
        final_transaction_amount: u64,
        final_transaction_id: String,
    ) -> Self {
        PySendSummary {
            aggregated_utxos,
            aggregated_fees,
            number_of_generated_transactions,
            final_transaction_amount,
            final_transaction_id,
        }
    }
}

#[pyclass]
pub struct PyAccount {
    account: Option<Arc<dyn Account + Send + Sync>>,
    listeners: ListenerCallback,
    stop_listener: Option<oneshot::Sender<()>>,
}

#[pymethods]
impl PyAccount {
    #[new]
    fn py_new() -> Self {
        PyAccount {
            account: None,
            listeners: Arc::new(Mutex::new(HashMap::new())),
            stop_listener: None,
        }
    }

    pub fn init<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny> {
        if let Some(account) = &self.account {
            let account = account.clone();

            pyo3_asyncio::tokio::future_into_py(py, async move {
                account.scan(None, None).await.unwrap();

                Ok(())
            })
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>("Account not initialized"))
        }
    }

    #[getter]
    fn receive_address(&self) -> PyResult<String> {
        match &self.account {
            Some(account) => Ok(account.receive_address().unwrap().to_string()),
            None => Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>("Account not initialized")),
        }
    }

    pub fn balance(&self) -> PyResult<PyBalance> {
        match &self.account {
            Some(account) => {
                let balance = account.balance();

                match balance {
                    Some(_balance) => {
                        let balance = account.balance().unwrap_or_default();
                        Ok(PyBalance::new(
                            balance.mature,
                            balance.pending,
                            balance.outgoing,
                            balance.mature_utxo_count,
                            balance.pending_utxo_count,
                            balance.stasis_utxo_count,
                        ))
                    }
                    None => Err(PyErr::new::<pyo3::exceptions::PyException, _>("account.init() not called")),
                }
            }
            None => Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>("Account not initialized")),
        }
    }

    pub fn send<'a>(&self, py: Python<'a>, address: String, priority_fee_pyi: f64, amount_pyi: f64) -> PyResult<&'a PyAny> {
        match &self.account {
            Some(account) => {
                let account = account.clone();
                let priority_fee_leor = kaspa_to_sompi(priority_fee_pyi);
                let amount_leor = kaspa_to_sompi(amount_pyi);

                pyo3_asyncio::tokio::future_into_py(py, async move {
                    let abortable = Abortable::default();

                    let summary = account.send(
                        PaymentDestination::PaymentOutputs(PaymentOutputs::from((Address::try_from(address).unwrap(), amount_leor))),
                        priority_fee_leor.into(),
                        None,
                        Secret::new(vec![]),
                        None,
                        &abortable,
                        None,
                    ).await.unwrap();

                    Ok(PySendSummary::new(
                        summary.0.aggregated_utxos(),
                        summary.0.aggregated_fees(),
                        summary.0.number_of_generated_transactions(),
                        summary.0.final_transaction_amount().unwrap(),
                        summary.0.final_transaction_id().unwrap().to_string(),
                    ))
                })
            }
            None => Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>("Account not initialized")),
        }
    }

    pub fn estimate<'a>(&self, py: Python<'a>, priority_fee_pyi: f64, amount_pyi: f64) -> PyResult<&'a PyAny> {
        match &self.account {
            Some(account) => {
                let account = account.clone();
                let priority_fee_leor = kaspa_to_sompi(priority_fee_pyi);
                let amount_leor = kaspa_to_sompi(amount_pyi);

                pyo3_asyncio::tokio::future_into_py(py, async move {
                    let abortable = Abortable::default();

                    // just use any address for an estimate (change address)
                    let change_address = account.clone().change_address().unwrap();
                    let destination = PaymentDestination::PaymentOutputs(PaymentOutputs::from((change_address.clone(), amount_leor)));
                    // let estimate = account.clone().estimate(destination, priority_fee_leor.into(), None, &abortable).await.unwrap();
                    let estimate = account.clone().estimate(destination, priority_fee_leor.into(), None, &abortable).await.unwrap();

                    Ok(estimate.aggregated_fees())
                })
            }
            None => Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>("Account not initialized")),
        }
    }

    pub fn change_address(&self) -> PyResult<String> {
        match &self.account {
            Some(account) => {
                let account = account.clone();
                Ok(account.change_address().unwrap().to_string())
            }
            None => Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>("Account not initialized")),
        }
    }

    pub fn listen<'a>(&mut self, py: Python<'a>, event: String, callback: Py<PyFunction>) -> PyResult<&'a PyAny> {
        match &self.account {
            Some(account) => {
                let account = account.clone();
                let multiplexer = MultiplexerChannel::from(account.wallet().multiplexer());

                let listeners = Arc::clone(&self.listeners);
                let event_clone = event.clone();

                py.allow_threads(|| {
                    let mut listeners = listeners.lock().unwrap();
                    listeners.insert(event_clone, callback);
                });

                match &self.stop_listener {
                    Some(_stop_listener) => {
                        pyo3_asyncio::tokio::future_into_py(py, async move {
                            Ok(())
                        })
                    }
                    None => {
                        let (stop_sender, _stop_receiver) = oneshot::channel();
                        self.stop_listener = Some(stop_sender);

                        fn emit_event(event: &str, args: impl IntoPy<Py<PyTuple>>, listeners: &ListenerCallback) {
                            let listeners = listeners.lock().unwrap();
                            if let Some(callback) = listeners.get(event) {
                                Python::with_gil(|py| {
                                    callback.call1(py, args).map_err(|e| e.print(py)).ok();
                                })
                            }
                        }

                        pyo3_asyncio::tokio::future_into_py(py, async move {
                            // let stop_listener = self.stop_listener.clone().unwrap();

                            tokio::spawn(async move {
                                loop {
                                    select! {
                                        // _ = stop_receiver => {
                                        //     println!("Stopping listeners");
                                        //     break;
                                        // }

                                        msg = multiplexer.receiver.recv().fuse() => {
                                            if let Ok(msg) = msg {
                                            println!("Received message: {:?}", msg);

                                            // if stop_listener.is_none() {
                                            //     println!("NONE");
                                            // }

                                            match *msg {
                                                Events::WalletPing { .. } => {},
                                                Events::Connect { .. } => {},
                                                Events::Disconnect { url, network_id: _ } => {
                                                    emit_event("disconnect", (url.unwrap_or("N/A".to_string()),), &listeners);
                                                    // TODO: Add reconnection logic ?
                                                },
                                                Events::UtxoIndexNotEnabled { .. } => {},
                                                Events::SyncState { sync_state } => {
                                                     emit_event("sync-state", (sync_state.is_synced(),), &listeners);
                                                },
                                                Events::WalletHint { .. } => {},
                                                Events::WalletOpen { .. } => {},
                                                Events::WalletCreate { .. } => {},
                                                Events::WalletReload { .. } => {},
                                                Events::WalletError { .. } => {},
                                                Events::WalletClose => {},
                                                Events::PrvKeyDataCreate { .. } => {},
                                                Events::AccountActivation { .. } => {},
                                                Events::AccountDeactivation { .. } => {},
                                                Events::AccountSelection { .. } => {},
                                                Events::AccountCreate { .. } => {},
                                                Events::AccountUpdate { .. } => {},
                                                Events::ServerStatus { .. } => {},
                                                Events::UtxoProcStart => {},
                                                Events::UtxoProcStop => {},
                                                Events::UtxoProcError { .. } => {},
                                                Events::DaaScoreChange { current_daa_score } => {
                                                    emit_event("dda-score", (current_daa_score,), &listeners);
                                                },
                                                Events::Pending { .. } => {}, // Handled by Balance TODO: We can give TransactionRecord data
                                                Events::Reorg { .. } => {},
                                                Events::Stasis { .. } => {},
                                                Events::Maturity { .. } => {}, // Handled by Balance TODO: We can give TransactionRecord data
                                                Events::Discovery { .. } => {},
                                                Events::Balance { balance, id: _ } => {
                                                    let balance = balance.unwrap_or_default();
                                                    let balance = PyBalance::new(
                                                        balance.mature,
                                                        balance.pending,
                                                        balance.outgoing,
                                                        balance.mature_utxo_count,
                                                        balance.pending_utxo_count,
                                                        balance.stasis_utxo_count,
                                                    );
                                                    emit_event("balance", (balance,), &listeners);
                                                },
                                                Events::Metrics { .. } => {},
                                                Events::Error { .. } => {},
                                            }
                                            }
                                        }
                                    }
                                }
                            });

                            Ok(())
                        })
                    },
                }
            }
            None => Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>("Account not initialized")),
        }
    }

    // pub fn stop_listeners(&mut self) -> PyResult<()> {
    //     self.stop_listener = None;
    //     Ok(())
    // }
}

impl PyAccount {
    pub fn new(account: Arc<dyn Account + Send + Sync>) -> Self {
        PyAccount {
            account: Some(account),
            listeners: Arc::new(Mutex::new(HashMap::new())),
            stop_listener: None,
        }
    }
}