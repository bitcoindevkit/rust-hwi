use std::ops::Deref;

use bitcoin::util::bip32::ExtendedPubKey;
use bitcoin::util::{address::Address, psbt::PartiallySignedTransaction};

use pyo3::types::PyModule;
use pyo3::{IntoPy, PyObject};
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
pub struct HWIExtendedPubKey {
    pub xpub: ExtendedPubKey,
}

impl Deref for HWIExtendedPubKey {
    type Target = ExtendedPubKey;

    fn deref(&self) -> &Self::Target {
        &self.xpub
    }
}

#[derive(Deserialize)]
pub struct HWISignature {
    #[serde(deserialize_with = "from_b64")]
    pub signature: Vec<u8>,
}

fn from_b64<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    let b64_string = String::deserialize(d)?;
    base64::decode(&b64_string)
        .map_err(|_| serde::de::Error::custom("Error while Deserializing Signature"))
}

impl Deref for HWISignature {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.signature
    }
}

#[derive(Deserialize)]
pub struct HWIAddress {
    pub address: Address,
}

#[derive(Deserialize)]
pub struct HWIPartiallySignedTransaction {
    pub psbt: PartiallySignedTransaction,
}

impl Deref for HWIPartiallySignedTransaction {
    type Target = PartiallySignedTransaction;

    fn deref(&self) -> &Self::Target {
        &self.psbt
    }
}

// TODO: use Descriptors
#[derive(Deserialize, Debug)]
pub struct HWIDescriptor {
    pub internal: Vec<String>,
    pub receive: Vec<String>,
}

#[derive(Deserialize)]
pub struct HWIKeyPoolElement {
    pub desc: String,
    pub range: Vec<u32>,
    pub timestamp: String,
    pub internal: bool,
    pub keypool: bool,
    pub watchonly: bool,
}

#[derive(Clone, Debug)]
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

#[derive(Debug)]
pub enum HWIChain {
    Main,
    Test,
    Regtest,
    Signet,
}
impl IntoPy<PyObject> for HWIChain {
    fn into_py(self, py: pyo3::Python) -> PyObject {
        let chain = PyModule::import(py, "hwilib.common")
            .unwrap()
            .getattr("Chain")
            .unwrap();
        match self {
            HWIChain::Main => chain.get_item("MAIN").unwrap().into(),
            HWIChain::Test => chain.get_item("TEST").unwrap().into(),
            HWIChain::Regtest => chain.get_item("REGTEST").unwrap().into(),
            HWIChain::Signet => chain.get_item("SIGNET").unwrap().into(),
        }
    }
}
