use serde_json::value::Value;
use std::str;

use crate::error::Error;
use crate::types::{
    HWIAddressType, HWIBinaryExecutor, HWIChain, HWIDevice, HWIDeviceType, HWIImplementation,
    LogLevel,
};
use bitcoin::Psbt;

macro_rules! deserialize_obj {
    ( $e: expr ) => {{
        let value: Value = serde_json::from_str($e)?;
        let obj = value.clone();
        serde_json::from_value(value)
            .map_err(|e| Error::Hwi(format!("error {} while deserializing {}", e, obj), None))
    }};
}

#[derive(Debug)]
pub struct BinaryHWIImplementation<T: HWIBinaryExecutor> {
    device: Option<HWIDevice>,
    expert: bool,
    chain: HWIChain,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: HWIBinaryExecutor> HWIImplementation for BinaryHWIImplementation<T> {
    fn enumerate() -> Result<String, Error> {
        let output =
            BinaryHWIImplementation::<T>::run_hwi_command(None, false, None, vec!["enumerate"])?;
        Ok(output.to_string())
    }

    fn get_client(device: &HWIDevice, expert: bool, chain: HWIChain) -> Result<Self, Error> {
        Ok(Self {
            device: Some(device.clone()),
            expert,
            chain,
            _phantom: std::marker::PhantomData,
        })
    }

    fn find_device(
        password: Option<&str>,
        device_type: Option<HWIDeviceType>,
        fingerprint: Option<&str>,
        expert: bool,
        chain: HWIChain,
    ) -> Result<Self, Error> {
        let mut client = BinaryHWIImplementation {
            device: None,
            expert,
            chain,
            _phantom: std::marker::PhantomData,
        };

        let mut args = vec!["enumerate"];

        if let Some(pw) = password {
            args.extend_from_slice(&["--password", pw]);
        }

        let output =
            BinaryHWIImplementation::<T>::run_hwi_command(None, expert, Some(&client.chain), args)?;
        let devices: Vec<HWIDevice> = deserialize_obj!(&output)?;

        let device = devices
            .into_iter()
            .find(|d| {
                device_type.as_ref().map_or(true, |t| &d.device_type == t)
                    && fingerprint.map_or(true, |f| d.fingerprint.to_string() == f)
            })
            .ok_or_else(|| Error::Hwi("No matching device found".to_string(), None))?;

        client.device = Some(device);
        Ok(client)
    }

    fn get_master_xpub(&self, addrtype: HWIAddressType, account: u32) -> Result<String, Error> {
        let mut args = vec!["getmasterxpub"];
        let addrtype_str = addrtype.to_string();
        let account_str = account.to_string();
        args.extend_from_slice(&["--addr-type", &addrtype_str]);
        args.extend_from_slice(&["--account", &account_str]);

        BinaryHWIImplementation::<T>::run_hwi_command(
            self.device.as_ref(),
            self.expert,
            Some(&self.chain),
            args,
        )
    }

    fn sign_tx(&self, psbt: &Psbt) -> Result<String, Error> {
        let psbt_str = psbt.to_string();
        let args = vec!["signtx", &psbt_str];

        let output = BinaryHWIImplementation::<T>::run_hwi_command(
            self.device.as_ref(),
            self.expert,
            Some(&self.chain),
            args,
        )?;
        Ok(output)
    }

    fn get_xpub(&self, path: &str, expert: bool) -> Result<String, Error> {
        let args = vec!["getxpub", &path];

        BinaryHWIImplementation::<T>::run_hwi_command(
            self.device.as_ref(),
            expert,
            Some(&self.chain),
            args,
        )
    }

    fn sign_message(&self, message: &str, path: &str) -> Result<String, Error> {
        let args = vec!["signmessage", message, path];

        BinaryHWIImplementation::<T>::run_hwi_command(
            self.device.as_ref(),
            self.expert,
            Some(&self.chain),
            args,
        )
    }

    fn get_keypool(
        &self,
        keypool: bool,
        internal: bool,
        addr_type: HWIAddressType,
        addr_all: bool,
        account: u32,
        path: Option<String>,
        start: u32,
        end: u32,
    ) -> Result<String, Error> {
        let mut args = vec!["getkeypool"];

        if keypool {
            args.push("--keypool");
        }
        if internal {
            args.push("--internal");
        }
        let addrtype_str = addr_type.to_string();
        args.extend_from_slice(&["--addr-type", &addrtype_str]);
        if addr_all {
            args.push("--addr-all");
        }
        let account_str = account.to_string();
        args.extend_from_slice(&["--account", &account_str]);
        if let Some(p) = path.as_deref() {
            args.extend_from_slice(&["--path", p]);
        }
        let start_str = start.to_string();
        args.push(&start_str);
        let end_str = end.to_string();
        args.push(&end_str);

        BinaryHWIImplementation::<T>::run_hwi_command(
            self.device.as_ref(),
            self.expert,
            Some(&self.chain),
            args,
        )
    }

    fn get_descriptors(&self, account: u32) -> Result<String, Error> {
        let mut args = vec!["getdescriptors"];
        let account_str = account.to_string();
        args.extend_from_slice(&["--account", &account_str]);

        BinaryHWIImplementation::<T>::run_hwi_command(
            self.device.as_ref(),
            self.expert,
            Some(&self.chain),
            args,
        )
    }

    fn display_address_with_desc(&self, descriptor: &str) -> Result<String, Error> {
        let mut args = vec!["displayaddress"];
        args.push("--desc");
        args.push(descriptor);

        BinaryHWIImplementation::<T>::run_hwi_command(
            self.device.as_ref(),
            self.expert,
            Some(&self.chain),
            args,
        )
    }

    fn display_address_with_path(
        &self,
        path: &str,
        address_type: HWIAddressType,
    ) -> Result<String, Error> {
        let mut args = vec!["displayaddress"];
        args.extend_from_slice(&["--path", path]);
        let addr_type_str = address_type.to_string();
        args.extend_from_slice(&["--addr-type", &addr_type_str]);

        BinaryHWIImplementation::<T>::run_hwi_command(
            self.device.as_ref(),
            self.expert,
            Some(&self.chain),
            args,
        )
    }

    fn install_udev_rules(_: &str, location: &str) -> Result<String, Error> {
        let mut args = vec!["installudevrules"];
        args.extend_from_slice(&["--location", location]);

        BinaryHWIImplementation::<T>::run_hwi_command(None, false, None, args)
    }

    fn set_log_level(_: LogLevel) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }

    fn toggle_passphrase(&self) -> Result<String, Error> {
        let args = vec!["togglepassphrase"];
        BinaryHWIImplementation::<T>::run_hwi_command(
            self.device.as_ref(),
            self.expert,
            Some(&self.chain),
            args,
        )
    }

    fn setup_device(&self, label: &str, passphrase: &str) -> Result<String, Error> {
        let mut args = vec!["setup"];
        args.extend_from_slice(&["--label", label]);
        args.extend_from_slice(&["--backup_passphrase", passphrase]);

        BinaryHWIImplementation::<T>::run_hwi_command(
            self.device.as_ref(),
            self.expert,
            Some(&self.chain),
            args,
        )
    }

    fn restore_device(&self, label: &str, word_count: u8) -> Result<String, Error> {
        let mut args = vec!["restore"];
        let word_count_str = word_count.to_string();
        args.extend_from_slice(&["--word_count", &word_count_str]);
        args.extend_from_slice(&["--label", label]);

        BinaryHWIImplementation::<T>::run_hwi_command(
            self.device.as_ref(),
            self.expert,
            Some(&self.chain),
            args,
        )
    }
    fn backup_device(&self, label: &str, backup_passphrase: &str) -> Result<String, Error> {
        let mut args = vec!["backup"];
        args.extend_from_slice(&["--label", label]);
        args.extend_from_slice(&["--backup_passphrase", backup_passphrase]);

        BinaryHWIImplementation::<T>::run_hwi_command(
            self.device.as_ref(),
            self.expert,
            Some(&self.chain),
            args,
        )
    }

    fn wipe_device(&self) -> Result<String, Error> {
        let args = vec!["wipe"];
        BinaryHWIImplementation::<T>::run_hwi_command(
            self.device.as_ref(),
            self.expert,
            Some(&self.chain),
            args,
        )
    }

    fn get_version() -> Result<String, Error> {
        let args = vec!["--version"];
        BinaryHWIImplementation::<T>::run_hwi_command(None, false, None, args)
    }

    fn install_hwilib(_: String) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }
}

impl<T: HWIBinaryExecutor> BinaryHWIImplementation<T> {
    fn run_hwi_command(
        device: Option<&HWIDevice>,
        expert: bool,
        chain: Option<&HWIChain>,
        args: Vec<&str>,
    ) -> Result<String, Error> {
        let mut command_args = Vec::new();

        if !args.contains(&"enumerate") && !args.contains(&"--version") {
            let fingerprint = device
                .ok_or(Error::Hwi("Device fingerprint not set".to_string(), None))?
                .fingerprint;
            command_args.push("--fingerprint".to_string());
            command_args.push(fingerprint.to_string());
        }

        if expert {
            command_args.push("--expert".to_string());
        }

        if let Some(c) = chain {
            command_args.push("--chain".to_string());
            command_args.push(c.to_string());
        }

        command_args.extend(args.iter().map(|s| s.to_string()));

        T::execute_command(command_args)
    }
}
