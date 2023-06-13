use std::convert::TryFrom;
use std::ops::Deref;
use std::str::FromStr;

use bitcoin::address::{Address, NetworkUnchecked};
use bitcoin::base64;
use bitcoin::bip32::{ExtendedPubKey, Fingerprint};
use bitcoin::psbt::PartiallySignedTransaction;
use bitcoin::Network;

use pyo3::types::PyModule;
use pyo3::{IntoPy, PyObject};
use serde::{Deserialize, Deserializer};

#[cfg(feature = "miniscript")]
use miniscript::{Descriptor, DescriptorPublicKey};

use crate::error::{Error, ErrorCode};

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
pub struct HWIExtendedPubKey {
    pub xpub: ExtendedPubKey,
}

impl Deref for HWIExtendedPubKey {
    type Target = ExtendedPubKey;

    fn deref(&self) -> &Self::Target {
        &self.xpub
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
pub struct HWISignature {
    #[serde(deserialize_with = "from_b64")]
    pub signature: Vec<u8>,
}

fn from_b64<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    let b64_string = String::deserialize(d)?;
    base64::decode(b64_string)
        .map_err(|_| serde::de::Error::custom("error while deserializing signature"))
}

impl Deref for HWISignature {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.signature
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
pub struct HWIAddress {
    pub address: Address<NetworkUnchecked>,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
pub struct HWIPartiallySignedTransaction {
    #[serde(deserialize_with = "deserialize_psbt")]
    pub psbt: PartiallySignedTransaction,
}

fn deserialize_psbt<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<PartiallySignedTransaction, D::Error> {
    let s = String::deserialize(d)?;
    PartiallySignedTransaction::from_str(&s).map_err(serde::de::Error::custom)
}

impl Deref for HWIPartiallySignedTransaction {
    type Target = PartiallySignedTransaction;

    fn deref(&self) -> &Self::Target {
        &self.psbt
    }
}

pub trait ToDescriptor {}
impl ToDescriptor for String {}
#[cfg(feature = "miniscript")]
impl ToDescriptor for Descriptor<DescriptorPublicKey> {}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
pub struct HWIDescriptor<T>
where
    T: ToDescriptor,
{
    pub internal: Vec<T>,
    pub receive: Vec<T>,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
pub struct HWIKeyPoolElement {
    pub desc: String,
    pub range: Vec<u32>,
    pub timestamp: String,
    pub internal: bool,
    pub keypool: bool,
    pub watchonly: bool,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
#[allow(non_camel_case_types)]
pub enum HWIAddressType {
    Legacy,
    Sh_Wit,
    Wit,
    Tap,
}

impl IntoPy<PyObject> for HWIAddressType {
    fn into_py(self, py: pyo3::Python) -> PyObject {
        let addrtype = PyModule::import(py, "hwilib.common")
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

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
pub struct HWIChain(bitcoin::Network);

impl IntoPy<PyObject> for HWIChain {
    fn into_py(self, py: pyo3::Python) -> PyObject {
        use bitcoin::Network::*;

        let chain = PyModule::import(py, "hwilib.common")
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
#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
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

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
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

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
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

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
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

impl ToString for HWIDeviceType {
    fn to_string(&self) -> String {
        match self {
            Self::Ledger => String::from("ledger"),
            Self::Trezor => String::from("trezor"),
            Self::BitBox01 => String::from("digitalbitbox"),
            Self::BitBox02 => String::from("bitbox02"),
            Self::KeepKey => String::from("keepkey"),
            Self::Coldcard => String::from("coldcard"),
            Self::Jade => String::from("jade"),
            Self::Other(name) => name.to_string(),
        }
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
