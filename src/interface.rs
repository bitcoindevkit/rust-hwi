use std::str::from_utf8;

use bitcoin::consensus::encode::serialize;
use bitcoin::util::bip32::DerivationPath;
use bitcoin::util::psbt::PartiallySignedTransaction;

use base64;

use crate::commands::{HWICommand, HWIFlag, HWISubcommand};
use crate::error::Error;
use crate::types::{
    HWIAddress, HWIAddressType, HWIDescriptor, HWIDevice, HWIExtendedPubKey, HWIKeyPoolElement,
    HWIPartiallySignedTransaction, HWISignature,
};

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

pub fn sign_tx(
    device: &HWIDevice,
    psbt: &PartiallySignedTransaction,
) -> Result<HWIPartiallySignedTransaction, Error> {
    let psbt = base64::encode(&serialize(psbt));
    let output = HWICommand::new()
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::SignTx)
        .add_psbt(&psbt)
        .execute()?;
    Ok(serde_json::from_str(from_utf8(&output.stdout)?)?)
}

pub fn get_xpub(device: &HWIDevice, path: &DerivationPath) -> Result<HWIExtendedPubKey, Error> {
    let output = HWICommand::new()
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::GetXpub)
        .add_path(&path, &false, &false)
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
        .add_message(&message)
        .add_path(&path, &false, &false)
        .execute()?;
    Ok(serde_json::from_str(from_utf8(&output.stdout)?)?)
}

pub fn get_keypool(
    device: &HWIDevice,
    keypool: &bool,
    internal: &bool,
    address_type: &HWIAddressType,
    account: Option<&u32>,
    path: Option<&DerivationPath>,
    start: &u32,
    end: &u32,
) -> Result<Vec<HWIKeyPoolElement>, Error> {
    let mut command = HWICommand::new();
    command
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::GetKeypool)
        .add_keypool(&keypool)
        .add_internal(&internal)
        .add_address_type(&address_type);

    if let Some(a) = account {
        command.add_account(&a);
    }

    if let Some(p) = path {
        command.add_path(&p, &true, &true);
    }

    let output = command.add_start_end(&start, &end).execute()?;
    Ok(serde_json::from_str(from_utf8(&output.stdout)?)?)
}

pub fn get_descriptors(device: &HWIDevice, account: Option<&u32>) -> Result<HWIDescriptor, Error> {
    let mut command = HWICommand::new();
    command
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::GetDescriptors);

    if let Some(a) = account {
        command.add_account(&a);
    };

    let output = command.execute()?;
    Ok(serde_json::from_str(from_utf8(&output.stdout)?)?)
}

pub fn display_address_with_desc(
    device: &HWIDevice,
    descriptor: &String,
) -> Result<HWIAddress, Error> {
    let output = HWICommand::new()
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::DisplayAddress)
        .add_descriptor(&descriptor)
        .execute()?;
    Ok(serde_json::from_str(from_utf8(&output.stdout)?)?)
}

pub fn display_address_with_path(
    device: &HWIDevice,
    path: &DerivationPath,
    address_type: &HWIAddressType,
) -> Result<HWIAddress, Error> {
    let output = HWICommand::new()
        .add_flag(HWIFlag::Fingerprint(device.fingerprint))
        .add_subcommand(HWISubcommand::DisplayAddress)
        .add_path(&path, &true, &false)
        .add_address_type(&address_type)
        .execute()?;
    Ok(serde_json::from_str(from_utf8(&output.stdout)?)?)
}
