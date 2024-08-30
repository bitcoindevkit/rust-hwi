//! Rust wrapper for the [Bitcoin Hardware Wallet Interface](https://github.com/bitcoin-core/HWI/).
//!
//! # Example - display address with path
//! ```no_run
//! use bitcoin::bip32::{ChildNumber, DerivationPath};
//! use hwi::error::Error;
//! use hwi::implementations::python_implementation::PythonHWIImplementation;
//! use hwi::interface::HWIClient;
//! use hwi::types;
//! use std::str::FromStr;
//!
//! fn main() -> Result<(), Error> {
//!     // Find information about devices
//!     let mut devices = HWIClient::<PythonHWIImplementation>::enumerate()?;
//!     if devices.is_empty() {
//!         panic!("No device found!");
//!     }
//!     let device = devices.remove(0)?;
//!     // Create a client for a device
//!     let client = HWIClient::<PythonHWIImplementation>::get_client(
//!         &device,
//!         true,
//!         bitcoin::Network::Testnet.into(),
//!     )?;
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
extern crate core;

pub use interface::HWIClient;

#[cfg(feature = "doctest")]
pub mod doctest;
pub mod error;
pub mod implementations;
pub mod interface;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::implementations::binary_implementation::BinaryHWIImplementation;
    use crate::implementations::python_implementation::PythonHWIImplementation;
    use crate::types::{self, HWIBinaryExecutor, HWIDeviceType, HWIImplementation, TESTNET};
    use crate::HWIClient;
    use std::collections::BTreeMap;
    use std::str::FromStr;

    use bitcoin::bip32::{DerivationPath, KeySource};
    use bitcoin::locktime::absolute;
    use bitcoin::psbt::{Input, Output};
    use bitcoin::{secp256k1, Transaction};
    use bitcoin::{transaction, Amount, Network, TxIn, TxOut};
    use std::path::PathBuf;
    use std::str;

    #[cfg(feature = "miniscript")]
    use miniscript::{Descriptor, DescriptorPublicKey};
    struct HWIBinaryExecutorImpl;

    impl HWIBinaryExecutor for HWIBinaryExecutorImpl {
        fn execute_command(args: Vec<String>) -> Result<String, Error> {
            let mut binary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            binary_path.push("hwi-binary/hwi");
            let output = std::process::Command::new(binary_path)
                .args(&args)
                .output()
                .map_err(|e| Error::Hwi(format!("Failed to execute command: {}", e), None))?;
            if output.status.success() {
                Ok(str::from_utf8(&output.stdout)
                    .map_err(|e| Error::Hwi(format!("Failed to parse HWI output: {}", e), None))?
                    .to_string())
            } else {
                Err(Error::Hwi(
                    str::from_utf8(&output.stderr)
                        .unwrap_or("Failed to parse error output")
                        .to_string(),
                    None,
                ))
            }
        }
    }

    macro_rules! generate_enumerate_test {
        ($impl:ty) => {
            #[test]
            #[serial]
            fn test_enumerate() {
                let devices = HWIClient::<$impl>::enumerate().unwrap();
                assert!(!devices.is_empty());
            }

            #[test]
            #[serial]
            #[ignore]
            fn test_find_trezor_device() {
                HWIClient::<$impl>::find_device(
                    None,
                    Some(HWIDeviceType::Trezor),
                    None,
                    false,
                    Network::Testnet,
                )
                .unwrap();
            }

            fn get_first_device() -> HWIClient<$impl> {
                let devices = HWIClient::<$impl>::enumerate().unwrap();
                let device = devices
                    .first()
                    .expect(
                        "No devices found. Either plug in a hardware wallet, or start a simulator.",
                    )
                    .as_ref()
                    .expect("Error when opening the first device");
                HWIClient::get_client(device, true, TESTNET).unwrap()
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
                assert!(!descriptor.internal.is_empty());
                assert!(!descriptor.receive.is_empty());
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
                assert!(!descriptor.internal.is_empty());
                assert!(!descriptor.receive.is_empty());
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

            // For testing on Coldcard, make sure to disable Max Network Fee in the Coldcard settings
            #[test]
            #[serial]
            fn test_sign_tx() {
                let devices = HWIClient::<$impl>::enumerate().unwrap();
                let device = devices.first().unwrap().as_ref().unwrap();
                let client = HWIClient::<$impl>::get_client(device, true, TESTNET).unwrap();
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
                    version: transaction::Version::ONE,
                    lock_time: absolute::LockTime::from_consensus(0),
                    input: vec![TxIn::default()],
                    output: vec![TxOut {
                        value: Amount::from_sat(100),
                        script_pubkey: script_pubkey.clone(),
                    }],
                };

                let previous_txin = TxIn {
                    previous_output: bitcoin::OutPoint {
                        txid: previous_tx.compute_txid(),
                        vout: Default::default(),
                    },
                    ..Default::default()
                };
                let psbt = bitcoin::Psbt {
                    unsigned_tx: Transaction {
                        version: transaction::Version::ONE,
                        lock_time: absolute::LockTime::from_consensus(0),
                        input: vec![previous_txin],
                        output: vec![TxOut {
                            value: Amount::from_sat(50),
                            script_pubkey,
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
                    HWIClient::<$impl>::install_udev_rules(None, None).unwrap()
                }
            }

            #[test]
            #[serial]
            fn test_toggle_passphrase() {
                let devices = HWIClient::<$impl>::enumerate().unwrap();
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
                    let client = HWIClient::<$impl>::get_client(&device, true, TESTNET).unwrap();
                    client.toggle_passphrase().unwrap();
                    break;
                }
            }

            #[test]
            #[serial]
            fn test_get_version() {
                HWIClient::<$impl>::get_version().unwrap();
            }

            #[test]
            #[serial]
            #[ignore]
            // At the moment (hwi v2.1.1 and trezor-firmware core v2.5.2) work only with physical devices and NOT emulators!
            fn test_setup_trezor_device() {
                let client = HWIClient::<$impl>::find_device(
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
                let client = HWIClient::<$impl>::find_device(
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
                let devices = HWIClient::<$impl>::enumerate().unwrap();
                let supported = [
                    HWIDeviceType::BitBox01,
                    HWIDeviceType::BitBox02,
                    HWIDeviceType::Coldcard,
                ];
                for device in devices {
                    let device = device.unwrap();
                    if supported.contains(&device.device_type) {
                        let client =
                            HWIClient::<$impl>::get_client(&device, true, TESTNET).unwrap();
                        client.backup_device(Some("My Label"), None).unwrap();
                    }
                }
            }
        };
    }

    mod test_python_tests {
        use super::*;
        generate_enumerate_test!(PythonHWIImplementation);
    }

    mod test_binary_tests {
        use super::*;
        generate_enumerate_test!(BinaryHWIImplementation<HWIBinaryExecutorImpl>);
    }

    #[test]
    #[serial]
    fn test_python_set_log_level() {
        HWIClient::<PythonHWIImplementation>::set_log_level(types::LogLevel::DEBUG).unwrap();
        let devices = HWIClient::<PythonHWIImplementation>::enumerate().unwrap();
        assert!(!devices.is_empty());
    }

    #[test]
    #[serial]
    #[ignore]
    fn test_python_install_hwi() {
        HWIClient::<PythonHWIImplementation>::install_hwilib(Some("2.1.1")).unwrap();
    }

    #[test]
    #[serial]
    #[ignore]
    fn test_python_wipe_device() {
        wipe_device::<PythonHWIImplementation>();
    }

    #[test]
    #[serial]
    #[ignore]
    fn test_binary_wipe_device() {
        wipe_device::<BinaryHWIImplementation<HWIBinaryExecutorImpl>>();
    }

    fn wipe_device<T: HWIImplementation>() {
        let devices = HWIClient::<T>::enumerate().unwrap();
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
            let client = HWIClient::<T>::get_client(&device, true, TESTNET).unwrap();
            client.wipe_device().unwrap();
        }
    }
}
