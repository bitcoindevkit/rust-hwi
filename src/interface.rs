use bitcoin::consensus::encode::serialize;
use bitcoin::util::bip32::{DerivationPath, Fingerprint};
use bitcoin::util::psbt::PartiallySignedTransaction;

use base64;

use serde::Deserialize;
use serde_json::value::Value;

use crate::error::Error;
use crate::types::{
    HWIAddress, HWIAddressType, HWIChain, HWIDescriptor, HWIExtendedPubKey, HWIKeyPoolElement,
    HWIPartiallySignedTransaction, HWISignature,
};

use pyo3::prelude::*;

macro_rules! deserialize_obj {
    ( $e: expr ) => {{
        let value: Value = serde_json::from_str($e)?;
        let obj = value.clone();
        serde_json::from_value(value)
            .map_err(|e| Error::HWIError(format!("Error {:?} while deserializing {:?}", e, obj)))
    }};
}

/// Convenience class containing required Python objects
pub struct HWILib {
    pub commands: Py<PyModule>,
    pub json_dumps: Py<PyAny>,
}

impl HWILib {
    pub fn initialize() -> Result<Self, Error> {
        Python::with_gil(|py| {
            let commands: Py<PyModule> = PyModule::import(py, "hwilib.commands")?.into();
            let json_dumps: Py<PyAny> = PyModule::import(py, "json")?.getattr("dumps")?.into();
            Ok(HWILib {
                commands,
                json_dumps,
            })
        })
    }
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
    pub fn enumerate(libs: &HWILib) -> Result<Vec<HWIDevice>, Error> {
        Python::with_gil(|py| {
            let output = libs.commands.getattr(py, "enumerate")?.call0(py)?;
            let output = libs.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Finds the Python object of the device corresponding to `Self`
    /// # Arguements
    /// * `expert` - Whether the device should be opened in expert mode (enables additional output for some actions)
    /// * `chain` - The Chain this client will be using
    /// * `libs` - HWILib instance
    pub fn find_device(
        &self,
        expert: bool,
        chain: HWIChain,
        libs: &HWILib,
    ) -> Result<PyObject, Error> {
        Python::with_gil(|py| {
            let client_args = ("", py.None(), self.fingerprint.to_string(), expert, chain);
            let client = libs
                .commands
                .getattr(py, "find_device")?
                .call1(py, client_args)?;
            Ok(client)
        })
    }

    /// Returns the master xpub of a device.
    /// # Arguments
    /// * `client` - The (Python) client to interact with
    /// * `libs` - HWILib instance
    pub fn get_master_xpub(
        &self,
        client: &PyObject,
        addrtype: HWIAddressType,
        account: u32,
        libs: &HWILib,
    ) -> Result<HWIExtendedPubKey, Error> {
        Python::with_gil(|py| {
            let output = libs
                .commands
                .getattr(py, "getmasterxpub")?
                .call1(py, (client, addrtype, account))?;
            let output = libs.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Returns a psbt signed.
    /// # Arguments
    /// * `client` - The (Python) client to interact with
    /// * `psbt` - The PSBT to be signed.
    /// * `libs` - HWILib instance
    pub fn sign_tx(
        &self,
        client: &PyObject,
        psbt: &PartiallySignedTransaction,
        libs: &HWILib,
    ) -> Result<HWIPartiallySignedTransaction, Error> {
        let psbt = base64::encode(&serialize(psbt));
        Python::with_gil(|py| {
            let output = libs
                .commands
                .getattr(py, "signtx")?
                .call1(py, (client, psbt))?;
            let output = libs.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Returns the xpub of a device.
    /// # Arguments
    /// * `path` - The derivation path to derive the key.
    /// * `client` - The (Python) client to interact with
    /// * `expert` - Whether the device should be opened in expert mode (enables additional output for some actions)
    /// * `libs` - HWILib instance
    pub fn get_xpub(
        &self,
        client: &PyObject,
        path: &DerivationPath,
        expert: bool,
        libs: &HWILib,
    ) -> Result<HWIExtendedPubKey, Error> {
        Python::with_gil(|py| {
            let func_args = (client, path.to_string(), expert);
            let output = libs.commands.getattr(py, "getxpub")?.call1(py, func_args)?;
            let output = libs.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Signs a message.
    /// # Arguments
    /// * `client` - The (Python) client to interact with
    /// * `message` - The message to sign.
    /// * `path` - The derivation path to derive the key.
    /// * `libs` - HWILib instance
    pub fn sign_message(
        &self,
        client: &PyObject,
        message: &str,
        path: &DerivationPath,
        libs: &HWILib,
    ) -> Result<HWISignature, Error> {
        Python::with_gil(|py| {
            let func_args = (client, message, path.to_string());
            let output = libs
                .commands
                .getattr(py, "signmessage")?
                .call1(py, func_args)?;
            let output = libs.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Returns an array of keys that can be imported in Bitcoin core using importmulti
    /// # Arguments
    /// * `client` - The (Python) client to interact with
    /// * `keypool` - `keypool` value in result. Check bitcoin core importmulti documentation for further information
    /// * `internal` - Whether to use internal (change) or external keys
    /// * `addr_type` - HWIAddressType to use
    /// * `addr_all` - Whether to return a multiple descriptors for every address type
    /// * `account` - Optional BIP43 account to use
    /// * `path` - The derivation path to derive the keys.
    /// * `start` - Keypool start
    /// * `end` - Keypool end
    /// * `libs` - HWILib instance
    #[allow(clippy::too_many_arguments)]
    pub fn get_keypool(
        &self,
        client: &PyObject,
        keypool: bool,
        internal: bool,
        addr_type: HWIAddressType,
        addr_all: bool,
        account: Option<u32>,
        path: Option<&DerivationPath>,
        start: u32,
        end: u32,
        libs: &HWILib,
    ) -> Result<Vec<HWIKeyPoolElement>, Error> {
        Python::with_gil(|py| {
            let mut p_str = py.None();
            if let Some(p) = path {
                p_str = format!("{}/*", p).into_py(py);
            }
            let func_args = (
                client,
                p_str,
                start,
                end,
                internal,
                keypool,
                account.unwrap_or(0),
                addr_type,
                addr_all,
            );
            let output = libs
                .commands
                .getattr(py, "getkeypool")?
                .call1(py, func_args)?;
            let output = libs.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Returns device descriptors
    /// # Arguments
    /// * `client` - The (Python) client to interact with
    /// * `account` - Optional BIP43 account to use.
    /// * `libs` - HWILib instance
    pub fn get_descriptors(
        &self,
        client: &PyObject,
        account: Option<u32>,
        libs: &HWILib,
    ) -> Result<HWIDescriptor, Error> {
        Python::with_gil(|py| {
            let func_args = (client, account.unwrap_or(0));
            let output = libs
                .commands
                .getattr(py, "getdescriptors")?
                .call1(py, func_args)?;
            let output = libs.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Returns an address given a descriptor.
    /// # Arguments
    /// * `client` - The (Python) client to interact with
    /// * `descriptor` - The descriptor to use. HWI doesn't support descriptors checksums.
    /// * `libs` - HWILib instance
    pub fn display_address_with_desc(
        &self,
        client: &PyObject,
        descriptor: &str,
        libs: &HWILib,
    ) -> Result<HWIAddress, Error> {
        Python::with_gil(|py| {
            let path = py.None();
            let func_args = (client, path, descriptor);
            let output = libs
                .commands
                .getattr(py, "displayaddress")?
                .call1(py, func_args)?;
            let output = libs.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Returns an address given pat and address type
    /// # Arguments
    /// * `client` - The (Python) client to interact with
    /// * `path` - The derivation path to use.
    /// * `address_type` - Address type to use.
    /// * `libs` - HWILib instance
    pub fn display_address_with_path(
        &self,
        client: &PyObject,
        path: &DerivationPath,
        address_type: HWIAddressType,
        libs: &HWILib,
    ) -> Result<HWIAddress, Error> {
        Python::with_gil(|py| {
            let descriptor = py.None();
            let func_args = (client, path.to_string(), descriptor, address_type);
            let output = libs
                .commands
                .getattr(py, "displayaddress")?
                .call1(py, func_args)?;
            let output = libs.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }
}
