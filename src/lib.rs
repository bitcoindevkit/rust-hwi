#[macro_use]
extern crate strum_macros;

#[cfg(test)]
#[macro_use]
extern crate serial_test;

pub mod commands;
pub mod error;
pub mod interface;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::interface;
    use crate::types;

    use bitcoin::util::bip32::{ChildNumber, DerivationPath};

    #[test]
    #[serial]
    fn test_enumerate() {
        let devices = interface::HWIDevice::enumerate().unwrap();
        assert!(devices.len() > 0);
    }

    fn get_first_device() -> interface::HWIDevice {
        interface::HWIDevice::enumerate()
            .unwrap()
            .first()
            .expect("No devices")
            .clone()
    }

    #[test]
    #[serial]
    fn test_get_master_xpub() {
        let device = get_first_device();
        device.get_master_xpub(true).unwrap();
    }

    #[test]
    #[serial]
    fn test_get_xpub() {
        let device = get_first_device();
        let derivation_path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(44).unwrap(),
            ChildNumber::from_normal_idx(0).unwrap(),
        ]);
        device.get_xpub(&derivation_path, true).unwrap();
    }

    #[test]
    #[serial]
    fn test_sign_message() {
        let device = get_first_device();
        let derivation_path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(44).unwrap(),
            ChildNumber::from_normal_idx(0).unwrap(),
        ]);
        device
            .sign_message("I love magical bitcoin wallet", &derivation_path, true)
            .unwrap();
    }

    #[test]
    #[serial]
    fn test_get_descriptors() {
        let device = get_first_device();
        let account = Some(10);
        let descriptor = device.get_descriptors(account, true).unwrap();
        assert!(descriptor.internal.len() > 0);
        assert!(descriptor.receive.len() > 0);
    }

    #[test]
    #[serial]
    fn test_display_address_with_desc() {
        let device = get_first_device();
        let descriptor = device.get_descriptors(None, true).unwrap();
        let descriptor = descriptor.receive.first().unwrap();
        // Seems like hwi doesn't support descriptors checksums
        let descriptor = &descriptor.split("#").collect::<Vec<_>>()[0].to_string();
        let descriptor = &descriptor.replace("*", "1"); // e.g. /0/* -> /0/1
        device.display_address_with_desc(&descriptor, true).unwrap();
    }

    #[test]
    #[serial]
    fn test_display_address_with_path_pkh() {
        let device = get_first_device();
        let derivation_path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(44).unwrap(),
            ChildNumber::from_normal_idx(0).unwrap(),
        ]);
        device
            .display_address_with_path(&derivation_path, types::HWIAddressType::Pkh, true)
            .unwrap();
    }

    #[test]
    #[serial]
    fn test_display_address_with_path_shwpkh() {
        let device = get_first_device();
        let derivation_path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(44).unwrap(),
            ChildNumber::from_normal_idx(0).unwrap(),
        ]);
        device
            .display_address_with_path(&derivation_path, types::HWIAddressType::ShWpkh, true)
            .unwrap();
    }

    #[test]
    #[serial]
    fn test_display_address_with_path_wpkh() {
        let device = get_first_device();
        let derivation_path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(44).unwrap(),
            ChildNumber::from_normal_idx(0).unwrap(),
        ]);
        device
            .display_address_with_path(&derivation_path, types::HWIAddressType::Wpkh, true)
            .unwrap();
    }

    // TODO:
    // #[test]
    // #[serial]
    // fn test_sign_tx() {
    // }

    #[test]
    #[serial]
    fn test_get_keypool() {
        let device = get_first_device();
        let keypool = true;
        let internal = false;
        let address_type = types::HWIAddressType::Pkh;
        let account = Some(8);
        let derivation_path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(44).unwrap(),
            ChildNumber::from_normal_idx(0).unwrap(),
        ]);
        let start = 1;
        let end = 5;
        device
            .get_keypool(
                keypool,
                internal,
                address_type,
                account,
                Some(&derivation_path),
                start,
                end,
                true,
            )
            .unwrap();

        let keypool = true;
        let internal = true;
        let address_type = types::HWIAddressType::Wpkh;
        let account = None;
        let start = 1;
        let end = 8;
        device
            .get_keypool(
                keypool,
                internal,
                address_type,
                account,
                None,
                start,
                end,
                true,
            )
            .unwrap();

        let keypool = false;
        let internal = true;
        let address_type = types::HWIAddressType::ShWpkh;
        let account = Some(1);
        let start = 0;
        let end = 10;
        device
            .get_keypool(
                keypool,
                internal,
                address_type,
                account,
                Some(&derivation_path),
                start,
                end,
                true,
            )
            .unwrap();
    }
}
