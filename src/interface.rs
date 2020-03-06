use std::str::from_utf8;

use bitcoin::util::bip32::DerivationPath;

use crate::commands::{HWICommand, HWIFlag, HWISubcommand};
use crate::types::{HWIDescriptor, HWIDevice, HWIExtendedPubKey, HWISignature};

pub fn enumerate() -> Vec<HWIDevice> {
    let output = HWICommand::new()
        .add_subcommand(HWISubcommand::Enumerate)
        .execute();
    serde_json::from_str(from_utf8(&output.stdout).unwrap()).unwrap()
}

pub fn getmasterxpub(device: &HWIDevice) -> HWIExtendedPubKey {
    let output = HWICommand::new()
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::GetMasterXpub)
        .execute();
    serde_json::from_str(from_utf8(&output.stdout).unwrap()).unwrap()
}

pub fn getxpub(device: &HWIDevice, path: &DerivationPath) -> HWIExtendedPubKey {
    let output = HWICommand::new()
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::GetXpub)
        .add_path(path)
        .execute();
    serde_json::from_str(from_utf8(&output.stdout).unwrap()).unwrap()
}

pub fn signmessage(device: &HWIDevice, message: &str, path: &DerivationPath) -> HWISignature {
    let output = HWICommand::new()
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::SignMessage)
        .add_message(message)
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
