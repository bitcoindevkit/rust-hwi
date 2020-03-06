use std::process::{Command, Output};

use bitcoin::util::bip32::{DerivationPath, Fingerprint};

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
    SendPin,
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

    pub fn add_path(&mut self, p: &DerivationPath) -> &mut Self {
        self.command.arg(p.to_string());
        self
    }

    // TODO: maybe deserialize here?
    pub fn execute(&mut self) -> Output {
        self.command
            .output()
            .expect("Failed to call HWI. Do you have it installed?")
    }
}
