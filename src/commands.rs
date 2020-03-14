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
    /// Returns a vector of args from a flag
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
    /// Creates a new HWICommand
    /// # Examples
    /// ```
    /// use hwi::commands::HWICommand;
    ///
    /// let command = HWICommand::new();
    /// ```
    pub fn new() -> Self {
        HWICommand {
            command: Command::new("hwi"),
        }
    }

    /// Adds a HWISubcommand to a HWICommand
    /// # Examples
    /// ```
    /// use hwi::commands::{HWICommand, HWISubcommand};
    ///
    /// let mut command = HWICommand::new();
    /// command.add_subcommand(HWISubcommand::Enumerate);
    /// ```
    pub fn add_subcommand(&mut self, s: HWISubcommand) -> &mut Self {
        self.command.arg(s.to_string().to_lowercase());
        self
    }

    /// Adds a HWIFlag to a HWICommand
    /// # Examples
    /// ```
    /// use hwi::commands::{HWICommand, HWIFlag, HWISubcommand};
    ///
    /// let mut command = HWICommand::new();
    /// command
    ///     .add_subcommand(HWISubcommand::Enumerate)
    ///     .add_flag(HWIFlag::Testnet);
    /// ```
    pub fn add_flag(&mut self, f: HWIFlag) -> &mut Self {
        self.command.args(f.to_args_vec());
        self
    }

    /// Adds a DerivationPath to a HWICommand.
    /// # Arguments
    /// * `derivation_path` - the derivation path to add
    /// * `flag` - whether to add `--flag` or not - some Subcommands require it, some others don't.
    /// * `star` - whether to add a `/*` at the end of the derivation path
    pub fn add_path(
        &mut self,
        derivation_path: &DerivationPath,
        flag: bool,
        star: bool,
    ) -> &mut Self {
        let mut v = Vec::new();

        if flag {
            v.push(String::from("--path"));
        }

        let mut s = derivation_path.to_string();

        if star {
            s = format!("{}/*", s);
        }

        v.push(s);

        self.command.args(v);
        self
    }

    /// Adds a message to a HWICommand
    /// # Arguments
    /// * `message` - the message to add. Note that it's escaped, preventing injections.
    pub fn add_message(&mut self, message: &str) -> &mut Self {
        self.command.arg(message);
        self
    }

    /// Adds a descriptor (and `--desc` flag) to a HWICommand
    pub fn add_descriptor(&mut self, d: &str) -> &mut Self {
        self.command.args(vec!["--desc", d]);
        self
    }

    /// Adds a BIP43 account (and `--account` flag) to a HWICommand
    pub fn add_account(&mut self, a: u32) -> &mut Self {
        self.command.args(vec!["--account", &a.to_string()[..]]);
        self
    }

    /// Adds the address type flag to a HWICommand
    pub fn add_address_type(&mut self, address_type: HWIAddressType) -> &mut Self {
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

    /// Adds a PSBT to a HWICommand
    pub fn add_psbt(&mut self, p: &str) -> &mut Self {
        self.command.arg(p);
        self
    }

    /// Adds keypool flag to a HWICommand
    pub fn add_keypool(&mut self, k: bool) -> &mut Self {
        if !k {
            self.command.arg("--nokeypool");
        }
        self
    }

    /// Adds internal flag to a HWICommand
    pub fn add_internal(&mut self, i: bool) -> &mut Self {
        if i {
            self.command.arg("--internal");
        }
        self
    }

    /// Adds start and end to a HWICommand (useful for keypools)
    pub fn add_start_end(&mut self, start: u32, end: u32) -> &mut Self {
        self.command
            .args(vec![format!("{}", start), format!("{}", end)]);
        self
    }

    /// Adds testnet flag to a HWICommand
    pub fn add_testnet(&mut self, testnet: bool) -> &mut Self {
        if testnet {
            self.add_flag(HWIFlag::Testnet);
        }
        self
    }

    /// Executes a HWICommand
    /// # Examples
    ///
    /// ```
    /// use hwi::commands::{HWICommand, HWISubcommand};
    ///
    /// let output = HWICommand::new()
    ///     .add_subcommand(HWISubcommand::Enumerate)
    ///     .execute();
    /// ```
    pub fn execute(&mut self) -> Result<Output, Error> {
        Ok(self.command.output()?)
    }
}
