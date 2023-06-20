use std::convert::TryFrom;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ErrorCode {
    NoDeviceType = 1,
    MissingArguments = 2,
    DeviceConnError = 3,
    UnknownDeviceType = 4,
    InvalidTx = 5,
    NoPassword = 6,
    BadArgument = 7,
    NotImplemented = 8,
    UnavailableAction = 9,
    DeviceAlreadyInit = 10,
    DeviceAlreadyUnlocked = 11,
    DeviceNotReady = 12,
    UnknownError = 13,
    ActionCanceled = 14,
    DeviceBusy = 15,
    NeedToBeRoot = 16,
    HelpText = 17,
    DeviceNotInitialized = 18,
}

impl ErrorCode {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn as_i8(&self) -> i8 {
        -(*self as i8)
    }
}

impl TryFrom<i8> for ErrorCode {
    type Error = Error;

    fn try_from(code: i8) -> Result<Self, Error> {
        use ErrorCode::*;

        let code = match code {
            -1 => NoDeviceType,
            -2 => MissingArguments,
            -3 => DeviceConnError,
            -4 => UnknownDeviceType,
            -5 => InvalidTx,
            -6 => NoPassword,
            -7 => BadArgument,
            -8 => NotImplemented,
            -9 => UnavailableAction,
            -10 => DeviceAlreadyInit,
            -11 => DeviceAlreadyUnlocked,
            -12 => DeviceNotReady,
            -13 => UnknownError,
            -14 => ActionCanceled,
            -15 => DeviceBusy,
            -16 => NeedToBeRoot,
            -17 => HelpText,
            -18 => DeviceNotInitialized,
            _ => return Err(Error::HWIError("Invalid error code".to_string(), None)),
        };
        Ok(code)
    }
}

impl fmt::Debug for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_i8())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    JSON(String),
    Utf8(String),
    IOError(String),
    InvalidOption(String),
    HWIError(String, Option<ErrorCode>),
    PyErr(String),
}

macro_rules! impl_error {
    ( $from:ty, $to:ident ) => {
        impl std::convert::From<$from> for Error {
            fn from(err: $from) -> Self {
                Error::$to(err.to_string())
            }
        }
    };
}

impl_error!(serde_json::Error, JSON);
impl_error!(std::str::Utf8Error, Utf8);
impl_error!(std::io::Error, IOError);
impl_error!(pyo3::prelude::PyErr, PyErr);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
