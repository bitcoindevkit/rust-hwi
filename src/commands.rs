use bitcoin::util::bip32::Fingerprint;

use std::fmt;
use std::process::Command;

#[derive(Display)]
pub enum HWISubcommand {
    Enumerate,
    GetMasterXpub,
    SignTx,
    GetXpub,
    SignMessage,
    GetKeypool,
    GetDescriptors,
    DisplayAddress,
    Setup,
    Wipe,
    Restore,
    Backup,
    PromptPin,
    SendPin
}

#[derive(Debug)]
pub enum HWIFlag {
    DevicePath(String),
    DeviceType(String),
    Password(String),
    StdinPass,
    Testnet,
    Debug,
    Fingerprint(Fingerprint),
    Version,
    Stdin,
    Interactive,
    Expert,
}

impl fmt::Display for HWIFlag {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HWIFlag::DevicePath(p) => write!(formatter, "--device-path {}", p),
            HWIFlag::DeviceType(t) => write!(formatter, "--device-type {}", t),
            HWIFlag::Password(p) => write!(formatter, "--password {}", p),
            HWIFlag::Fingerprint(f) => write!(formatter, "--fingerprint {:?}", f),
            _ => write!(formatter, "--{:?}", self),
        }
    }
}

#[derive(Debug)]
pub struct HWICommand {
    command: Command,
}

impl HWICommand {
    pub fn new() -> Self {
        HWICommand {
            command: Command::new("hwi")
        }
    }

    pub fn add_subcommand(&mut self, s: HWISubcommand) -> &Self {
        self.command.arg(s.to_string().to_lowercase());
        self
    }

    pub fn add_flag(&mut self, f: HWIFlag) -> &Self {
        self.command.arg(f.to_string().to_lowercase());
        self
    }
}
