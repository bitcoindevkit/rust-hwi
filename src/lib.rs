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
    use crate::types::HWIDevice;

    use bitcoin::util::bip32::{ChildNumber, DerivationPath};

    #[test]
    #[serial]
    fn test_enumerate() {
        let devices = interface::enumerate().unwrap();
        assert!(devices.len() > 0);
    }

    fn get_first_device() -> HWIDevice {
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
        let account = Some(10);
        let descriptor = interface::get_descriptors(&device, account).unwrap();
        assert!(descriptor.internal.len() > 0);
        assert!(descriptor.receive.len() > 0);
    }

    #[test]
    #[serial]
    fn test_display_address() {
        let device = get_first_device();
        let descriptor = interface::get_descriptors(&device, None).unwrap();
        let descriptor = descriptor.receive.first().unwrap();
        // Seems like hwi doesn't support descriptors checksums
        let descriptor = &descriptor.split("#").collect::<Vec<_>>()[0].to_string();
        let descriptor = &descriptor.replace("*", "1");     // e.g. /0/* -> /0/1
        interface::display_address(&device, &descriptor).unwrap();
    }
}
