use std::convert::TryInto;
use std::ops::Deref;
use std::process::Command;

use bitcoin::bip32::DerivationPath;
use bitcoin::psbt::PartiallySignedTransaction;

use serde::de::DeserializeOwned;
use serde_json::value::Value;

use crate::error::Error;
use crate::types::{
    HWIAddress, HWIAddressType, HWIChain, HWIDescriptor, HWIDevice, HWIDeviceInternal,
    HWIDeviceType, HWIExtendedPubKey, HWIKeyPoolElement, HWIPartiallySignedTransaction,
    HWISignature, HWIStatus, HWIWordCount, LogLevel, ToDescriptor,
};

use pyo3::{prelude::*, py_run};

macro_rules! deserialize_obj {
    ( $e: expr ) => {{
        let value: Value = serde_json::from_str($e)?;
        let obj = value.clone();
        serde_json::from_value(value)
            .map_err(|e| Error::Hwi(format!("error {} while deserializing {}", e, obj), None))
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
    ///     match device {
    ///         Ok(d) => println!("I can see a {} here 😄", d.model),
    ///         Err(e) => println!("Uh oh, something went wrong when opening the device: {}", e),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn enumerate() -> Result<Vec<Result<HWIDevice, Error>>, Error> {
        let libs = HWILib::initialize()?;
        Python::with_gil(|py| {
            let output = libs.commands.getattr(py, "enumerate")?.call0(py)?;
            let output = libs.json_dumps.call1(py, (output,))?;
            let devices_internal: Vec<HWIDeviceInternal> = deserialize_obj!(&output.to_string())?;
            Ok(devices_internal.into_iter().map(|d| d.try_into()).collect())
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
    ///     let device = device?;
    ///     let client = HWIClient::get_client(&device, false, bitcoin::Network::Testnet.into())?;
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
            let client_args = (
                device.device_type.to_string(),
                &device.path,
                "",
                expert,
                chain,
            );
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

    /// Returns the HWIClient for a certain `device_type` or `fingerprint`. You can list all the available devices using
    /// [`enumerate`](HWIClient::enumerate).
    ///
    /// Setting `expert` to `true` will enable additional output for some commands.
    /// ```no_run
    /// # use hwi::HWIClient;
    /// # use hwi::types::*;
    /// # use hwi::error::Error;
    /// # fn main() -> Result<(), Error> {
    /// let client = HWIClient::find_device(
    ///     None,
    ///     Some(HWIDeviceType::Trezor),
    ///     None,
    ///     false,
    ///     bitcoin::Network::Testnet,
    /// )?;
    /// let xpub = client.get_master_xpub(HWIAddressType::Tap, 0)?;
    /// println!("Trezor's xpub is {}", xpub.to_string());
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_device(
        password: Option<&str>,
        device_type: Option<HWIDeviceType>,
        fingerprint: Option<&str>,
        expert: bool,
        chain: bitcoin::Network,
    ) -> Result<HWIClient, Error> {
        let libs = HWILib::initialize()?;
        Python::with_gil(|py| {
            let client_args = (
                password.unwrap_or(""),
                device_type.map_or_else(String::new, |d| d.to_string()),
                fingerprint.unwrap_or(""),
                expert,
                HWIChain::from(chain),
            );
            let client = libs
                .commands
                .getattr(py, "find_device")?
                .call1(py, client_args)?;

            if client.is_none(py) {
                return Err(Error::Hwi("device not found".to_string(), None));
            }

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
        Python::with_gil(|py| {
            let output = self
                .hwilib
                .commands
                .getattr(py, "signtx")?
                .call1(py, (&self.hw_client, psbt.to_string()))?;
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
    pub fn get_descriptors<T>(&self, account: Option<u32>) -> Result<HWIDescriptor<T>, Error>
    where
        T: ToDescriptor + DeserializeOwned,
    {
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

    /// Returns an address given a descriptor.
    pub fn display_address_with_desc<T>(&self, descriptor: &T) -> Result<HWIAddress, Error>
    where
        T: ToDescriptor + ToString,
    {
        Python::with_gil(|py| {
            let path = py.None();
            let descriptor = descriptor.to_string().split('#').collect::<Vec<_>>()[0].to_string();
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

    /// Setup a device
    pub fn setup_device(&self, label: Option<&str>, passphrase: Option<&str>) -> Result<(), Error> {
        Python::with_gil(|py| {
            let func_args = (
                &self.hw_client,
                label.unwrap_or(""),
                passphrase.unwrap_or(""),
            );
            let output = self
                .hwilib
                .commands
                .getattr(py, "setup_device")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            let status: HWIStatus = deserialize_obj!(&output.to_string())?;
            status.into()
        })
    }

    /// Restore a device
    pub fn restore_device(
        &self,
        label: Option<&str>,
        word_count: Option<HWIWordCount>,
    ) -> Result<(), Error> {
        Python::with_gil(|py| {
            let word_count: u8 = word_count.map_or_else(|| 24, |w| w as u8);
            let func_args = (&self.hw_client, label.unwrap_or(""), word_count);
            let output = self
                .hwilib
                .commands
                .getattr(py, "restore_device")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            let status: HWIStatus = deserialize_obj!(&output.to_string())?;
            status.into()
        })
    }

    /// Create a backup of the device
    pub fn backup_device(
        &self,
        label: Option<&str>,
        backup_passphrase: Option<&str>,
    ) -> Result<(), Error> {
        Python::with_gil(|py| {
            let func_args = (
                &self.hw_client,
                label.unwrap_or_default(),
                backup_passphrase.unwrap_or_default(),
            );
            let output = self
                .hwilib
                .commands
                .getattr(py, "backup_device")?
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
                .getattr(py, "wipe_device")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            let status: HWIStatus = deserialize_obj!(&output.to_string())?;
            status.into()
        })
    }

    /// Get the installed version of hwilib. Returns None if hwi is not installed.
    pub fn get_version() -> Option<String> {
        Python::with_gil(|py| {
            Some(
                PyModule::import(py, "hwilib")
                    .ok()?
                    .getattr("__version__")
                    .expect("Should have a __version__")
                    .to_string(),
            )
        })
    }

    /// Install hwi for the current user via pip. If no version is specified, the default version from pip will be installed.
    pub fn install_hwilib(version: Option<&str>) -> Result<(), Error> {
        let hwi_with_version = match version {
            Some(ver) => "hwi==".to_owned() + ver,
            None => "hwi".to_owned(),
        };
        let output = Command::new("pip")
            .args(vec!["install", "--user", hwi_with_version.as_str()])
            .output()?;
        if output.status.success() {
            Ok(())
        } else {
            Err(Error::Hwi(
                std::str::from_utf8(&output.stderr)
                    .expect("Non UTF-8 error while installing")
                    .to_string(),
                None,
            ))
        }
    }
}
