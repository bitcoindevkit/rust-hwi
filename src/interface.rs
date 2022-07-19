use std::ops::Deref;

use bitcoin::consensus::encode::serialize;
use bitcoin::util::bip32::DerivationPath;
use bitcoin::util::psbt::PartiallySignedTransaction;

use bitcoin::base64;

use serde_json::value::Value;

use crate::error::Error;
use crate::types::{
    HWIAddress, HWIAddressType, HWIChain, HWIDescriptor, HWIDevice, HWIExtendedPubKey,
    HWIKeyPoolElement, HWIPartiallySignedTransaction, HWISignature, HWIStatus, LogLevel,
};

use pyo3::{prelude::*, py_run};

macro_rules! deserialize_obj {
    ( $e: expr ) => {{
        let value: Value = serde_json::from_str($e)?;
        let obj = value.clone();
        serde_json::from_value(value)
            .map_err(|e| Error::HWIError(format!("Error {} while deserializing {}", e, obj)))
    }};
}

/// Convenience class containing required Python objects
#[derive(Debug)]
struct HWILib {
    commands: Py<PyModule>,
    json_dumps: Py<PyAny>,
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

#[derive(Debug)]
pub struct HWIClient {
    hwilib: HWILib,
    hw_client: PyObject,
}

impl Deref for HWIClient {
    type Target = PyObject;

    fn deref(&self) -> &Self::Target {
        &self.hw_client
    }
}

impl HWIClient {
    /// Lists all HW devices currently connected.
    /// ```no_run
    /// # use hwi::HWIClient;
    /// # use hwi::error::Error;
    /// # fn main() -> Result<(), Error> {
    /// let devices = HWIClient::enumerate()?;
    /// for device in devices {
    ///     println!("I can see a {} here 😄", device.model);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn enumerate() -> Result<Vec<HWIDevice>, Error> {
        let libs = HWILib::initialize()?;
        Python::with_gil(|py| {
            let output = libs.commands.getattr(py, "enumerate")?.call0(py)?;
            let output = libs.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Returns the HWIClient for a certain device. You can list all the available devices using
    /// [`enumerate`](HWIClient::enumerate).
    ///
    /// Setting `expert` to `true` will enable additional output for some commands.
    /// ```
    /// # use hwi::HWIClient;
    /// # use hwi::types::*;
    /// # use hwi::error::Error;
    /// # fn main() -> Result<(), Error> {
    /// let devices = HWIClient::enumerate()?;
    /// for device in devices {
    ///     let client = HWIClient::get_client(&device, false, HWIChain::Test)?;
    ///     let xpub = client.get_master_xpub(HWIAddressType::Tap, 0)?;
    ///     println!(
    ///         "I can see a {} here, and its xpub is {}",
    ///         device.model,
    ///         xpub.to_string()
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_client(
        device: &HWIDevice,
        expert: bool,
        chain: HWIChain,
    ) -> Result<HWIClient, Error> {
        let libs = HWILib::initialize()?;
        Python::with_gil(|py| {
            let client_args = (&device.device_type, &device.path, "", expert, chain);
            let client = libs
                .commands
                .getattr(py, "get_client")?
                .call1(py, client_args)?;
            Ok(HWIClient {
                hwilib: libs,
                hw_client: client,
            })
        })
    }

    /// Returns the master xpub of a device, given the address type and the account number.
    pub fn get_master_xpub(
        &self,
        addrtype: HWIAddressType,
        account: u32,
    ) -> Result<HWIExtendedPubKey, Error> {
        Python::with_gil(|py| {
            let output = self
                .hwilib
                .commands
                .getattr(py, "getmasterxpub")?
                .call1(py, (&self.hw_client, addrtype, account))?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Signs a PSBT.
    pub fn sign_tx(
        &self,
        psbt: &PartiallySignedTransaction,
    ) -> Result<HWIPartiallySignedTransaction, Error> {
        let psbt = base64::encode(&serialize(psbt));
        Python::with_gil(|py| {
            let output = self
                .hwilib
                .commands
                .getattr(py, "signtx")?
                .call1(py, (&self.hw_client, psbt))?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Returns the xpub of a device. If `expert` is set, additional output is returned.
    pub fn get_xpub(
        &self,
        path: &DerivationPath,
        expert: bool,
    ) -> Result<HWIExtendedPubKey, Error> {
        Python::with_gil(|py| {
            let func_args = (&self.hw_client, path.to_string(), expert);
            let output = self
                .hwilib
                .commands
                .getattr(py, "getxpub")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Signs a message.
    pub fn sign_message(
        &self,
        message: &str,
        path: &DerivationPath,
    ) -> Result<HWISignature, Error> {
        Python::with_gil(|py| {
            let func_args = (&self.hw_client, message, path.to_string());
            let output = self
                .hwilib
                .commands
                .getattr(py, "signmessage")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Returns an array of keys that can be imported in Bitcoin core using importmulti
    ///
    /// * `keypool` - `keypool` value in result. Check bitcoin core importmulti documentation for further information
    /// * `internal` - Whether to use internal (change) or external keys
    /// * `addr_type` - Address type to use
    /// * `addr_all` - Whether to return a multiple descriptors for every address type
    /// * `account` - Optional BIP43 account to use
    /// * `path` - The derivation path to derive the keys.
    /// * `start` - Keypool start
    /// * `end` - Keypool end
    #[allow(clippy::too_many_arguments)]
    pub fn get_keypool(
        &self,
        keypool: bool,
        internal: bool,
        addr_type: HWIAddressType,
        addr_all: bool,
        account: Option<u32>,
        path: Option<&DerivationPath>,
        start: u32,
        end: u32,
    ) -> Result<Vec<HWIKeyPoolElement>, Error> {
        Python::with_gil(|py| {
            let mut p_str = py.None();
            if let Some(p) = path {
                p_str = format!("{}/*", p).into_py(py);
            }
            let func_args = (
                &self.hw_client,
                p_str,
                start,
                end,
                internal,
                keypool,
                account.unwrap_or(0),
                addr_type,
                addr_all,
            );
            let output = self
                .hwilib
                .commands
                .getattr(py, "getkeypool")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Returns device descriptors. You can optionally specify a BIP43 account to use.
    pub fn get_descriptors(&self, account: Option<u32>) -> Result<HWIDescriptor, Error> {
        Python::with_gil(|py| {
            let func_args = (&self.hw_client, account.unwrap_or(0));
            let output = self
                .hwilib
                .commands
                .getattr(py, "getdescriptors")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Returns an address given a descriptor. Note that HWI doesn't support descriptors checksums.
    pub fn display_address_with_desc(&self, descriptor: &str) -> Result<HWIAddress, Error> {
        Python::with_gil(|py| {
            let path = py.None();
            let func_args = (&self.hw_client, path, descriptor);
            let output = self
                .hwilib
                .commands
                .getattr(py, "displayaddress")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Returns an address given path and address type.
    pub fn display_address_with_path(
        &self,
        path: &DerivationPath,
        address_type: HWIAddressType,
    ) -> Result<HWIAddress, Error> {
        Python::with_gil(|py| {
            let descriptor = py.None();
            let func_args = (&self.hw_client, path.to_string(), descriptor, address_type);
            let output = self
                .hwilib
                .commands
                .getattr(py, "displayaddress")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            deserialize_obj!(&output.to_string())
        })
    }

    /// Install the udev rules to the local machine.
    ///
    /// The rules will be copied from the source to the location; the default source location is
    /// `./udev`, the default destination location is `/lib/udev/rules.d`
    pub fn install_udev_rules(source: Option<&str>, location: Option<&str>) -> Result<(), Error> {
        Python::with_gil(|py| {
            let libs = HWILib::initialize()?;
            let func_args = (
                source.unwrap_or("./udev"),
                location.unwrap_or("/lib/udev/rules.d/"),
            );
            let output = libs
                .commands
                .getattr(py, "install_udev_rules")?
                .call1(py, func_args)?;
            let output = libs.json_dumps.call1(py, (output,))?;
            let status: HWIStatus = deserialize_obj!(&output.to_string())?;
            status.into()
        })
    }

    /// Set logging level
    /// # Arguments
    /// * `level` - Log level.
    pub fn set_log_level(level: LogLevel) -> Result<(), Error> {
        Python::with_gil(|py| {
            let arg = match level {
                LogLevel::DEBUG => 10,
                LogLevel::INFO => 20,
                LogLevel::WARNING => 30,
                LogLevel::ERROR => 40,
                LogLevel::CRITICAL => 50,
            };
            py_run!(
                py,
                arg,
                r#"
                import logging
                logging.basicConfig(level=arg)            
                "#
            );
            Ok(())
        })
    }

    /// Toggle whether the device is using a BIP 39 passphrase.
    pub fn toggle_passphrase(&self) -> Result<(), Error> {
        Python::with_gil(|py| {
            let func_args = (&self.hw_client,);
            let output = self
                .hwilib
                .commands
                .getattr(py, "toggle_passphrase")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            let status: HWIStatus = deserialize_obj!(&output.to_string())?;
            status.into()
        })
    }

    /// Wipe a device
    pub fn wipe_device(&self) -> Result<(), Error> {
        Python::with_gil(|py| {
            let func_args = (&self.hw_client,);
            let output = self
                .hwilib
                .commands
                .getattr(py, "toggle_passphrase")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            let status: HWIStatus = deserialize_obj!(&output.to_string())?;
            status.into()
        })
    }
}
