use std::process::{Command, Output};

use bitcoin::util::bip32::{DerivationPath, Fingerprint};

use crate::error::Error;
use crate::types::{HWIAddressType, HWIChain};

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
    Chain(HWIChain),
    AddrType(HWIAddressType),
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
            HWIFlag::DevicePath(p) => vec![String::from("--device-path"), p.to_string()],
            HWIFlag::DeviceType(t) => vec![String::from("--device-type"), t.to_string()],
            HWIFlag::Password(p) => vec![String::from("--password"), p.to_string()],
            HWIFlag::Fingerprint(f) => vec![String::from("--fingerprint"), f.to_string()],
            HWIFlag::Chain(chain) => vec![
                String::from("--chain"),
                format!("{:?}", chain).to_lowercase(),
            ],
            HWIFlag::AddrType(a) => vec![
                String::from("--addr-type"),
                format!("{:?}", a).to_lowercase(),
            ],
        }
    }
}

#[derive(Debug)]
pub struct HWICommand {
    command: Command,
}

impl Default for HWICommand {
    fn default() -> Self {
        Self::new()
    }
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
    /// use hwi::types::HWIChain;
    ///
    /// let mut command = HWICommand::new();
    /// command
    ///     .add_subcommand(HWISubcommand::Enumerate)
    ///     .add_flag(HWIFlag::Chain(HWIChain::Test));
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

    /// Adds expert flag to a HWICommand
    /// # Arguments
    /// * `e` - whether to add "--expert" flag or not.
    pub fn add_expert(&mut self, e: bool) -> &mut Self {
        if e {
            self.command.arg("--expert");
        }
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
        self.add_flag(HWIFlag::AddrType(address_type));
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

    /// Adds chain flag and arguements to a HWICommand
    pub fn add_chain(&mut self, chain: HWIChain) -> &mut Self {
        self.add_flag(HWIFlag::Chain(chain));
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
