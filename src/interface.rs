use std::collections::HashMap;
use std::str::from_utf8;

use bitcoin::util::bip32::{DerivationPath, ExtendedPubKey};

use crate::commands::{HWICommand, HWIFlag, HWISubcommand};
use crate::types::{HWIDescriptor, HWIDevice};

pub fn enumerate() -> Vec<HWIDevice> {
    let output = HWICommand::new()
        .add_subcommand(HWISubcommand::Enumerate)
        .execute();
    serde_json::from_str(from_utf8(&output.stdout).unwrap()).unwrap()
}

pub fn getmasterxpub(device: &HWIDevice) -> HashMap<String, ExtendedPubKey> {
    let output = HWICommand::new()
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::GetMasterXpub)
        .execute();
    serde_json::from_str(from_utf8(&output.stdout).unwrap()).unwrap()
}

pub fn getxpub(device: &HWIDevice, path: &DerivationPath) -> HashMap<String, ExtendedPubKey> {
    let output = HWICommand::new()
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::GetXpub)
        .add_path(path)
        .execute();
    serde_json::from_str(from_utf8(&output.stdout).unwrap()).unwrap()
}

pub fn getdescriptors(device: &HWIDevice) -> HWIDescriptor {
    let output = HWICommand::new()
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::GetDescriptors)
        .execute();
    serde_json::from_str(from_utf8(&output.stdout).unwrap()).unwrap()
}
