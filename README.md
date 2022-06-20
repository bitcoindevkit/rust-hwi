# rust-hwi
Rust wrapper for the [Bitcoin Hardware Wallet Interface](https://github.com/bitcoin-core/HWI/) library.

<a href="https://crates.io/crates/hwi"><img alt="Crate Info" src="https://img.shields.io/crates/v/hwi.svg"/></a>
<a href="https://docs.rs/hwi"><img alt="API Docs" src="https://img.shields.io/badge/docs.rs-hwi-green"/></a>
<a href="https://blog.rust-lang.org/2020/02/27/Rust-1.41.1.html"><img alt="Rustc Version 1.41+" src="https://img.shields.io/badge/rustc-1.41%2B-lightgrey.svg"/></a>
<a href="https://discord.gg/d7NkDKm"><img alt="Chat on Discord" src="https://img.shields.io/discord/753336465005608961?logo=discord"></a>

This library internally uses PyO3 to call HWI's functions. It is not a re-implementation of HWI in native Rust.

## Prerequisites

Python 3 is required. The libraries and [udev rules](https://github.com/bitcoin-core/HWI/blob/master/hwilib/udev/README.md) for each device must also be installed. Some libraries will need to be installed

For Ubuntu/Debian:
```
sudo apt install libusb-1.0-0-dev libudev-dev python3-dev
```

For Centos:
```
sudo yum -y install python3-devel libusbx-devel systemd-devel
```

For macOS:
```
brew install libusb
```

## Install

- Clone the repo
```
git clone https://github.com/bitcoindevkit/rust-hwi.git && cd rust-hwi
```

- Create a virtualenv:

```
virtualenv -p python3 venv
source venv/bin/activate
```

- Install all the dependencies using pip:

```
pip install -r requirements.txt
```

## Usage

```rust
use bitcoin::util::bip32::DerivationPath;
use hwi::error::Error;
use hwi::{interface, types, HWIClient};
use std::str::FromStr;

fn main() -> Result<(), Error> {
    let devices = interface::HWIClient::enumerate()?;
    let device = devices.first().expect("No devices found");
    let client = HWIClient::get_client(
        &device,
        true,
        types::HWIChain::Test,
    )?;
    let derivation_path = DerivationPath::from_str("m/44'/1'/0'/0/0").unwrap();
    let s = client.sign_message("I love BDK wallet", &derivation_path)?;
    println!("{:?}", s.signature);
    Ok(())
}
```

## Testing

To run the tests, you need to have a hardware wallet plugged in. If you don't have a HW for testing, you can try:
- [Coldcard simulator](https://github.com/Coldcard/firmware)
- [Trezor simulator](https://docs.trezor.io/trezor-firmware/core/emulator/index.html)
- [Ledger simulator](https://github.com/LedgerHQ/speculos)

**Don't use a device with funds for testing!**

Either use a testing device with no funds, or use a simulator.

You can run the tests with `cargo test`.