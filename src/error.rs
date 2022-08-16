#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    JSON(String),
    Utf8(String),
    IOError(String),
    InvalidOption(String),
    HWIError(String),
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
