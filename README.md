# rust-hwi
Rust wrapper for [HWI](https://github.com/bitcoin-core/HWI/).

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
$ git clone https://github.com/MagicalBitcoin/rust-hwi.git && cd rust-hwi
```

- Create a virtualenv:

```
$ virtualenv -p python3 venv
$ source venv/bin/activate
```

- Install all the dependencies using pip:

```
pip install -r requirements.txt
```

## Supported commands

| Command | Supported? |
|:---:|:---: |
| enumerate | YES |
| getmasterxpub | YES |
| signtx | YES |
| getxpub | YES |
| signmessage | YES |
| getkeypool | YES |
| getdescriptors | YES |
| displayaddress | YES | 
| setup | Planned |
| wipe | Planned |
| restore | Planned |
| backup | Planned |
| promptpin | Planned |
| sendpin | Planned |

| Flag | Supported? |
|:---:|:---:|
| --device-path | YES |
| --device-type | YES |
| --password | Planned |
| --stdinpass | NO |
| --testnet | Planned |
| --debug | Planned |
| --fingerprint | YES |
| --version | Planned |
| --stdin | NO |
| --interactive | Planned |

## Tests

Unfortunatly at the moment you'll need a HW plugged in to be able to run tests.

If you don't have a hardware wallet, you can try [coldcard simulator](https://github.com/Coldcard/firmware).

To run tests you should:

- Install requirements and activate the virtualenv, as specified before
- Plug in a HW.
- `cargo test`

## Devices tested
| Device | Tested |
|:---:|:---:|
| Ledger Nano X | NO
| Ledger Nano S | YES
| Trezor One | NO
| Trezor Model T | YES
| Digital BitBox | NO
| KeepKey | NO
| Coldcard | YES
