use crate::error::Error;
use crate::types::{
    HWIAddressType, HWIChain, HWIDevice, HWIDeviceType, HWIImplementation, LogLevel,
};
use bitcoin::Psbt;
use pyo3::{prelude::*, py_run};
use std::ops::Deref;
use std::process::Command;

/// Convenience class containing required Python objects
#[derive(Debug)]
struct HWILib {
    commands: Py<PyModule>,
    json_dumps: Py<PyAny>,
}

impl HWILib {
    pub fn initialize() -> Result<Self, Error> {
        Python::with_gil(|py| {
            let commands: Py<PyModule> = PyModule::import_bound(py, "hwilib.commands")?.into();
            let json_dumps: Py<PyAny> =
                PyModule::import_bound(py, "json")?.getattr("dumps")?.into();
            Ok(HWILib {
                commands,
                json_dumps,
            })
        })
    }
}

#[derive(Debug)]
pub struct PythonHWIImplementation {
    hwilib: HWILib,
    hw_client: PyObject,
}

impl Deref for PythonHWIImplementation {
    type Target = PyObject;

    fn deref(&self) -> &Self::Target {
        &self.hw_client
    }
}

impl HWIImplementation for PythonHWIImplementation {
    fn enumerate() -> Result<String, Error> {
        let libs = HWILib::initialize()?;
        Python::with_gil(|py| {
            let output = libs.commands.getattr(py, "enumerate")?.call0(py)?;
            let output = libs.json_dumps.call1(py, (output,))?;
            Ok(output.to_string())
        })
    }

    fn get_client(device: &HWIDevice, expert: bool, chain: HWIChain) -> Result<Self, Error> {
        let libs = HWILib::initialize()?;
        let hw_client = Python::with_gil(|py| {
            let client_args = (
                device.device_type.to_string(),
                &device.path,
                "",
                expert,
                chain,
            );
            libs.commands
                .getattr(py, "get_client")?
                .call1(py, client_args)
        })?;

        Ok(Self {
            hwilib: libs,
            hw_client,
        })
    }

    fn find_device(
        password: Option<&str>,
        device_type: Option<HWIDeviceType>,
        fingerprint: Option<&str>,
        expert: bool,
        chain: HWIChain,
    ) -> Result<Self, Error> {
        let libs = HWILib::initialize()?;
        let hw_client = Python::with_gil(|py| {
            let client_args = (
                password.unwrap_or(""),
                device_type.map_or_else(String::new, |d| d.to_string()),
                fingerprint.unwrap_or(""),
                expert,
                chain,
            );
            let client = libs
                .commands
                .getattr(py, "find_device")?
                .call1(py, client_args)?;
            if client.is_none(py) {
                return Err(Error::Hwi("device not found".to_string(), None));
            }
            Ok(client)
        })?;

        Ok(Self {
            hwilib: libs,
            hw_client,
        })
    }

    fn get_master_xpub(&self, addrtype: HWIAddressType, account: u32) -> Result<String, Error> {
        Python::with_gil(|py| {
            let func_args = (&self.hw_client, addrtype, account);
            let output = self
                .hwilib
                .commands
                .getattr(py, "getmasterxpub")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            Ok(output.to_string())
        })
    }

    fn sign_tx(&self, psbt: &Psbt) -> Result<String, Error> {
        Python::with_gil(|py| {
            let output = self
                .hwilib
                .commands
                .getattr(py, "signtx")?
                .call1(py, (&self.hw_client, psbt.to_string()))?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            Ok(output.to_string())
        })
    }

    fn get_xpub(&self, path: &str, expert: bool) -> Result<String, Error> {
        Python::with_gil(|py| {
            let func_args = (&self.hw_client, path, expert);
            let output = self
                .hwilib
                .commands
                .getattr(py, "getxpub")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            Ok(output.to_string())
        })
    }

    fn sign_message(&self, message: &str, path: &str) -> Result<String, Error> {
        Python::with_gil(|py| {
            let func_args = (&self.hw_client, message, path);
            let output = self
                .hwilib
                .commands
                .getattr(py, "signmessage")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            Ok(output.to_string())
        })
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
        Python::with_gil(|py| {
            let p_str = path.map_or(py.None(), |p| p.into_py(py));
            let func_args = (
                &self.hw_client,
                p_str,
                start,
                end,
                internal,
                keypool,
                account,
                addr_type,
                addr_all,
            );
            let output = self
                .hwilib
                .commands
                .getattr(py, "getkeypool")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            Ok(output.to_string())
        })
    }

    fn get_descriptors(&self, account: u32) -> Result<String, Error> {
        Python::with_gil(|py| {
            let func_args = (&self.hw_client, account);
            let output = self
                .hwilib
                .commands
                .getattr(py, "getdescriptors")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            Ok(output.to_string())
        })
    }

    fn display_address_with_desc(&self, descriptor: &str) -> Result<String, Error> {
        Python::with_gil(|py| {
            let path = py.None();
            let func_args = (&self.hw_client, path, descriptor);
            let output = self
                .hwilib
                .commands
                .getattr(py, "displayaddress")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            Ok(output.to_string())
        })
    }

    fn display_address_with_path(
        &self,
        path: &str,
        address_type: HWIAddressType,
    ) -> Result<String, Error> {
        Python::with_gil(|py| {
            let descriptor = py.None();
            let func_args = (&self.hw_client, path, descriptor, address_type);
            let output = self
                .hwilib
                .commands
                .getattr(py, "displayaddress")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            Ok(output.to_string())
        })
    }

    fn install_udev_rules(source: &str, location: &str) -> Result<String, Error> {
        let libs = HWILib::initialize()?;

        Python::with_gil(|py| {
            let func_args = (source, location);
            let output = libs
                .commands
                .getattr(py, "install_udev_rules")?
                .call1(py, func_args)?;
            let output = libs.json_dumps.call1(py, (output,))?;
            Ok(output.to_string())
        })
    }

    fn set_log_level(level: LogLevel) -> Result<(), Error> {
        Python::with_gil(|py| {
            let arg = match level {
                LogLevel::DEBUG => 10,
                LogLevel::INFO => 20,
                LogLevel::WARNING => 30,
                LogLevel::ERROR => 40,
                LogLevel::CRITICAL => 50,
            };
            py_run!(
                py,
                arg,
                r#"
                import logging
                logging.basicConfig(level=arg)            
                "#
            );
            Ok(())
        })
    }

    fn toggle_passphrase(&self) -> Result<String, Error> {
        Python::with_gil(|py| {
            let func_args = (&self.hw_client,);
            let output = self
                .hwilib
                .commands
                .getattr(py, "toggle_passphrase")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            Ok(output.to_string())
        })
    }

    fn setup_device(&self, label: &str, passphrase: &str) -> Result<String, Error> {
        Python::with_gil(|py| {
            let func_args = (&self.hw_client, label, passphrase);
            let output = self
                .hwilib
                .commands
                .getattr(py, "setup_device")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            Ok(output.to_string())
        })
    }

    fn restore_device(&self, label: &str, word_count: u8) -> Result<String, Error> {
        Python::with_gil(|py| {
            let func_args = (&self.hw_client, label, word_count);
            let output = self
                .hwilib
                .commands
                .getattr(py, "restore_device")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            Ok(output.to_string())
        })
    }

    fn backup_device(&self, label: &str, backup_passphrase: &str) -> Result<String, Error> {
        Python::with_gil(|py| {
            let func_args = (&self.hw_client, label, backup_passphrase);
            let output = self
                .hwilib
                .commands
                .getattr(py, "backup_device")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            Ok(output.to_string())
        })
    }

    fn wipe_device(&self) -> Result<String, Error> {
        Python::with_gil(|py| {
            let func_args = (&self.hw_client,);
            let output = self
                .hwilib
                .commands
                .getattr(py, "wipe_device")?
                .call1(py, func_args)?;
            let output = self.hwilib.json_dumps.call1(py, (output,))?;
            Ok(output.to_string())
        })
    }

    fn get_version() -> Result<String, Error> {
        Python::with_gil(|py| {
            let hwilib = PyModule::import_bound(py, "hwilib")?;
            let version = hwilib.getattr("__version__")?.extract::<String>()?;

            Ok(version)
        })
    }

    fn install_hwilib(version: String) -> Result<(), Error> {
        let output = Command::new("pip")
            .args(vec!["install", "--user", &version])
            .output()
            .map_err(|e| Error::Hwi(format!("Failed to execute pip: {}", e), None))?;

        if output.status.success() {
            Ok(())
        } else {
            let error_message = String::from_utf8_lossy(&output.stderr).into_owned();
            Err(Error::Hwi(
                format!("Failed to install HWI: {}", error_message),
                None,
            ))
        }
    }
}
