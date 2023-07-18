//! Rust wrapper for the [Bitcoin Hardware Wallet Interface](https://github.com/bitcoin-core/HWI/).
//!
//! # Example - display address with path
//! ```no_run
//! use bitcoin::bip32::{ChildNumber, DerivationPath};
//! use hwi::error::Error;
//! use hwi::interface::HWIClient;
//! use hwi::types;
//! use std::str::FromStr;
//!
//! fn main() -> Result<(), Error> {
//!     // Find information about devices
//!     let mut devices = HWIClient::enumerate()?;
//!     if devices.is_empty() {
//!         panic!("No device found!");
//!     }
//!     let device = devices.remove(0)?;
//!     // Create a client for a device
//!     let client = HWIClient::get_client(&device, true, bitcoin::Network::Testnet.into())?;
//!     // Display the address from path
//!     let derivation_path = DerivationPath::from_str("m/44'/1'/0'/0/0").unwrap();
//!     let hwi_address =
//!         client.display_address_with_path(&derivation_path, types::HWIAddressType::Tap)?;
//!     println!("{}", hwi_address.address.assume_checked());
//!     Ok(())
//! }
//! ```

#[cfg(test)]
#[macro_use]
extern crate serial_test;

pub use interface::HWIClient;

#[cfg(feature = "doctest")]
pub mod doctest;
pub mod error;
pub mod interface;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::types::{self, HWIDeviceType, TESTNET};
    use crate::HWIClient;
    use std::collections::BTreeMap;
    use std::str::FromStr;

    use bitcoin::bip32::{DerivationPath, KeySource};
    use bitcoin::locktime::absolute;
    use bitcoin::psbt::{Input, Output};
    use bitcoin::{secp256k1, Transaction};
    use bitcoin::{Network, TxIn, TxOut};

    #[cfg(feature = "miniscript")]
    use miniscript::{Descriptor, DescriptorPublicKey};

    #[test]
    #[serial]
    fn test_enumerate() {
        let devices = HWIClient::enumerate().unwrap();
        assert!(devices.len() > 0);
    }

    #[test]
    #[serial]
    #[ignore]
    fn test_find_trezor_device() {
        HWIClient::find_device(
            None,
            Some(HWIDeviceType::Trezor),
            None,
            false,
            Network::Testnet,
        )
        .unwrap();
    }

    fn get_first_device() -> HWIClient {
        let devices = HWIClient::enumerate().unwrap();
        let device = devices
            .first()
            .expect("No devices found. Either plug in a hardware wallet, or start a simulator.")
            .as_ref()
            .expect("Error when opening the first device");
        HWIClient::get_client(&device, true, TESTNET).unwrap()
    }

    #[test]
    #[serial]
    fn test_get_master_xpub() {
        let client = get_first_device();
        client
            .get_master_xpub(types::HWIAddressType::Wit, 0)
            .unwrap();
    }

    #[test]
    #[serial]
    fn test_get_xpub() {
        let client = get_first_device();
        let derivation_path = DerivationPath::from_str("m/44'/1'/0'/0/0").unwrap();
        client.get_xpub(&derivation_path, false).unwrap();
    }

    #[test]
    #[serial]
    fn test_sign_message() {
        let client = get_first_device();
        let derivation_path = DerivationPath::from_str("m/44'/1'/0'/0/0").unwrap();
        client
            .sign_message("I love BDK wallet", &derivation_path)
            .unwrap();
    }

    #[test]
    #[serial]
    fn test_get_string_descriptors() {
        let client = get_first_device();
        let account = Some(10);
        let descriptor = client.get_descriptors::<String>(account).unwrap();
        assert!(descriptor.internal.len() > 0);
        assert!(descriptor.receive.len() > 0);
    }

    #[test]
    #[serial]
    fn test_display_address_with_string_desc() {
        let client = get_first_device();
        let descriptor = client.get_descriptors::<String>(None).unwrap();
        let descriptor = descriptor.receive.first().unwrap();
        client.display_address_with_desc(descriptor).unwrap();
    }

    #[test]
    #[serial]
    #[cfg(feature = "miniscript")]
    fn test_get_miniscript_descriptors() {
        let client = get_first_device();
        let account = Some(10);
        let descriptor = client
            .get_descriptors::<Descriptor<DescriptorPublicKey>>(account)
            .unwrap();
        assert!(descriptor.internal.len() > 0);
        assert!(descriptor.receive.len() > 0);
    }

    #[test]
    #[serial]
    #[cfg(feature = "miniscript")]
    fn test_display_address_with_miniscript_desc() {
        let client = get_first_device();
        let descriptor = client
            .get_descriptors::<Descriptor<DescriptorPublicKey>>(None)
            .unwrap();
        let descriptor = descriptor.receive.first().unwrap();
        client.display_address_with_desc(descriptor).unwrap();
    }

    #[test]
    #[serial]
    fn test_display_address_with_path_legacy() {
        let client = get_first_device();
        let derivation_path = DerivationPath::from_str("m/44'/1'/0'/0/0").unwrap();
        client
            .display_address_with_path(&derivation_path, types::HWIAddressType::Legacy)
            .unwrap();
    }

    #[test]
    #[serial]
    fn test_display_address_with_path_nested_segwit() {
        let client = get_first_device();
        let derivation_path = DerivationPath::from_str("m/49'/1'/0'/0/0").unwrap();

        client
            .display_address_with_path(&derivation_path, types::HWIAddressType::Sh_Wit)
            .unwrap();
    }

    #[test]
    #[serial]
    fn test_display_address_with_path_native_segwit() {
        let client = get_first_device();
        let derivation_path = DerivationPath::from_str("m/84'/1'/0'/0/0").unwrap();

        client
            .display_address_with_path(&derivation_path, types::HWIAddressType::Wit)
            .unwrap();
    }

    // TODO: HWI 2.0.2 doesn't support displayaddress with taproot
    // #[test]
    // #[serial]
    // fn test_display_address_with_path_taproot() {}

    #[test]
    #[serial]
    fn test_sign_tx() {
        let devices = HWIClient::enumerate().unwrap();
        let device = devices.first().unwrap().as_ref().unwrap();
        let client = HWIClient::get_client(&device, true, TESTNET).unwrap();
        let derivation_path = DerivationPath::from_str("m/44'/1'/0'/0/0").unwrap();

        let address = client
            .display_address_with_path(&derivation_path, types::HWIAddressType::Legacy)
            .unwrap();

        let pk = client.get_xpub(&derivation_path, true).unwrap();
        let mut hd_keypaths: BTreeMap<secp256k1::PublicKey, KeySource> = Default::default();
        // Here device fingerprint is same as master xpub fingerprint
        hd_keypaths.insert(pk.public_key, (device.fingerprint, derivation_path));

        let script_pubkey = address.address.assume_checked().script_pubkey();

        let previous_tx = Transaction {
            version: 1,
            lock_time: absolute::LockTime::from_consensus(0),
            input: vec![TxIn::default()],
            output: vec![TxOut {
                value: 100,
                script_pubkey: script_pubkey.clone(),
            }],
        };

        let previous_txin = TxIn {
            previous_output: bitcoin::OutPoint {
                txid: previous_tx.txid(),
                vout: Default::default(),
            },
            ..Default::default()
        };
        let psbt = bitcoin::psbt::PartiallySignedTransaction {
            unsigned_tx: Transaction {
                version: 1,
                lock_time: absolute::LockTime::from_consensus(0),
                input: vec![previous_txin],
                output: vec![TxOut {
                    value: 50,
                    script_pubkey: script_pubkey,
                }],
            },
            xpub: Default::default(),
            version: Default::default(),
            proprietary: Default::default(),
            unknown: Default::default(),

            inputs: vec![Input {
                non_witness_utxo: Some(previous_tx),
                witness_utxo: None,
                bip32_derivation: hd_keypaths,
                ..Default::default()
            }],
            outputs: vec![Output::default()],
        };
        let client = get_first_device();
        client.sign_tx(&psbt).unwrap();
    }

    #[test]
    #[serial]
    fn test_get_keypool() {
        let client = get_first_device();
        let keypool = true;
        let internal = false;
        let address_type = types::HWIAddressType::Legacy;
        let account = Some(8);
        let derivation_path = DerivationPath::from_str("m/44'/1'/0'").unwrap();
        let start = 1;
        let end = 5;
        client
            .get_keypool(
                keypool,
                internal,
                address_type,
                false,
                account,
                Some(&derivation_path),
                start,
                end,
            )
            .unwrap();

        let keypool = true;
        let internal = true;
        let address_type = types::HWIAddressType::Wit;
        let account = None;
        let start = 1;
        let end = 8;
        client
            .get_keypool(
                keypool,
                internal,
                address_type,
                false,
                account,
                None,
                start,
                end,
            )
            .unwrap();

        let keypool = false;
        let internal = true;
        let address_type = types::HWIAddressType::Sh_Wit;
        let account = Some(1);
        let start = 0;
        let end = 10;
        client
            .get_keypool(
                keypool,
                internal,
                address_type,
                false,
                account,
                Some(&derivation_path),
                start,
                end,
            )
            .unwrap();
    }

    #[test]
    #[serial]
    #[ignore]
    fn test_install_udev_rules() {
        if cfg!(target_os = "linux") {
            HWIClient::install_udev_rules(None, None).unwrap()
        }
    }

    #[test]
    #[serial]
    fn test_set_log_level() {
        HWIClient::set_log_level(types::LogLevel::DEBUG).unwrap();
        test_enumerate();
    }

    #[test]
    #[serial]
    fn test_toggle_passphrase() {
        let devices = HWIClient::enumerate().unwrap();
        let unsupported = [
            HWIDeviceType::Ledger,
            HWIDeviceType::BitBox01,
            HWIDeviceType::Coldcard,
            HWIDeviceType::Jade,
        ];
        for device in devices {
            let device = device.unwrap();
            if unsupported.contains(&device.device_type) {
                // These devices don't support togglepassphrase
                continue;
            }
            let client = HWIClient::get_client(&device, true, TESTNET).unwrap();
            client.toggle_passphrase().unwrap();
            break;
        }
    }

    #[test]
    #[serial]
    fn test_get_version() {
        HWIClient::get_version().unwrap();
    }

    #[test]
    #[serial]
    #[ignore]
    // At the moment (hwi v2.1.1 and trezor-firmware core v2.5.2) work only with physical devices and NOT emulators!
    fn test_setup_trezor_device() {
        let client = HWIClient::find_device(
            None,
            Some(HWIDeviceType::Trezor),
            None,
            false,
            Network::Testnet,
        )
        .unwrap();
        client.setup_device(Some("My Label"), None).unwrap();
    }

    #[test]
    #[serial]
    #[ignore]
    // At the moment (hwi v2.1.1 and trezor-firmware core v2.5.2) work only with physical devices and NOT emulators!
    fn test_restore_trezor_device() {
        let client = HWIClient::find_device(
            None,
            Some(HWIDeviceType::Trezor),
            None,
            false,
            Network::Testnet,
        )
        .unwrap();
        client.restore_device(Some("My Label"), None).unwrap();
    }

    #[test]
    #[serial]
    fn test_backup_device() {
        let devices = HWIClient::enumerate().unwrap();
        let supported = [
            HWIDeviceType::BitBox01,
            HWIDeviceType::BitBox02,
            HWIDeviceType::Coldcard,
        ];
        for device in devices {
            let device = device.unwrap();
            if supported.contains(&device.device_type) {
                let client = HWIClient::get_client(&device, true, TESTNET).unwrap();
                client.backup_device(Some("My Label"), None).unwrap();
            }
        }
    }

    #[test]
    #[serial]
    #[ignore]
    fn test_wipe_device() {
        let devices = HWIClient::enumerate().unwrap();
        let unsupported = [
            HWIDeviceType::Ledger,
            HWIDeviceType::Coldcard,
            HWIDeviceType::Jade,
        ];
        for device in devices {
            let device = device.unwrap();
            if unsupported.contains(&device.device_type) {
                // These devices don't support wipe
                continue;
            }
            let client = HWIClient::get_client(&device, true, TESTNET).unwrap();
            client.wipe_device().unwrap();
        }
    }

    #[test]
    #[serial]
    #[ignore]
    fn test_install_hwi() {
        HWIClient::install_hwilib(Some("2.1.1")).unwrap();
    }
}
