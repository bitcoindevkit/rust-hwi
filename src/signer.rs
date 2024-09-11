use bdk_wallet::bitcoin::bip32::Fingerprint;
use bdk_wallet::bitcoin::secp256k1::{All, Secp256k1};
use bdk_wallet::bitcoin::Psbt;

use crate::error::Error;
use crate::types::{HWIChain, HWIDevice};
use crate::HWIClient;

use bdk_wallet::signer::{SignerCommon, SignerError, SignerId, TransactionSigner};

#[derive(Debug)]
/// Custom signer for Hardware Wallets
///
/// This ignores `sign_options` and leaves the decisions up to the hardware wallet.
pub struct HWISigner {
    fingerprint: Fingerprint,
    client: HWIClient,
}

impl HWISigner {
    /// Create an instance from the specified device and chain
    pub fn from_device(device: &HWIDevice, chain: HWIChain) -> Result<HWISigner, Error> {
        let client = HWIClient::get_client(device, false, chain)?;
        Ok(HWISigner {
            fingerprint: device.fingerprint,
            client,
        })
    }
}

impl SignerCommon for HWISigner {
    fn id(&self, _secp: &Secp256k1<All>) -> SignerId {
        SignerId::Fingerprint(self.fingerprint)
    }
}

impl TransactionSigner for HWISigner {
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
