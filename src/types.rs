use std::ops::Deref;

use bitcoin::consensus::encode::deserialize;
use bitcoin::util::{address::Address, psbt::PartiallySignedTransaction};
use bitcoin::util::bip32::ExtendedPubKey;

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
    #[serde(deserialize_with="from_b64")]
    pub signature: Vec<u8>,
}

fn from_b64<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    let b64_string = String::deserialize(d)?;
    base64::decode(&b64_string).map_err(|_| serde::de::Error::custom("Error while Deserializing Signature"))
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
    #[serde(deserialize_with="deserialize_psbt")]
    pub psbt: PartiallySignedTransaction,
}

fn deserialize_psbt<'de, D: Deserializer<'de>>(d: D) -> Result<PartiallySignedTransaction, D::Error> {
    let b64_string = String::deserialize(d)?;
    let bytes = base64::decode(&b64_string).map_err(|e| serde::de::Error::custom(e))?;
    deserialize::<PartiallySignedTransaction>(&bytes[..]).map_err(|e| serde::de::Error::custom(e))
}

impl Deref for HWIPartiallySignedTransaction {
    type Target = PartiallySignedTransaction;

    fn deref(&self) -> &Self::Target {
        &self.psbt
    }
}

// TODO: use Descriptors
#[derive(Deserialize)]
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

#[derive(Debug)]
pub enum HWIChain {
    Main,
    Test,
    RegTest,
    SigNet,
}
