use bdk::signer::{SignerCommon, SignerError, SignerId, TransactionSigner};
use bitcoin::{
    psbt::PartiallySignedTransaction,
    secp256k1::{All, Secp256k1},
    util::bip32::Fingerprint,
};

use crate::{
    error::Error,
    types::{HWIChain, HWIDevice},
    HWIClient,
};

#[derive(Debug)]
pub struct HWISigner {
    fingerprint: Fingerprint,
    client: HWIClient,
}

impl HWISigner {
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
        psbt: &mut PartiallySignedTransaction,
        _secp: &Secp256k1<All>,
    ) -> Result<(), SignerError> {
        psbt.combine(
            self.client
                .sign_tx(psbt)
                .expect("Hardware Wallet couldn't sign transaction")
                .psbt,
        )
        .expect("Failed to combine HW signed psbt with passed PSBT");
        Ok(())
    }
}
