use std::str::from_utf8;

use bitcoin::util::bip32::DerivationPath;

use crate::commands::{HWICommand, HWIFlag, HWISubcommand};
use crate::error::Error;
use crate::types::{HWIAddress, HWIDescriptor, HWIDevice, HWIExtendedPubKey, HWISignature};

pub fn enumerate() -> Result<Vec<HWIDevice>, Error> {
    let output = HWICommand::new()
        .add_subcommand(HWISubcommand::Enumerate)
        .execute()?;
    Ok(serde_json::from_str(from_utf8(&output.stdout)?)?)
}

pub fn get_master_xpub(device: &HWIDevice) -> Result<HWIExtendedPubKey, Error> {
    let output = HWICommand::new()
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::GetMasterXpub)
        .execute()?;
    Ok(serde_json::from_str(from_utf8(&output.stdout)?)?)
}

pub fn sign_tx(device: &HWIDevice) {}

pub fn get_xpub(device: &HWIDevice, path: &DerivationPath) -> Result<HWIExtendedPubKey, Error> {
    let output = HWICommand::new()
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::GetXpub)
        .add_path(path)
        .execute()?;
    Ok(serde_json::from_str(from_utf8(&output.stdout)?)?)
}

pub fn sign_message(
    device: &HWIDevice,
    message: &String,
    path: &DerivationPath,
) -> Result<HWISignature, Error> {
    let output = HWICommand::new()
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::SignMessage)
        .add_message(message)
        .add_path(path)
        .execute()?;
    Ok(serde_json::from_str(from_utf8(&output.stdout)?)?)
}

pub fn get_keypool(device: &HWIDevice) {}

pub fn get_descriptors(device: &HWIDevice, account: Option<u32>) -> Result<HWIDescriptor, Error> {
    let account = match account {
        Some(i) => i,
        None => 0
    };
    let output = HWICommand::new()
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::GetDescriptors)
        .add_account(account)
        .execute()?;
    Ok(serde_json::from_str(from_utf8(&output.stdout)?)?)
}

pub fn display_address(device: &HWIDevice, descriptor: &String) -> Result<HWIAddress, Error> {
    let output = HWICommand::new()
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::DisplayAddress)
        .add_descriptor(descriptor)
        .execute()?;
    Ok(serde_json::from_str(from_utf8(&output.stdout)?)?)
}
