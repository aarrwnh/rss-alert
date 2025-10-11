#![allow(dead_code)]
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Generic(String),
    Io(std::io::Error),
    Fmt(std::fmt::Error),
    Reqwest(reqwest::Error),
    // RoxmltreError(roxmltree::Error),
    // VarError(std::env::VarError),
    // ParseInt(std::num::ParseIntError),
}
macro_rules! err {
    ($s:ty, $en:ident) => {
        impl From<$s> for Error {
            fn from(e: $s) -> Self {
                Self::$en(e)
            }
        }
    };
}
err!(String, Generic);
err!(std::io::Error, Io);
err!(std::fmt::Error, Fmt);
err!(reqwest::Error, Reqwest);
// err!(roxmltree::Error, RoxmltreError);
// err!(std::env::VarError, VarError);
// err!(std::num::ParseIntError, ParseInt);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Generic(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::Fmt(e) => write!(f, "{e}"),
            Self::Reqwest(e) => write!(f, "{e}"),
            // Self::RoxmltreError(e) => f.debug_tuple("RoxmltreError").field(e).finish(),
            // Self::VarError(e) => f.debug_tuple("VarError").field(e).finish(),
            // Self::ParseInt(e) => f.debug_tuple("ParseIntError").field(e).finish(),
        }
    }
}
impl std::error::Error for Error {}
