use std::sync::LazyLock;

use tauri_winrt_notification::{IconCrop, Toast};

pub mod config;
pub mod item;

mod error;
pub use error::Result;

mod timestamp;
pub use timestamp::Timestamp;

static ICON_PATH: LazyLock<std::path::PathBuf> =
    LazyLock::new(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("rss.png"));

// ----------------------------------------------------------------------------------
//   - Toaster -
// ----------------------------------------------------------------------------------
pub trait Toastable: std::fmt::Debug {
    fn get_title(&self) -> &str;
    fn get_link(&self) -> &str;
    fn get_timestamp(&self) -> i64;

    fn show_toast(&self, wait_sec: std::time::Duration) {
        Toast::new(Toast::POWERSHELL_APP_ID)
            .title(self.get_title())
            .text1(self.get_link())
            .icon(&ICON_PATH, IconCrop::Square, "rss")
            .sound(None)
            .show()
            .expect("unable to show toast notification");
        std::thread::sleep(wait_sec);
    }
}

// ----------------------------------------------------------------------------------
// enum AppError {
//     RoxmltreError(roxmltree::Error),
//     VarError(std::env::VarError),
//     ParseInt(ParseIntError),
// }

// impl std::fmt::Debug for AppError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::RoxmltreError(e) => f.debug_tuple("RoxmltreError").field(e).finish(),
//             Self::VarError(e) => f.debug_tuple("VarError").field(e).finish(),
//             Self::ParseInt(e) => f.debug_tuple("ParseIntError").field(e).finish(),
//         }
//     }
// }

// impl From<roxmltree::Error> for AppError {
//     fn from(value: roxmltree::Error) -> Self {
//         Self::RoxmltreError(value)
//     }
// }

// impl From<std::env::VarError> for AppError {
//     fn from(value: std::env::VarError) -> Self {
//         Self::VarError(value)
//     }
// }

// impl From<ParseIntError> for AppError {
//     fn from(value: ParseIntError) -> Self {
//         Self::ParseInt(value)
//     }
// }
