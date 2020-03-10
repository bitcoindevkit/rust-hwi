use bitcoin::util::address::Address;
use bitcoin::util::bip32::{ExtendedPubKey, Fingerprint};

use serde::{Deserialize};

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

#[derive(Deserialize)]
pub struct HWIExtendedPubKey {
    pub xpub: ExtendedPubKey,
}

// TODO: is signature a String?
#[derive(Deserialize)]
pub struct HWISignature {
    pub signature: String,
}

#[derive(Deserialize)]
pub struct HWIAddress {
    pub address: Address,
}

#[derive(Deserialize)]
pub struct HWIPartiallySignedTransaction {
    pub psbt: String,
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

pub enum HWIAddressType {
    Pkh,
    ShWpkh,
    Wpkh,
}
