use std::process::{Command, Output};

use bitcoin::util::bip32::{DerivationPath, Fingerprint};

use crate::error::Error;
use crate::types::HWIAddressType;

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
    // TODO: Setup,
    // TODO: Wipe,
    // TODO: Restore,
    // TODO: Backup,
    // TODO: PromptPin,
    // TODO: SendPin,
}

#[derive(Debug)]
pub enum HWIFlag {
    DevicePath(String),
    DeviceType(String),
    Password(String),
    // TODO: StdinPass,
    Testnet,
    // TODO: Debug,
    Fingerprint(Fingerprint),
    // TODO: Version,
    // TODO: Stdin,
    // TODO: Interactive,
}

impl HWIFlag {
    fn to_args_vec(&self) -> Vec<String> {
        match self {
            HWIFlag::DevicePath(p) => vec![String::from("--device-path"), format!("{}", p)],
            HWIFlag::DeviceType(t) => vec![String::from("--device-type"), format!("{}", t)],
            HWIFlag::Password(p) => vec![String::from("--password"), format!("{}", p)],
            HWIFlag::Fingerprint(f) => vec![String::from("--fingerprint"), format!("{}", f)],
            _ => vec![format!("--{:?}", self).to_lowercase()],
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
            command: Command::new("hwi"),
        }
    }

    pub fn add_subcommand(&mut self, s: HWISubcommand) -> &mut Self {
        self.command.arg(s.to_string().to_lowercase());
        self
    }

    pub fn add_flag(&mut self, f: HWIFlag) -> &mut Self {
        self.command.args(f.to_args_vec());
        self
    }

    pub fn add_path(&mut self, p: &DerivationPath, flag: &bool, star: &bool) -> &mut Self {
        let mut v = Vec::new();

        if *flag {
            v.push(String::from("--path"));
        }

        let mut s = p.to_string();

        if *star {
            s = format!("{}/*", s);
        }

        v.push(s);

        self.command.args(v);
        self
    }

    // Command escapes m, preventing injections
    pub fn add_message(&mut self, m: &String) -> &mut Self {
        self.command.arg(m);
        self
    }

    pub fn add_descriptor(&mut self, d: &String) -> &mut Self {
        self.command.args(vec!["--desc", d]);
        self
    }

    pub fn add_account(&mut self, a: &u32) -> &mut Self {
        self.command.args(vec!["--account", &a.to_string()[..]]);
        self
    }

    pub fn add_address_type(&mut self, address_type: &HWIAddressType) -> &mut Self {
        match address_type {
            HWIAddressType::ShWpkh => {
                self.command.arg("--sh_wpkh");
            }
            HWIAddressType::Wpkh => {
                self.command.arg("--wpkh");
            }
            _ => {}
        };
        self
    }

    pub fn add_psbt(&mut self, p: &String) -> &mut Self {
        self.command.arg(p);
        self
    }

    pub fn add_keypool(&mut self, k: &bool) -> &mut Self {
        if !k {
            self.command.arg("--nokeypool");
        }
        self
    }

    pub fn add_internal(&mut self, i: &bool) -> &mut Self {
        if *i {
            self.command.arg("--internal");
        }
        self
    }

    pub fn add_start_end(&mut self, start: &u32, end: &u32) -> &mut Self {
        self.command
            .args(vec![format!("{}", start), format!("{}", end)]);
        self
    }

    pub fn add_testnet(&mut self, testnet: &bool) -> &mut Self {
        if *testnet {
            self.add_flag(HWIFlag::Testnet);
        }
        self
    }

    // TODO: maybe deserialize here?
    pub fn execute(&mut self) -> Result<Output, Error> {
        Ok(self.command.output()?)
    }
}
