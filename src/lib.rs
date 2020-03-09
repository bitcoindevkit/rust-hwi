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
        let devices = interface::enumerate().unwrap();
        assert!(devices.len() > 0);
    }

    fn get_first_device() -> types::HWIDevice {
        interface::enumerate()
            .unwrap()
            .first()
            .expect("No devices")
            .clone()
    }

    #[test]
    #[serial]
    fn test_get_master_xpub() {
        let device = get_first_device();
        interface::get_master_xpub(&device).unwrap();
    }

    #[test]
    #[serial]
    fn test_get_xpub() {
        let device = get_first_device();
        let derivation_path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(44).unwrap(),
            ChildNumber::from_normal_idx(0).unwrap(),
        ]);
        interface::get_xpub(&device, &derivation_path).unwrap();
    }

    #[test]
    #[serial]
    fn test_sign_message() {
        let device = get_first_device();
        let derivation_path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(44).unwrap(),
            ChildNumber::from_normal_idx(0).unwrap(),
        ]);
        interface::sign_message(
            &device,
            &String::from("I love magical bitcoin wallet"),
            &derivation_path,
        )
        .unwrap();
    }

    #[test]
    #[serial]
    fn test_get_descriptors() {
        let device = get_first_device();
        let account = Some(&10);
        let descriptor = interface::get_descriptors(&device, account).unwrap();
        assert!(descriptor.internal.len() > 0);
        assert!(descriptor.receive.len() > 0);
    }

    #[test]
    #[serial]
    fn test_display_address_with_desc() {
        let device = get_first_device();
        let descriptor = interface::get_descriptors(&device, None).unwrap();
        let descriptor = descriptor.receive.first().unwrap();
        // Seems like hwi doesn't support descriptors checksums
        let descriptor = &descriptor.split("#").collect::<Vec<_>>()[0].to_string();
        let descriptor = &descriptor.replace("*", "1"); // e.g. /0/* -> /0/1
        interface::display_address_with_desc(&device, &descriptor).unwrap();
    }

    #[test]
    #[serial]
    fn test_display_address_with_path_pkh() {
        let device = get_first_device();
        let derivation_path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(44).unwrap(),
            ChildNumber::from_normal_idx(0).unwrap(),
        ]);
        interface::display_address_with_path(
            &device,
            &derivation_path,
            &types::HWIAddressType::Pkh,
        )
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
        interface::display_address_with_path(
            &device,
            &derivation_path,
            &types::HWIAddressType::ShWpkh,
        )
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
        interface::display_address_with_path(
            &device,
            &derivation_path,
            &types::HWIAddressType::Wpkh,
        )
        .unwrap();
    }

    /*
       TODO: generalize this test using magical
       Only works with my coldcard simulator at the moment
    #[test]
    #[serial]
    fn test_sign_tx() {
        use bitcoin::util::psbt::PartiallySignedTransaction;
        use bitcoin::consensus::encode::{deserialize};
        let device = get_first_device();
        let psbt_buf = base64::decode("cHNidP8BAHcCAAAAASo8/raGYGem0zxP2aiKIBrRy9hNq/d2StEtpG9FUW/lAQAAAAD/////AugDAAAAAAAAGXapFJ/833DkRBlDm/fsYT0ANYSvRXsYiKxFIgAAAAAAABl2qRQPoiwXDpKAOmsHT0NcjIr4XBox84isAAAAAAABAOICAAAAAAEB7ilZ8keXcKRJbLjjCY+54jB1tweZW19jiljUtLpMGkYBAAAAAP7///8CgMAfAAAAAAAXqRT9zZYHcN0ixQUrvSHehgvly6RNfIcQJwAAAAAAABl2qRT/y++vdIZvp4hfjfIllGi72Sml5YisAkcwRAIgX+b63MO4jU1G0LkkMDrMsfPPW1SHIAO0Pyu5vUEFZqECIDL/HNS4VB5Qti1B0xuSot8uEWL2N63Mmj5dL6UYh1ZzASECMgf20/uJZlO/yPEZjvm/ZhNmWqhfANrIiIQFzM/ewJcIeRkAAQMEAQAAACIGAkX4058RlrR33vfbHqdqlBi3+Z64WuSrTr7Qtn38aycuGA8FaUMsAACAAAAAgAAAAIAAAAAAAAAAAAAAAA==").unwrap();
        let psbt: PartiallySignedTransaction = deserialize(&psbt_buf).unwrap();
        interface::sign_tx(&device, &psbt).unwrap();
    }
    */

    #[test]
    #[serial]
    fn test_get_keypool() {
        let device = get_first_device();
        let keypool = true;
        let internal = false;
        let address_type = types::HWIAddressType::Pkh;
        let account = Some(&8);
        let derivation_path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(44).unwrap(),
            ChildNumber::from_normal_idx(0).unwrap(),
        ]);
        let start = 1;
        let end = 5;
        interface::get_keypool(
            &device,
            &keypool,
            &internal,
            &address_type,
            account,
            Some(&derivation_path),
            &start,
            &end,
        )
        .unwrap();

        let keypool = true;
        let internal = true;
        let address_type = types::HWIAddressType::Wpkh;
        let account = None;
        let start = 1;
        let end = 8;
        interface::get_keypool(
            &device,
            &keypool,
            &internal,
            &address_type,
            account,
            None,
            &start,
            &end,
        )
        .unwrap();

        let keypool = false;
        let internal = true;
        let address_type = types::HWIAddressType::ShWpkh;
        let account = Some(&1);
        let start = 0;
        let end = 10;
        interface::get_keypool(
            &device,
            &keypool,
            &internal,
            &address_type,
            account,
            Some(&derivation_path),
            &start,
            &end,
        )
        .unwrap();
    }
}
