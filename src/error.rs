use std::convert::TryFrom;
use std::{fmt, io, str};

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
            _ => return Err(Error::Hwi("invalid error code".to_string(), None)),
        };
        Ok(code)
    }
}

impl fmt::Debug for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_i8())
    }
}

#[derive(Debug)]
pub enum Error {
    Json(serde_json::Error),
    Utf8(std::str::Utf8Error),
    Io(std::io::Error),
    Hwi(String, Option<ErrorCode>),
    Python(pyo3::PyErr),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;

        match *self {
            Json(_) => f.write_str("serde_json error"),
            Utf8(_) => f.write_str("utf8 error"),
            Io(_) => f.write_str("I/O error"),
            Hwi(ref s, ref code) => write!(f, "HWI error: {}, ({:?})", s, code),
            Python(_) => f.write_str("python error"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use self::Error::*;

        match *self {
            Json(ref e) => Some(e),
            Utf8(ref e) => Some(e),
            Io(ref e) => Some(e),
            Hwi(_, _) => None,
            Python(ref e) => Some(e),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(e: str::Utf8Error) -> Self {
        Error::Utf8(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<pyo3::PyErr> for Error {
    fn from(e: pyo3::PyErr) -> Self {
        Error::Python(e)
    }
}
