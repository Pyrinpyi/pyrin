use pyo3::prelude::*;

use kaspa_bip32::{Language, Mnemonic, WordCount, Prefix as KeyPrefix};
use kaspa_wallet_core::account::BIP32_ACCOUNT_KIND;
use kaspa_wallet_core::prelude::{EncryptionKind, Secret};
use kaspa_wallet_core::storage::PrvKeyData;
use kaspa_addresses::{Address, Prefix, Version};
use kaspa_bip32::secp256k1::serde::Serialize;

#[pyclass]
pub struct Bip32 {

}

#[pymethods]
impl Bip32 {
    #[new]
    fn new() -> Self {
        Bip32 {}
    }

    #[staticmethod]
    fn generate_mnemonic() -> PyResult<String> {
        let mnemonic = Mnemonic::random(WordCount::Words24, Language::English).unwrap();
        Ok(mnemonic.phrase().to_string())
    }

    #[staticmethod]
    fn generate_short_mnemonic() -> PyResult<String> {
        let mnemonic = Mnemonic::random(WordCount::Words12, Language::English).unwrap();
        Ok(mnemonic.phrase().to_string())
    }

    #[staticmethod]
    fn generate_mnemonicasd(py: Python) -> PyResult<&PyAny> {
        pyo3_asyncio::tokio::future_into_py(py, async {
            let mnemonic = Mnemonic::random(WordCount::Words24, Language::English).unwrap();
            let account_index = 0;
            let payment_secret = Some(Secret::from("".to_string()));
            let prv_key_data = PrvKeyData::try_from_mnemonic(mnemonic, payment_secret.as_ref(), EncryptionKind::default()).unwrap();
            let xpub_key = prv_key_data
                .create_xpub(payment_secret.as_ref(), BIP32_ACCOUNT_KIND.into(), account_index)
                .await.unwrap();
            // Ok(xpub_key.to_string(Some(KeyPrefix::XPUB)))
            // Ok(xpub_key.public_key.to_string())
            Ok(Address::new(Prefix::Mainnet, Version::PubKey, xpub_key.public_key.x_only_public_key().0.serialize().as_slice()).to_string())
        })
    }
}