#[macro_use] extern crate strum_macros;

pub mod types;
pub mod commands;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    /*
    #[test]
    fn commands() {
        use crate::commands::{HWICommand, Subcommand, Flag};
        use std::process::Command;
        use std::string::ToString;
        let mut HWIcommand = HWICommand::new();
        HWIcommand.add_subcommand(Subcommand::Enumerate);
        HWIcommand.add_flag(Flag::Password(String::from("123")));
        HWIcommand.add_flag(Flag::StdinPass);
        println!("{:?}", HWIcommand.command);
    }
    */
}
