use std::str::from_utf8;

use bitcoin::consensus::encode::serialize;
use bitcoin::util::bip32::{DerivationPath, Fingerprint};
use bitcoin::util::psbt::PartiallySignedTransaction;

use base64;

use serde::Deserialize;
use serde_json::value::Value;

use crate::commands::{HWICommand, HWIFlag, HWISubcommand};
use crate::error::Error;
use crate::types::{
    HWIAddress, HWIAddressType, HWIDescriptor, HWIExtendedPubKey, HWIKeyPoolElement,
    HWIPartiallySignedTransaction, HWISignature, HWIChain,
};

macro_rules! deserialize_obj {
    ( $e: expr ) => {{
        let value: Value = serde_json::from_str(from_utf8($e)?)?;
        let obj = value.clone();
        serde_json::from_value(value)
            .map_err(|e| Error::HWIError(format!("Error {:?} while deserializing {:?}", e, obj)))
    }};
}

#[derive(Clone, Deserialize)]
pub struct HWIDevice {
    #[serde(rename(deserialize = "type"))]
    pub device_type: String,
    pub model: String,
    pub path: String,
    pub needs_pin_sent: bool,
    pub needs_passphrase_sent: bool,
    pub fingerprint: Fingerprint,
}

impl HWIDevice {
    /// Lists all HW devices currently connected.
    pub fn enumerate() -> Result<Vec<HWIDevice>, Error> {
        let output = HWICommand::new()
            .add_subcommand(HWISubcommand::Enumerate)
            .execute()?;
        deserialize_obj!(&output.stdout)
    }

    /// Returns the master xpub of a device.
    /// # Arguments
    /// * `testnet` - Whether to use testnet or not.
    pub fn get_master_xpub(&self, chain: HWIChain) -> Result<HWIExtendedPubKey, Error> {
        let output = HWICommand::new()
            .add_flag(HWIFlag::Fingerprint(self.fingerprint))
            .add_chain(chain)
            .add_subcommand(HWISubcommand::GetMasterXpub)
            .execute()?;
        deserialize_obj!(&output.stdout)
    }

    /// Returns a psbt signed.
    /// # Arguments
    /// * `psbt` - The PSBT to be signed.
    /// * `testnet` - Whether to use testnet or not.
    pub fn sign_tx(
        &self,
        psbt: &PartiallySignedTransaction,
        chain: HWIChain,
    ) -> Result<HWIPartiallySignedTransaction, Error> {
        let psbt = base64::encode(&serialize(psbt));
        let output = HWICommand::new()
            .add_flag(HWIFlag::Fingerprint(self.fingerprint))
            .add_chain(chain)
            .add_subcommand(HWISubcommand::SignTx)
            .add_psbt(&psbt)
            .execute()?;
        deserialize_obj!(&output.stdout)
    }

    /// Returns the xpub of a device.
    /// # Arguments
    /// * `path` - The derivation path to derive the key.
    /// * `testnet` - Whether to use testnet or not.
    pub fn get_xpub(
        &self,
        path: &DerivationPath,
        chain: HWIChain,
    ) -> Result<HWIExtendedPubKey, Error> {
        let output = HWICommand::new()
            .add_flag(HWIFlag::Fingerprint(self.fingerprint))
            .add_chain(chain)
            .add_subcommand(HWISubcommand::GetXpub)
            .add_path(&path, false, false)
            .execute()?;
        deserialize_obj!(&output.stdout)
    }

    /// Signs a message.
    /// # Arguments
    /// * `message` - The message to sign.
    /// * `path` - The derivation path to derive the key.
    /// * `testnet` - Whether to use testnet or not.
    pub fn sign_message(
        &self,
        message: &str,
        path: &DerivationPath,
        chain: HWIChain,
    ) -> Result<HWISignature, Error> {
        let output = HWICommand::new()
            .add_flag(HWIFlag::Fingerprint(self.fingerprint))
            .add_chain(chain)
            .add_subcommand(HWISubcommand::SignMessage)
            .add_message(&message)
            .add_path(&path, false, false)
            .execute()?;
        deserialize_obj!(&output.stdout)
    }

    /// Returns an array of keys that can be imported in Bitcoin core using importmulti
    /// # Arguments
    /// * `keypool` - `keypool` value in result. Check bitcoin core importmulti documentation for further information
    /// * `internal` - Whether to use internal (change) or external keys
    /// * `address type` - HWIAddressType to use
    /// * `account` - Optional BIP43 account to use
    /// * `path` - The derivation path to derive the keys.
    /// * `start` - Keypool start
    /// * `end` - Keypool end
    /// * `testnet` - Whether to use testnet or not.
    pub fn get_keypool(
        &self,
        keypool: bool,
        internal: bool,
        address_type: HWIAddressType,
        account: Option<u32>,
        path: Option<&DerivationPath>,
        start: u32,
        end: u32,
        chain: HWIChain,
    ) -> Result<Vec<HWIKeyPoolElement>, Error> {
        let mut command = HWICommand::new();
        command
            .add_flag(HWIFlag::Fingerprint(self.fingerprint))
            .add_chain(chain)
            .add_subcommand(HWISubcommand::GetKeypool)
            .add_keypool(keypool)
            .add_internal(internal)
            .add_address_type(address_type);

        if let Some(a) = account {
            command.add_account(a);
        }

        if let Some(p) = path {
            command.add_path(&p, true, true);
        }

        let output = command.add_start_end(start, end).execute()?;
        deserialize_obj!(&output.stdout)
    }

    /// Returns device descriptors
    /// # Arguments
    /// * `account` - Optional BIP43 account to use.
    /// * `testnet` - Whether to use testnet or not.
    pub fn get_descriptors(
        &self,
        account: Option<u32>,
        chain: HWIChain,
    ) -> Result<HWIDescriptor, Error> {
        let mut command = HWICommand::new();
        command
            .add_flag(HWIFlag::Fingerprint(self.fingerprint))
            .add_chain(chain)
            .add_subcommand(HWISubcommand::GetDescriptors);

        if let Some(a) = account {
            command.add_account(a);
        };

        let output = command.execute()?;
        deserialize_obj!(&output.stdout)
    }

    /// Returns an address given a descriptor.
    /// # Arguments
    /// * `descriptor` - The descriptor to use. HWI doesn't support descriptors checksums.
    /// * `testnet` - Whether to use testnet or not.
    pub fn display_address_with_desc(
        &self,
        descriptor: &str,
        chain: HWIChain,
    ) -> Result<HWIAddress, Error> {
        let output = HWICommand::new()
            .add_flag(HWIFlag::Fingerprint(self.fingerprint))
            .add_chain(chain)
            .add_subcommand(HWISubcommand::DisplayAddress)
            .add_descriptor(&descriptor)
            .execute()?;
        deserialize_obj!(&output.stdout)
    }

    /// Returns an address given pat and address type
    /// # Arguments
    /// * `path` - The derivation path to use.
    /// * `address_type` - Address type to use.
    /// * `testnet` - Whether to use testnet or not.
    pub fn display_address_with_path(
        &self,
        path: &DerivationPath,
        address_type: HWIAddressType,
        chain: HWIChain,
    ) -> Result<HWIAddress, Error> {
        let output = HWICommand::new()
            .add_flag(HWIFlag::Fingerprint(self.fingerprint))
            .add_chain(chain)
            .add_subcommand(HWISubcommand::DisplayAddress)
            .add_path(&path, true, false)
            .add_address_type(address_type)
            .execute()?;
        deserialize_obj!(&output.stdout)
    }
}
