#[derive(Debug)]
pub enum Error {
    JSON(serde_json::error::Error),
    Utf8(std::str::Utf8Error),
    IOError(std::io::Error),
}

macro_rules! impl_error {
    ( $from:ty, $to:ident ) => {
        impl std::convert::From<$from> for Error {
            fn from(err: $from) -> Self {
                Error::$to(err)
            }
        }
    };
}

impl_error!(serde_json::Error, JSON);
impl_error!(std::str::Utf8Error, Utf8);
impl_error!(std::io::Error, IOError);
