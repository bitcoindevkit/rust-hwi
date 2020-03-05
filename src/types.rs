use bitcoin::util::bip32::Fingerprint;

pub struct Device {
    device_type: String,
    model: String,
    path: String,
    needs_pin_sent: bool,
    need_passphrase_sent: bool,
    fingerprint: Fingerprint
}
