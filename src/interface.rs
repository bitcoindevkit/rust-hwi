use std::collections::HashMap;
use std::str::from_utf8;

use bitcoin::util::bip32::{DerivationPath, ExtendedPubKey};

use crate::types::HWIDevice;
use crate::commands::{HWICommand, HWIFlag, HWISubcommand};

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
    println!("{:?}", from_utf8(&output.stdout));
    serde_json::from_str(from_utf8(&output.stdout).unwrap()).unwrap()
}
