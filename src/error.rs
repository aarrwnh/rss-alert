pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// #[derive(Debug)]
// pub enum Error {
//     Generic(String),
//     IoError(std::io::Error),
//     FmtError(std::fmt::Error),
//     Reqwest(reqwest::Error),
// }
// macro_rules! err {
//     ($s:ty, $en:ident) => {
//         impl From<$s> for Error {
//             fn from(e: $s) -> Self {
//                 Self::$en(e)
//             }
//         }
//     };
// }
// err!(String, Generic);
// err!(std::io::Error, IoError);
// err!(std::fmt::Error, FmtError);
// err!(reqwest::Error, Reqwest);
// impl std::fmt::Display for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Generic(e) => write!(f, "{e}"),
//             Self::IoError(e) => write!(f, "{e}"),
//             Self::FmtError(e) => write!(f, "{e}"),
//             Self::Reqwest(e) => write!(f, "{e}"),
//         }
//     }
// }
// impl std::error::Error for Error {}
