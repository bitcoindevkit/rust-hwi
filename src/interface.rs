use std::str::from_utf8;

use bitcoin::util::bip32::DerivationPath;

use crate::commands::{HWICommand, HWIFlag, HWISubcommand};
use crate::error::Error;
use crate::types::{HWIDescriptor, HWIDevice, HWIExtendedPubKey, HWISignature};

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
    message: &str,
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

pub fn get_descriptors(device: &HWIDevice) -> Result<HWIDescriptor, Error> {
    let output = HWICommand::new()
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::GetDescriptors)
        .execute()?;
    Ok(serde_json::from_str(from_utf8(&output.stdout)?)?)
}
