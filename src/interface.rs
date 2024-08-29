use std::convert::TryInto;

use bitcoin::bip32::DerivationPath;
use bitcoin::Psbt;

use serde::de::DeserializeOwned;
use serde_json::value::Value;

use crate::error::Error;
use crate::types::{
    HWIAddress, HWIAddressType, HWIChain, HWIDescriptor, HWIDevice, HWIDeviceInternal,
    HWIDeviceType, HWIExtendedPubKey, HWIImplementation, HWIKeyPoolElement,
    HWIPartiallySignedTransaction, HWISignature, HWIStatus, HWIWordCount, LogLevel, ToDescriptor,
};

macro_rules! deserialize_obj {
    ( $e: expr ) => {{
        let value: Value = serde_json::from_str($e)?;
        let obj = value.clone();
        serde_json::from_value(value)
            .map_err(|e| Error::Hwi(format!("error {} while deserializing {}", e, obj), None))
    }};
}

pub struct HWIClient<T: HWIImplementation> {
    implementation: T,
}

impl<T: HWIImplementation> HWIClient<T> {
    /// Lists all HW devices currently connected.
    /// ```no_run
    /// # use hwi::HWIClient;
    /// # use hwi::implementations::python_implementation::PythonHWIImplementation;
    /// # use hwi::error::Error;
    /// # fn main() -> Result<(), Error> {
    /// let devices = HWIClient::<PythonHWIImplementation>::enumerate()?;
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
        let output = T::enumerate()?;
        let devices_internal: Vec<HWIDeviceInternal> = deserialize_obj!(&output)?;
        Ok(devices_internal.into_iter().map(|d| d.try_into()).collect())
    }

    /// Returns the HWIClient for a certain device. You can list all the available devices using
    /// [`enumerate`](HWIClient::enumerate).
    ///
    /// Setting `expert` to `true` will enable additional output for some commands.
    /// ```
    /// # use hwi::HWIClient;
    /// # use hwi::types::*;
    /// # use hwi::error::Error;
    /// # use hwi::implementations::python_implementation::PythonHWIImplementation;
    /// # fn main() -> Result<(), Error> {
    /// let devices = HWIClient::<PythonHWIImplementation>::enumerate()?;
    /// for device in devices {
    ///     let device = device?;
    ///     let client = HWIClient::<PythonHWIImplementation>::get_client(
    ///         &device,
    ///         false,
    ///         bitcoin::Network::Testnet.into(),
    ///     )?;
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
    pub fn get_client(device: &HWIDevice, expert: bool, chain: HWIChain) -> Result<Self, Error> {
        let implementation = T::get_client(device, expert, chain)?;

        Ok(Self { implementation })
    }

    /// Returns the HWIClient for a certain `device_type` or `fingerprint`. You can list all the available devices using
    /// [`enumerate`](HWIClient::enumerate).
    ///
    /// Setting `expert` to `true` will enable additional output for some commands.
    /// ```no_run
    /// # use hwi::HWIClient;
    /// # use hwi::types::*;
    /// # use hwi::error::Error;
    /// # use hwi::implementations::python_implementation::PythonHWIImplementation;
    /// # fn main() -> Result<(), Error> {
    /// let client = HWIClient::<PythonHWIImplementation>::find_device(
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
    ) -> Result<Self, Error> {
        let implementation = T::find_device(
            password,
            device_type,
            fingerprint,
            expert,
            HWIChain::from(chain),
        )?;

        Ok(Self { implementation })
    }

    /// Returns the master xpub of a device, given the address type and the account number.
    pub fn get_master_xpub(
        &self,
        addrtype: HWIAddressType,
        account: u32,
    ) -> Result<HWIExtendedPubKey, Error> {
        let output = self.implementation.get_master_xpub(addrtype, account)?;
        deserialize_obj!(&output)
    }

    /// Signs a PSBT.
    pub fn sign_tx(&self, psbt: &Psbt) -> Result<HWIPartiallySignedTransaction, Error> {
        let output = self.implementation.sign_tx(psbt)?;
        deserialize_obj!(&output)
    }

    /// Returns the xpub of a device. If `expert` is set, additional output is returned.
    pub fn get_xpub(
        &self,
        path: &DerivationPath,
        expert: bool,
    ) -> Result<HWIExtendedPubKey, Error> {
        let prefixed_path = format!("m/{}", path);
        let output = self.implementation.get_xpub(&prefixed_path, expert)?;
        deserialize_obj!(&output)
    }

    /// Signs a message.
    pub fn sign_message(
        &self,
        message: &str,
        path: &DerivationPath,
    ) -> Result<HWISignature, Error> {
        let prefixed_path = format!("m/{}", path);
        let output = self.implementation.sign_message(message, &prefixed_path)?;
        deserialize_obj!(&output)
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
        let path_str = path.map(|p| format!("m/{}/*", p));
        let output = self.implementation.get_keypool(
            keypool,
            internal,
            addr_type,
            addr_all,
            account.unwrap_or(0),
            path_str,
            start,
            end,
        )?;
        deserialize_obj!(&output)
    }

    /// Returns device descriptors. You can optionally specify a BIP43 account to use.
    pub fn get_descriptors<U>(&self, account: Option<u32>) -> Result<HWIDescriptor<U>, Error>
    where
        U: ToDescriptor + DeserializeOwned,
    {
        let output = self.implementation.get_descriptors(account.unwrap_or(0))?;
        deserialize_obj!(&output)
    }

    /// Returns an address given a descriptor.
    pub fn display_address_with_desc<U>(&self, descriptor: &U) -> Result<HWIAddress, Error>
    where
        U: ToDescriptor + ToString,
    {
        let descriptor = descriptor.to_string().split('#').collect::<Vec<_>>()[0].to_string();
        let output = self.implementation.display_address_with_desc(&descriptor)?;
        deserialize_obj!(&output)
    }

    /// Returns an address given path and address type.
    pub fn display_address_with_path(
        &self,
        path: &DerivationPath,
        address_type: HWIAddressType,
    ) -> Result<HWIAddress, Error> {
        let prefixed_path = format!("m/{}", path);
        let output = self
            .implementation
            .display_address_with_path(&prefixed_path, address_type)?;
        deserialize_obj!(&output)
    }

    /// Install the udev rules to the local machine.
    ///
    /// The rules will be copied from the source to the location; the default source location is
    /// `./udev`, the default destination location is `/lib/udev/rules.d`
    pub fn install_udev_rules(source: Option<&str>, location: Option<&str>) -> Result<(), Error> {
        let output = T::install_udev_rules(
            source.unwrap_or("./udev"),
            location.unwrap_or("/lib/udev/rules.d/"),
        )?;
        let status: HWIStatus = deserialize_obj!(&output)?;
        status.into()
    }

    /// Set logging level
    /// # Arguments
    /// * `level` - Log level.
    pub fn set_log_level(level: LogLevel) -> Result<(), Error> {
        T::set_log_level(level)?;
        Ok(())
    }

    /// Toggle whether the device is using a BIP 39 passphrase.
    pub fn toggle_passphrase(&self) -> Result<(), Error> {
        let output = self.implementation.toggle_passphrase()?;
        let status: HWIStatus = deserialize_obj!(&output)?;
        status.into()
    }

    /// Set up the device
    pub fn setup_device(&self, label: Option<&str>, passphrase: Option<&str>) -> Result<(), Error> {
        let output = self
            .implementation
            .setup_device(label.unwrap_or(""), passphrase.unwrap_or(""))?;
        let status: HWIStatus = deserialize_obj!(&output)?;
        status.into()
    }

    /// Restore a device
    pub fn restore_device(
        &self,
        label: Option<&str>,
        word_count: Option<HWIWordCount>,
    ) -> Result<(), Error> {
        let word_count: u8 = word_count.map_or_else(|| 24, |w| w as u8);
        let output = self
            .implementation
            .restore_device(label.unwrap_or(""), word_count)?;
        let status: HWIStatus = deserialize_obj!(&output)?;
        status.into()
    }

    /// Create a backup of the device
    pub fn backup_device(
        &self,
        label: Option<&str>,
        backup_passphrase: Option<&str>,
    ) -> Result<(), Error> {
        let output = self.implementation.backup_device(
            label.unwrap_or_default(),
            backup_passphrase.unwrap_or_default(),
        )?;
        let status: HWIStatus = deserialize_obj!(&output)?;
        status.into()
    }

    /// Wipe a device
    pub fn wipe_device(&self) -> Result<(), Error> {
        let output = self.implementation.wipe_device()?;
        let status: HWIStatus = deserialize_obj!(&output)?;
        status.into()
    }

    /// Get the installed version of hwilib. Returns None if hwi is not installed.
    pub fn get_version() -> Result<String, Error> {
        T::get_version()
    }

    /// Install hwi for the current user via pip. If no version is specified, the default version from pip will be installed.
    pub fn install_hwilib(version: Option<&str>) -> Result<(), Error> {
        let hwi_with_version = match version {
            Some(ver) => "hwi==".to_owned() + ver,
            None => "hwi".to_owned(),
        };
        T::install_hwilib(hwi_with_version)?;
        Ok(())
    }
}
