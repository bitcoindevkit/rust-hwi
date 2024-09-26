use core::fmt;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

use bitcoin::address::{Address, NetworkUnchecked};
use bitcoin::bip32::{Fingerprint, Xpub};
use bitcoin::Network;
use bitcoin::Psbt;

use pyo3::types::PyModule;
use pyo3::{IntoPy, PyObject};
use serde::{Deserialize, Deserializer, Serialize};

#[cfg(feature = "miniscript")]
use miniscript::{Descriptor, DescriptorPublicKey};
use pyo3::prelude::PyAnyMethods;

use crate::error::{Error, ErrorCode};

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct HWIExtendedPubKey {
    pub xpub: Xpub,
}

impl Deref for HWIExtendedPubKey {
    type Target = Xpub;

    fn deref(&self) -> &Self::Target {
        &self.xpub
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct HWISignature {
    #[serde(deserialize_with = "from_b64")]
    pub signature: Vec<u8>,
}

fn from_b64<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    use bitcoin::base64::{engine::general_purpose, Engine as _};

    let b64_string = String::deserialize(d)?;
    general_purpose::STANDARD
        .decode(b64_string)
        .map_err(|_| serde::de::Error::custom("error while deserializing signature"))
}

impl Deref for HWISignature {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.signature
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct HWIAddress {
    pub address: Address<NetworkUnchecked>,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct HWIPartiallySignedTransaction {
    #[serde(deserialize_with = "deserialize_psbt")]
    pub psbt: Psbt,
}

fn deserialize_psbt<'de, D: Deserializer<'de>>(d: D) -> Result<Psbt, D::Error> {
    let s = String::deserialize(d)?;
    Psbt::from_str(&s).map_err(serde::de::Error::custom)
}

impl Deref for HWIPartiallySignedTransaction {
    type Target = Psbt;

    fn deref(&self) -> &Self::Target {
        &self.psbt
    }
}

pub trait ToDescriptor {}
impl ToDescriptor for String {}
#[cfg(feature = "miniscript")]
impl ToDescriptor for Descriptor<DescriptorPublicKey> {}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct HWIDescriptor<T>
where
    T: ToDescriptor,
{
    pub internal: Vec<T>,
    pub receive: Vec<T>,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct HWIKeyPoolElement {
    pub desc: String,
    pub range: Vec<u32>,
    pub timestamp: String,
    pub internal: bool,
    pub keypool: bool,
    pub watchonly: bool,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
#[allow(non_camel_case_types)]
pub enum HWIAddressType {
    Legacy,
    Sh_Wit,
    Wit,
    Tap,
}

impl fmt::Display for HWIAddressType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            HWIAddressType::Legacy => "LEGACY",
            HWIAddressType::Sh_Wit => "SH_WIT",
            HWIAddressType::Wit => "WIT",
            HWIAddressType::Tap => "TAP",
        })
    }
}

impl IntoPy<PyObject> for HWIAddressType {
    fn into_py(self, py: pyo3::Python) -> PyObject {
        let addrtype = PyModule::import_bound(py, "hwilib.common")
            .unwrap()
            .getattr("AddressType")
            .unwrap();
        match self {
            HWIAddressType::Legacy => addrtype.get_item("LEGACY").unwrap().into(),
            HWIAddressType::Sh_Wit => addrtype.get_item("SH_WIT").unwrap().into(),
            HWIAddressType::Wit => addrtype.get_item("WIT").unwrap().into(),
            HWIAddressType::Tap => addrtype.get_item("TAP").unwrap().into(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct HWIChain(bitcoin::Network);

impl fmt::Display for HWIChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self.0 {
            bitcoin::Network::Bitcoin => "MAIN",
            bitcoin::Network::Testnet => "TEST",
            bitcoin::Network::Regtest => "REGTEST",
            bitcoin::Network::Signet => "SIGNET",
            _ => "UNKNOWN",
        })
    }
}

impl IntoPy<PyObject> for HWIChain {
    fn into_py(self, py: pyo3::Python) -> PyObject {
        use bitcoin::Network::*;

        let chain = PyModule::import_bound(py, "hwilib.common")
            .unwrap()
            .getattr("Chain")
            .unwrap();
        match self.0 {
            Bitcoin => chain.get_item("MAIN").unwrap().into(),
            Testnet => chain.get_item("TEST").unwrap().into(),
            Regtest => chain.get_item("REGTEST").unwrap().into(),
            Signet => chain.get_item("SIGNET").unwrap().into(),
            // This handles non_exhaustive on Network which is only there to future proof
            // rust-bitcoin, will need to check this when upgrading rust-bitcoin.
            // Sane as of rust-bitcoin v0.30.0
            _ => panic!("unknown network"),
        }
    }
}

impl From<Network> for HWIChain {
    fn from(network: Network) -> Self {
        Self(network)
    }
}

#[cfg(test)]
pub const TESTNET: HWIChain = HWIChain(Network::Testnet);

// Used internally to deserialize the result of `hwi enumerate`. This might
// contain an `error`, when it does, it might not contain all the fields `HWIDevice`
// is supposed to have - for this reason, they're all Option.
#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub(crate) struct HWIDeviceInternal {
    #[serde(rename(deserialize = "type"))]
    pub device_type: Option<String>,
    pub model: Option<String>,
    pub path: Option<String>,
    pub needs_pin_sent: Option<bool>,
    pub needs_passphrase_sent: Option<bool>,
    pub fingerprint: Option<Fingerprint>,
    pub error: Option<String>,
    pub code: Option<i8>,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct HWIDevice {
    #[serde(rename(deserialize = "type"))]
    pub device_type: HWIDeviceType,
    pub model: String,
    pub path: String,
    pub needs_pin_sent: bool,
    pub needs_passphrase_sent: bool,
    pub fingerprint: Fingerprint,
}

impl TryFrom<HWIDeviceInternal> for HWIDevice {
    type Error = Error;
    fn try_from(h: HWIDeviceInternal) -> Result<HWIDevice, Error> {
        match h.error {
            Some(e) => {
                let code = h.code.and_then(|c| ErrorCode::try_from(c).ok());
                Err(Error::Hwi(e, code))
            }
            // When HWIDeviceInternal contains errors, some fields might be missing
            // (depending on the error, hwi might not be able to know all of them).
            // When there's no error though, all the fields must be present, and
            // for this reason we expect here.
            None => Ok(HWIDevice {
                device_type: HWIDeviceType::from(
                    h.device_type.expect("Device type should be here"),
                ),
                model: h.model.expect("Model should be here"),
                path: h.path.expect("Path should be here"),
                needs_pin_sent: h.needs_pin_sent.expect("needs_pin_sent should be here"),
                needs_passphrase_sent: h
                    .needs_passphrase_sent
                    .expect("needs_passphrase_sent should be here"),
                fingerprint: h.fingerprint.expect("Fingerprint should be here"),
            }),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct HWIStatus {
    pub success: bool,
}

impl From<HWIStatus> for Result<(), Error> {
    fn from(s: HWIStatus) -> Self {
        if s.success {
            Ok(())
        } else {
            Err(Error::Hwi(
                "request returned with failure".to_string(),
                None,
            ))
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HWIDeviceType {
    Ledger,
    Trezor,
    BitBox01,
    BitBox02,
    KeepKey,
    Coldcard,
    Jade,
    Other(String),
}

impl<T> From<T> for HWIDeviceType
where
    T: AsRef<str>,
{
    fn from(s: T) -> Self {
        match s.as_ref() {
            "ledger" => Self::Ledger,
            "trezor" => Self::Trezor,
            "digitalbitbox" => Self::BitBox01,
            "bitbox02" => Self::BitBox02,
            "keepkey" => Self::KeepKey,
            "coldcard" => Self::Coldcard,
            "jade" => Self::Jade,
            name => Self::Other(name.to_string()),
        }
    }
}

impl Display for HWIDeviceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Ledger => String::from("ledger"),
            Self::Trezor => String::from("trezor"),
            Self::BitBox01 => String::from("digitalbitbox"),
            Self::BitBox02 => String::from("bitbox02"),
            Self::KeepKey => String::from("keepkey"),
            Self::Coldcard => String::from("coldcard"),
            Self::Jade => String::from("jade"),
            Self::Other(name) => name.to_string(),
        };
        fmt::Display::fmt(&name, f)
    }
}

pub enum LogLevel {
    DEBUG,
    INFO,
    WARNING,
    ERROR,
    CRITICAL,
}

#[derive(Clone, Eq, PartialEq, Debug, Copy)]
#[repr(u8)]
/// The number of words in the recovery phrase
pub enum HWIWordCount {
    W12 = 12,
    W18 = 18,
    W24 = 24,
}

pub trait HWIImplementation: Debug + Send + Sync {
    fn enumerate() -> Result<String, Error>;
    fn get_client(device: &HWIDevice, expert: bool, chain: HWIChain) -> Result<Self, Error>
    where
        Self: Sized;
    fn find_device(
        password: Option<&str>,
        device_type: Option<HWIDeviceType>,
        fingerprint: Option<&str>,
        expert: bool,
        chain: HWIChain,
    ) -> Result<Self, Error>
    where
        Self: Sized;
    fn get_xpub(&self, path: &str, expert: bool) -> Result<String, Error>;
    fn sign_tx(&self, psbt: &Psbt) -> Result<String, Error>;
    fn get_master_xpub(&self, addrtype: HWIAddressType, account: u32) -> Result<String, Error>;
    fn sign_message(&self, message: &str, path: &str) -> Result<String, Error>;
    fn display_address_with_desc(&self, descriptor: &str) -> Result<String, Error>;
    fn display_address_with_path(
        &self,
        path: &str,
        address_type: HWIAddressType,
    ) -> Result<String, Error>;
    fn toggle_passphrase(&self) -> Result<String, Error>;
    fn setup_device(&self, label: &str, passphrase: &str) -> Result<String, Error>;
    fn restore_device(&self, label: &str, word_count: u8) -> Result<String, Error>;
    fn backup_device(&self, label: &str, backup_passphrase: &str) -> Result<String, Error>;
    fn wipe_device(&self) -> Result<String, Error>;
    fn get_descriptors(&self, account: u32) -> Result<String, Error>;
    #[allow(clippy::too_many_arguments)]
    fn get_keypool(
        &self,
        keypool: bool,
        internal: bool,
        addr_type: HWIAddressType,
        addr_all: bool,
        account: u32,
        path: Option<String>,
        start: u32,
        end: u32,
    ) -> Result<String, Error>;
    fn get_version() -> Result<String, Error>;
    fn install_udev_rules(source: &str, location: &str) -> Result<String, Error>;
    fn set_log_level(level: LogLevel) -> Result<(), Error>;
    fn install_hwilib(version: String) -> Result<(), Error>;
}

#[derive(Clone, Eq, PartialEq, Debug, Copy)]
pub struct HWIClient<T: HWIImplementation> {
    pub implementation: T,
}

pub trait HWIBinaryExecutor: Debug + Send + Sync {
    fn execute_command(args: Vec<String>) -> Result<String, Error>;
}
