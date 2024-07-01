use std::sync::Arc;

use pyo3::prelude::*;

use kaspa_bip32::{Language, Mnemonic, WordCount};
use kaspa_consensus_core::network::{NetworkId, NetworkType};
use kaspa_wallet_core::account::Account;
use kaspa_wallet_core::api::WalletApi;
use kaspa_wallet_core::encryption::EncryptionKind;
use kaspa_wallet_core::prelude::{AccountCreateArgsBip32, ConnectOptions, ConnectStrategy, PrvKeyDataCreateArgs, Secret};
use kaspa_wallet_core::wallet::Wallet as WalletCore;
use kaspa_wallet_core::wallet::WalletCreateArgs;

use crate::account::PyAccount;

#[pyclass]
pub struct Wallet {
    wallet: Option<Arc<WalletCore>>,
    resident: bool,
}

#[pymethods]
impl Wallet {
    #[new]
    fn new(resident: Option<bool>) -> Self {
        Wallet {
            wallet: None,
            resident: resident.unwrap_or(true),
        }
    }

    pub fn connect<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let storage = if self.resident == true {
            WalletCore::resident_store().unwrap()
        } else {
            WalletCore::local_store().unwrap()
        };

        self.wallet = Some(Arc::new(kaspa_wallet_core::prelude::Wallet::try_new(storage, None, None).unwrap()));
        let wallet = self.wallet.clone().unwrap();

        pyo3_asyncio::tokio::future_into_py(py, async move {

            let binding = wallet.clone().try_wrpc_client();
            let wrpc_client = binding.as_ref();

            let options = ConnectOptions {
                block_async_connect: true,
                strategy: ConnectStrategy::Fallback,
                url: Some("ws://127.0.0.1:17110".to_string()),
                ..Default::default()
            };

            wrpc_client.unwrap().connect(Some(options)).await.unwrap();

            wallet.set_network_id(&NetworkId::new(NetworkType::Mainnet)).unwrap();

            let _ = wallet.start().await; // TODO: Do we always need it with the python wallet api ?

            Ok(wallet.is_connected())
        })
    }

    pub fn create_account<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let wallet = self.wallet.clone().unwrap();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let wallet_secret = Secret::new(vec![]);
            wallet.create_wallet(&wallet_secret, WalletCreateArgs::new(None, None, EncryptionKind::XChaCha20Poly1305, None, true)).await.unwrap();

            let prv_key_data_args = PrvKeyDataCreateArgs::new(
                None,
                None,
                Secret::from(Mnemonic::random(WordCount::Words24, Language::default()).unwrap().phrase()),
            );
            let prv_key_data_id = wallet.create_prv_key_data(&wallet_secret, prv_key_data_args).await.unwrap();

            let account_args = AccountCreateArgsBip32::new(None, None);
            let account = wallet.create_account_bip32(&wallet_secret, prv_key_data_id, None, account_args).await.unwrap();

            Ok(PyAccount::new(account as Arc<dyn Account + Send + Sync>))
        })
    }

    pub fn import_account<'a>(&mut self, py: Python<'a>, mnemonic: String) -> PyResult<&'a PyAny> {
        let wallet = self.wallet.clone().unwrap();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let wallet_secret = Secret::new(vec![]);
            wallet.create_wallet(&wallet_secret, WalletCreateArgs::new(None, None, EncryptionKind::XChaCha20Poly1305, None, true)).await.unwrap();

            let prv_key_data_args = PrvKeyDataCreateArgs::new(None, None, Secret::from(mnemonic));
            let prv_key_data_id = wallet.create_prv_key_data(&wallet_secret, prv_key_data_args).await.unwrap();

            let account_args = AccountCreateArgsBip32::new(None, None);
            let account = wallet.create_account_bip32(&wallet_secret, prv_key_data_id, None, account_args).await.unwrap();

            Ok(PyAccount::new(account as Arc<dyn Account + Send + Sync>))
        })
    }
}