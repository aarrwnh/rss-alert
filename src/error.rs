#![allow(dead_code)]
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Generic(String),
    Io(std::io::Error),
    Fmt(std::fmt::Error),
    Reqwest(reqwest::Error),
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Generic(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::Fmt(e) => write!(f, "{e}"),
            Self::Reqwest(e) => write!(f, "{e}"),
        }
    }
}
impl std::error::Error for Error {}
