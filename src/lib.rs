#[macro_use]
extern crate strum_macros;

#[cfg(test)]
#[macro_use]
extern crate serial_test;

pub mod commands;
pub mod interface;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::interface;
    use crate::types::HWIDevice;

    use bitcoin::util::bip32::{ChildNumber, DerivationPath};

    #[test]
    #[serial]
    fn enumerate() {
        let devices = interface::enumerate();
        assert!(devices.len() > 0);
    }

    fn get_device() -> HWIDevice {
        let devices = interface::enumerate();
        devices[0].clone()
    }

    #[test]
    #[serial]
    fn getmasterxpub() {
        let device = get_device();
        let pb = interface::getmasterxpub(&device);
        assert!(pb.contains_key("xpub"));
    }

    #[test]
    #[serial]
    fn getxpub() {
        let device = get_device();
        let derivation_path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(44).unwrap(),
            ChildNumber::from_normal_idx(0).unwrap(),
        ]);
        let pb = interface::getxpub(&device, &derivation_path);
        assert!(pb.contains_key("xpub"));
    }

    #[test]
    #[serial]
    fn getdescriptors() {
        let device = get_device();
        let descriptor = interface::getdescriptors(&device);
        assert!(descriptor.internal.len() > 0);
        assert!(descriptor.receive.len() > 0);
    }
}
