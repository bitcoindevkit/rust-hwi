use bdk_wallet::bitcoin::bip32::Fingerprint;
use bdk_wallet::bitcoin::secp256k1::{All, Secp256k1};
use bdk_wallet::bitcoin::Psbt;

use crate::error::Error;
use crate::types::{HWIChain, HWIClient, HWIDevice, HWIImplementation};

use bdk_wallet::signer::{SignerCommon, SignerError, SignerId, TransactionSigner};

#[derive(Debug)]
/// Custom signer for Hardware Wallets
///
/// This ignores `sign_options` and leaves the decisions up to the hardware wallet.
pub struct HWISigner<T: HWIImplementation> {
    fingerprint: Fingerprint,
    client: HWIClient<T>,
}

impl<T: HWIImplementation> HWISigner<T> {
    /// Create an instance from the specified device and chain
    pub fn from_device(device: &HWIDevice, chain: HWIChain) -> Result<Self, Error> {
        let client = HWIClient::<T>::get_client(device, false, chain)?;
        Ok(Self {
            fingerprint: device.fingerprint,
            client,
        })
    }
}

impl<T: HWIImplementation> SignerCommon for HWISigner<T> {
    fn id(&self, _secp: &Secp256k1<All>) -> SignerId {
        SignerId::Fingerprint(self.fingerprint)
    }
}

impl<T: HWIImplementation> TransactionSigner for HWISigner<T> {
    fn sign_transaction(
        &self,
        psbt: &mut Psbt,
        _sign_options: &bdk_wallet::SignOptions,
        _secp: &Secp256k1<All>,
    ) -> Result<(), SignerError> {
        psbt.combine(
            self.client
                .sign_tx(psbt)
                .map_err(|e| {
                    SignerError::External(format!("While signing with hardware wallet: {}", e))
                })?
                .psbt,
        )
        .expect("Failed to combine HW signed psbt with passed PSBT");
        Ok(())
    }
}
